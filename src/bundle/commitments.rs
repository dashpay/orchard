//! Utility functions for computing bundle commitments

use blake2b_simd::{Hash as Blake2bHash, Params, State};

use crate::bundle::{Authorization, Authorized, Bundle};
use crate::memo::{MemoSize, COMPACT_NOTE_SIZE};

const ZCASH_ORCHARD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActCHash";
const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActNHash";
const ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";

fn hasher(personal: &[u8; 16]) -> State {
    Params::new().hash_length(32).personal(personal).to_state()
}

/// Write disjoint parts of each Orchard shielded action as 3 separate hashes
/// as defined in [ZIP-244: Transaction Identifier Non-Malleability][zip244]:
/// * \[(nullifier, cmx, ephemeral_key, enc_ciphertext\[..52\])*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION
/// * \[enc_ciphertext\[52..564\]*\] (memo ciphertexts) personalized
///   with ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION
/// * \[(cv, rk, enc_ciphertext\[564..\], out_ciphertext)*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION
///
/// Then, hash these together along with (flags, value_balance_orchard, anchor_orchard),
/// personalized with ZCASH_ORCHARD_ACTIONS_HASH_PERSONALIZATION
///
/// [zip244]: https://zips.z.cash/zip-0244
pub(crate) fn hash_bundle_txid_data<A: Authorization, V: Copy + Into<i64>, M: MemoSize>(
    bundle: &Bundle<A, V, M>,
) -> Blake2bHash {
    // Guarantees `enc_ciphertext` is at least `COMPACT_NOTE_SIZE + AEAD_TAG_SIZE`
    // bytes, so the slicing below cannot panic; a mis-sized `MemoSize`
    // implementation is rejected at compile time.
    #[allow(clippy::let_unit_value)]
    let _ = M::SIZE_CHECK;

    let mut h = hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION);
    let mut ch = hasher(ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION);
    let mut mh = hasher(ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION);
    let mut nh = hasher(ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION);

    for action in bundle.actions().iter() {
        let enc = action.encrypted_note().enc_ciphertext.as_ref();
        let aead_tag_start = enc.len() - 16;

        ch.update(&action.nullifier().to_bytes());
        ch.update(&action.cmx().to_bytes());
        ch.update(&action.encrypted_note().epk_bytes);
        ch.update(&enc[..COMPACT_NOTE_SIZE]);

        mh.update(&enc[COMPACT_NOTE_SIZE..aead_tag_start]);

        nh.update(&action.cv_net().to_bytes());
        nh.update(&<[u8; 32]>::from(action.rk()));
        nh.update(&enc[aead_tag_start..]);
        nh.update(&action.encrypted_note().out_ciphertext);
    }

    h.update(ch.finalize().as_bytes());
    h.update(mh.finalize().as_bytes());
    h.update(nh.finalize().as_bytes());
    h.update(&[bundle.flags().to_byte()]);
    h.update(&(*bundle.value_balance()).into().to_le_bytes());
    h.update(&bundle.anchor().to_bytes());
    h.finalize()
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_txid_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION).finalize()
}

/// Construct the commitment to the authorizing data of an
/// authorized bundle as defined in [ZIP-244: Transaction
/// Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub(crate) fn hash_bundle_auth_data<V, M: MemoSize>(
    bundle: &Bundle<Authorized, V, M>,
) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION);
    h.update(bundle.authorization().proof().as_ref());
    for action in bundle.actions().iter() {
        h.update(&<[u8; 64]>::from(action.authorization()));
    }
    h.update(&<[u8; 64]>::from(
        bundle.authorization().binding_signature(),
    ));
    h.finalize()
}

/// Construct the commitment for an absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_auth_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION).finalize()
}

#[cfg(test)]
mod tests {
    use nonempty::NonEmpty;
    use zcash_note_encryption::Domain;

    use super::hash_bundle_txid_data;
    use crate::{
        action::Action,
        bundle::{Bundle, EffectsOnly, Flags},
        keys::{FullViewingKey, Scope, SpendingKey},
        memo::{DashMemo, MemoSize, ZcashMemo},
        note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho, TransmittedNoteCiphertext},
        note_encryption::{OrchardDomain, OrchardNoteEncryption},
        primitives::redpallas,
        tree::Anchor,
        value::{NoteValue, ValueCommitment},
        Note,
    };

    /// Builds a single-action bundle entirely from fixed inputs, so its txid
    /// digest is a stable golden value for the given memo size.
    fn deterministic_bundle<M: MemoSize>(memo: M::Memo) -> Bundle<EffectsOnly, i64, M> {
        let sk = SpendingKey::from_bytes([0x0d; 32]).unwrap();
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        let nf = Nullifier::from_bytes(&[0x01; 32]).unwrap();
        let rho = Rho::from_nf_old(nf);
        let rseed = RandomSeed::from_bytes([0x02; 32], &rho).unwrap();
        let note = Note::from_parts(recipient, NoteValue::from_raw(97_000), rho, rseed).unwrap();

        // esk (and thus epk and the ciphertext) is derived from rseed, so the
        // encryption is deterministic.
        let ne = OrchardNoteEncryption::<M>::new(None, note, memo);
        let epk_bytes = OrchardDomain::<M>::epk_bytes(ne.epk());
        let encrypted_note = TransmittedNoteCiphertext::<M>::from_parts(
            epk_bytes.0,
            ne.encrypt_note_plaintext(),
            [0x03; 80],
        );

        let rsk = redpallas::SigningKey::try_from([0x21; 32]).unwrap();
        let action = Action::from_parts(
            nf,
            redpallas::VerificationKey::from(&rsk),
            ExtractedNoteCommitment::from(note.commitment()),
            encrypted_note,
            ValueCommitment::from_bytes(&[0u8; 32]).unwrap(),
            (),
        )
        .unwrap();

        Bundle::from_parts(
            NonEmpty::new(action),
            Flags::ENABLED,
            0i64,
            Anchor::from_bytes([0u8; 32]).unwrap(),
            EffectsOnly,
        )
    }

    /// Golden vectors pinning the txid digest for both supported memo sizes.
    /// A change to these digests is a consensus-visible wire format change.
    #[test]
    fn txid_digest_golden_vectors() {
        let zcash_bundle = deterministic_bundle::<ZcashMemo>([0xa5; 512]);
        assert_eq!(
            hex::encode(hash_bundle_txid_data(&zcash_bundle).as_bytes()),
            "c32a1001c23eb79113356f5e8223c5b6c91f1ba8726068f5cdad1baf324c5bb4"
        );

        let dash_bundle = deterministic_bundle::<DashMemo>([0xa5; 36]);
        assert_eq!(
            hex::encode(hash_bundle_txid_data(&dash_bundle).as_bytes()),
            "4129796966e923cb34134db4e743c5e8aeb10b8fe43f5964f4ed4dfbb21b7f7e"
        );
    }
}
