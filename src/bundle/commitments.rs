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
        #[cfg(feature = "hybrid-kem")]
        {
            nh.update(&action.encrypted_note().ct_pq);
            // The diversifier hint is transmitted on-chain and steers the recipient's
            // per-diversifier decapsulation; it must be committed so it cannot be
            // mutated without invalidating the bundle (txid) commitment.
            nh.update(&action.encrypted_note().diversifier_hint);
        }
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

#[cfg(all(test, feature = "hybrid-kem"))]
mod tests {
    use group::{Group, GroupEncoding};
    use nonempty::NonEmpty;
    use pasta_curves::pallas;
    use zcash_note_encryption::note_bytes::NoteBytesData;

    use super::hash_bundle_txid_data;
    use crate::{
        action::Action,
        bundle::{Bundle, EffectsOnly, Flags},
        memo::ZcashMemo,
        note::{ExtractedNoteCommitment, Nullifier, TransmittedNoteCiphertext},
        primitives::redpallas::{SpendAuth, VerificationKey},
        value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
        Anchor,
    };

    /// Builds a single-action bundle whose only varying field is the diversifier hint;
    /// every other committed field (nf, rk, cmx, cv_net, ciphertexts) is identical.
    fn bundle_with_hint(
        nf: Nullifier,
        rk: VerificationKey<SpendAuth>,
        cmx: ExtractedNoteCommitment,
        cv_net: ValueCommitment,
        diversifier_hint: [u8; 11],
    ) -> Bundle<EffectsOnly, i64> {
        // `Action::from_parts` requires `epk_bytes` to encode a non-identity
        // Pallas point, so use a fixed valid one.
        let epk_bytes = pallas::Point::generator().to_bytes();
        let encrypted_note = TransmittedNoteCiphertext::<ZcashMemo>::from_parts(
            epk_bytes,
            NoteBytesData([0u8; 580]),
            [0u8; 1088],
            diversifier_hint,
            [0u8; 112],
        );
        let action =
            Action::from_parts(nf, rk, cmx, encrypted_note, cv_net, ()).expect("non-identity rk");
        Bundle::from_parts(
            NonEmpty::from_vec(vec![action]).unwrap(),
            Flags::ENABLED,
            0i64,
            Anchor::from_bytes([0u8; 32]).unwrap(),
            EffectsOnly,
        )
    }

    #[test]
    fn diversifier_hint_is_committed_in_txid() {
        // Shared, fixed components so the only difference between the two bundles
        // is the diversifier hint.
        let nf = Nullifier::from_bytes(&[1u8; 32]).unwrap();
        let rk = VerificationKey::dummy();
        let cmx = ExtractedNoteCommitment::from_bytes(&[2u8; 32]).unwrap();
        let cv_net = ValueCommitment::derive(ValueSum::from_raw(0), ValueCommitTrapdoor::zero());

        let a = hash_bundle_txid_data(&bundle_with_hint(
            nf,
            rk.clone(),
            cmx,
            cv_net.clone(),
            [1u8; 11],
        ));

        let mut flipped = [1u8; 11];
        flipped[0] ^= 0xff; // change only the hint
        let b = hash_bundle_txid_data(&bundle_with_hint(nf, rk, cmx, cv_net, flipped));

        assert_ne!(
            a.as_bytes(),
            b.as_bytes(),
            "mutating only the diversifier hint must change the bundle (txid) commitment"
        );
    }
}
