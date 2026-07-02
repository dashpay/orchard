//! In-band secret distribution for Orchard bundles.

use alloc::vec::Vec;
use core::fmt;
use core::marker::PhantomData;

use blake2b_simd::{Hash, Params};
use group::ff::PrimeField;
use zcash_note_encryption::{
    note_bytes::{NoteBytes, NoteBytesData},
    BatchDomain, Domain, EphemeralKeyBytes, OutPlaintextBytes, OutgoingCipherKey, ShieldedOutput,
    OUT_PLAINTEXT_SIZE,
};

use crate::{
    action::Action,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        OutgoingViewingKey, PreparedEphemeralPublicKey, PreparedIncomingViewingKey, SharedSecret,
    },
    memo::{MemoSize, ZcashMemo, COMPACT_NOTE_SIZE},
    note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho},
    value::{NoteValue, ValueCommitment},
    Address, Note,
};

const PRF_OCK_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_Orchardock";

/// Defined in [Zcash Protocol Spec section 5.4.2: Pseudo Random Functions][concreteprfs].
///
/// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
pub(crate) fn prf_ock_orchard(
    ovk: &OutgoingViewingKey,
    cv: &ValueCommitment,
    cmx_bytes: &[u8; 32],
    ephemeral_key: &EphemeralKeyBytes,
) -> OutgoingCipherKey {
    OutgoingCipherKey(
        Params::new()
            .hash_length(32)
            .personal(PRF_OCK_ORCHARD_PERSONALIZATION)
            .to_state()
            .update(ovk.as_ref())
            .update(&cv.to_bytes())
            .update(cmx_bytes)
            .update(ephemeral_key.as_ref())
            .finalize()
            .as_bytes()
            .try_into()
            .unwrap(),
    )
}

fn orchard_parse_note_plaintext_without_memo<F>(
    domain: &OrchardDomainBase,
    plaintext: &[u8],
    get_pk_d: F,
) -> Option<(Note, Address)>
where
    F: FnOnce(&Diversifier) -> DiversifiedTransmissionKey,
{
    assert!(plaintext.len() >= COMPACT_NOTE_SIZE);

    // Check note plaintext version
    if plaintext[0] != 0x02 {
        return None;
    }

    // The unwraps below are guaranteed to succeed by the assertion above
    let diversifier = Diversifier::from_bytes(plaintext[1..12].try_into().unwrap());
    let value = NoteValue::from_bytes(plaintext[12..20].try_into().unwrap());
    let rseed = Option::from(RandomSeed::from_bytes(
        plaintext[20..COMPACT_NOTE_SIZE].try_into().unwrap(),
        &domain.rho,
    ))?;

    let pk_d = get_pk_d(&diversifier);

    let recipient = Address::from_parts(diversifier, pk_d);
    let note = Option::from(Note::from_parts(recipient, value, domain.rho, rseed))?;
    Some((note, recipient))
}

/// The non-generic core of [`OrchardDomain`].
///
/// This holds the protocol-specific state (just `rho`) that is independent of the
/// memo size parameter `M`.
#[derive(Debug)]
struct OrchardDomainBase {
    rho: Rho,
}

/// Orchard-specific note encryption logic.
///
/// The type parameter `M` determines the memo size (and thus the note plaintext
/// and ciphertext sizes). The default is [`ZcashMemo`] (512-byte memos).
#[derive(Debug)]
pub struct OrchardDomain<M: MemoSize = ZcashMemo> {
    base: OrchardDomainBase,
    _memo: PhantomData<M>,
}

impl<M: MemoSize> memuse::DynamicUsage for OrchardDomain<M> {
    fn dynamic_usage(&self) -> usize {
        self.base.rho.dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        self.base.rho.dynamic_usage_bounds()
    }
}

impl<M: MemoSize> OrchardDomain<M> {
    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T, M>) -> Self {
        Self {
            base: OrchardDomainBase { rho: act.rho() },
            _memo: PhantomData,
        }
    }

    /// Constructs a domain from a nullifier (used for PCZT actions).
    pub fn for_nullifier(nf: Nullifier) -> Self {
        Self {
            base: OrchardDomainBase {
                rho: Rho::from_nf_old(nf),
            },
            _memo: PhantomData,
        }
    }

    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_compact_action(act: &CompactAction) -> Self {
        Self {
            base: OrchardDomainBase { rho: act.rho() },
            _memo: PhantomData,
        }
    }

    /// Constructs a domain that can be used to trial-decrypt a PCZT action's output note.
    pub fn for_pczt_action(act: &crate::pczt::Action<M>) -> Self {
        Self::for_nullifier(act.spend().nullifier)
    }
}

impl<M: MemoSize> Domain for OrchardDomain<M> {
    type EphemeralSecretKey = EphemeralSecretKey;
    type EphemeralPublicKey = EphemeralPublicKey;
    type PreparedEphemeralPublicKey = PreparedEphemeralPublicKey;
    type SharedSecret = SharedSecret;
    type SymmetricKey = Hash;
    type Note = Note;
    type Recipient = Address;
    type DiversifiedTransmissionKey = DiversifiedTransmissionKey;
    type IncomingViewingKey = PreparedIncomingViewingKey;
    type OutgoingViewingKey = OutgoingViewingKey;
    type ValueCommitment = ValueCommitment;
    type ExtractedCommitment = ExtractedNoteCommitment;
    type ExtractedCommitmentBytes = [u8; 32];
    type Memo = M::Memo;

    type NotePlaintextBytes = M::NotePlaintextBytes;
    type NoteCiphertextBytes = M::NoteCiphertextBytes;
    type CompactNotePlaintextBytes = NoteBytesData<COMPACT_NOTE_SIZE>;
    type CompactNoteCiphertextBytes = NoteBytesData<COMPACT_NOTE_SIZE>;

    fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey> {
        Some(note.esk())
    }

    fn get_pk_d(note: &Self::Note) -> Self::DiversifiedTransmissionKey {
        *note.recipient().pk_d()
    }

    fn prepare_epk(epk: Self::EphemeralPublicKey) -> Self::PreparedEphemeralPublicKey {
        PreparedEphemeralPublicKey::new(epk)
    }

    fn ka_derive_public(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> Self::EphemeralPublicKey {
        esk.derive_public(note.recipient().g_d())
    }

    fn ka_agree_enc(
        esk: &Self::EphemeralSecretKey,
        pk_d: &Self::DiversifiedTransmissionKey,
    ) -> Self::SharedSecret {
        esk.agree(pk_d)
    }

    fn ka_agree_dec(
        ivk: &Self::IncomingViewingKey,
        epk: &Self::PreparedEphemeralPublicKey,
    ) -> Self::SharedSecret {
        epk.agree(ivk)
    }

    fn kdf(secret: Self::SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Self::SymmetricKey {
        secret.kdf_orchard(ephemeral_key)
    }

    fn note_plaintext_bytes(note: &Self::Note, memo: &Self::Memo) -> Self::NotePlaintextBytes {
        // Rejects mis-sized `MemoSize` implementations at compile time.
        #[allow(clippy::let_unit_value)]
        let _ = M::SIZE_CHECK;

        let mut np = [0u8; COMPACT_NOTE_SIZE];
        np[0] = 0x02;
        np[1..12].copy_from_slice(note.recipient().diversifier().as_array());
        np[12..20].copy_from_slice(&note.value().to_bytes());
        np[20..COMPACT_NOTE_SIZE].copy_from_slice(note.rseed().as_bytes());

        let mut buf = alloc::vec![0u8; COMPACT_NOTE_SIZE + memo.as_ref().len()];
        buf[..COMPACT_NOTE_SIZE].copy_from_slice(&np);
        buf[COMPACT_NOTE_SIZE..].copy_from_slice(memo.as_ref());

        Self::NotePlaintextBytes::from_slice(&buf).expect("memo size is consistent with M")
    }

    fn derive_ock(
        ovk: &Self::OutgoingViewingKey,
        cv: &Self::ValueCommitment,
        cmstar_bytes: &Self::ExtractedCommitmentBytes,
        ephemeral_key: &EphemeralKeyBytes,
    ) -> OutgoingCipherKey {
        prf_ock_orchard(ovk, cv, cmstar_bytes, ephemeral_key)
    }

    fn outgoing_plaintext_bytes(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> OutPlaintextBytes {
        let mut op = [0; OUT_PLAINTEXT_SIZE];
        op[..32].copy_from_slice(&note.recipient().pk_d().to_bytes());
        op[32..].copy_from_slice(&esk.0.to_repr());
        OutPlaintextBytes(op)
    }

    fn epk_bytes(epk: &Self::EphemeralPublicKey) -> EphemeralKeyBytes {
        epk.to_bytes()
    }

    fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<Self::EphemeralPublicKey> {
        EphemeralPublicKey::from_bytes(&ephemeral_key.0).into()
    }

    fn cmstar(note: &Self::Note) -> Self::ExtractedCommitment {
        note.commitment().into()
    }

    fn parse_note_plaintext_without_memo_ivk(
        &self,
        ivk: &Self::IncomingViewingKey,
        plaintext: &Self::CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        orchard_parse_note_plaintext_without_memo(&self.base, plaintext.as_ref(), |diversifier| {
            DiversifiedTransmissionKey::derive(ivk, diversifier)
        })
    }

    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &Self::DiversifiedTransmissionKey,
        plaintext: &Self::CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        orchard_parse_note_plaintext_without_memo(&self.base, plaintext.as_ref(), |_| *pk_d)
    }

    fn split_plaintext_at_memo(
        &self,
        plaintext: &Self::NotePlaintextBytes,
    ) -> Option<(Self::CompactNotePlaintextBytes, Self::Memo)> {
        // Rejects mis-sized `MemoSize` implementations at compile time.
        #[allow(clippy::let_unit_value)]
        let _ = M::SIZE_CHECK;

        let bytes = plaintext.as_ref();
        if bytes.len() < COMPACT_NOTE_SIZE {
            return None;
        }
        let compact = NoteBytesData::<COMPACT_NOTE_SIZE>::from_slice(&bytes[..COMPACT_NOTE_SIZE])?;
        let memo = M::memo_from_bytes(&bytes[COMPACT_NOTE_SIZE..])?;
        Some((compact, memo))
    }

    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<Self::DiversifiedTransmissionKey> {
        DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap()).into()
    }

    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey> {
        EphemeralSecretKey::from_bytes(out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap())
            .into()
    }
}

impl<M: MemoSize> BatchDomain for OrchardDomain<M> {
    fn batch_kdf<'a>(
        items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
    ) -> Vec<Option<Self::SymmetricKey>> {
        let (shared_secrets, ephemeral_keys): (Vec<_>, Vec<_>) = items.unzip();

        SharedSecret::batch_to_affine(shared_secrets)
            .zip(ephemeral_keys)
            .map(|(secret, ephemeral_key)| {
                secret.map(|dhsecret| SharedSecret::kdf_orchard_inner(dhsecret, ephemeral_key))
            })
            .collect()
    }
}

/// Implementation of in-band secret distribution for Orchard bundles.
pub type OrchardNoteEncryption<M = ZcashMemo> =
    zcash_note_encryption::NoteEncryption<OrchardDomain<M>>;

impl<T, M: MemoSize> ShieldedOutput<OrchardDomain<M>> for Action<T, M> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.cmx()
    }

    fn enc_ciphertext(&self) -> Option<&M::NoteCiphertextBytes> {
        Some(&self.encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> NoteBytesData<COMPACT_NOTE_SIZE> {
        NoteBytesData::<COMPACT_NOTE_SIZE>::from_slice(
            &self.encrypted_note().enc_ciphertext.as_ref()[..COMPACT_NOTE_SIZE],
        )
        .expect("enc_ciphertext is at least COMPACT_NOTE_SIZE bytes")
    }
}

impl<M: MemoSize> ShieldedOutput<OrchardDomain<M>> for crate::pczt::Action<M> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.output().encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.output().cmx()
    }

    fn enc_ciphertext(&self) -> Option<&M::NoteCiphertextBytes> {
        Some(&self.output().encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> NoteBytesData<COMPACT_NOTE_SIZE> {
        NoteBytesData::<COMPACT_NOTE_SIZE>::from_slice(
            &self.output().encrypted_note().enc_ciphertext.as_ref()[..COMPACT_NOTE_SIZE],
        )
        .expect("enc_ciphertext is at least COMPACT_NOTE_SIZE bytes")
    }
}

/// A compact Action for light clients.
#[derive(Clone)]
pub struct CompactAction {
    nullifier: Nullifier,
    cmx: ExtractedNoteCommitment,
    ephemeral_key: EphemeralKeyBytes,
    enc_ciphertext: [u8; COMPACT_NOTE_SIZE],
}

impl fmt::Debug for CompactAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactAction")
    }
}

impl<T, M: MemoSize> From<&Action<T, M>> for CompactAction {
    fn from(action: &Action<T, M>) -> Self {
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: action.ephemeral_key(),
            enc_ciphertext: action.encrypted_note().enc_ciphertext.as_ref()[..COMPACT_NOTE_SIZE]
                .try_into()
                .unwrap(),
        }
    }
}

impl<M: MemoSize> ShieldedOutput<OrchardDomain<M>> for CompactAction {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.ephemeral_key.0)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    fn enc_ciphertext(&self) -> Option<&M::NoteCiphertextBytes> {
        None
    }

    fn enc_ciphertext_compact(&self) -> NoteBytesData<COMPACT_NOTE_SIZE> {
        NoteBytesData(self.enc_ciphertext)
    }
}

impl CompactAction {
    /// Create a CompactAction from its constituent parts
    pub fn from_parts(
        nullifier: Nullifier,
        cmx: ExtractedNoteCommitment,
        ephemeral_key: EphemeralKeyBytes,
        enc_ciphertext: [u8; COMPACT_NOTE_SIZE],
    ) -> Self {
        Self {
            nullifier,
            cmx,
            ephemeral_key,
            enc_ciphertext,
        }
    }

    /// Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> Nullifier {
        self.nullifier
    }

    /// Returns the commitment to the new note being created.
    pub fn cmx(&self) -> ExtractedNoteCommitment {
        self.cmx
    }

    /// Obtains the [`Rho`] value that was used to construct the new note being created.
    pub fn rho(&self) -> Rho {
        Rho::from_nf_old(self.nullifier)
    }
}

/// Utilities for constructing test data.
#[cfg(feature = "test-dependencies")]
pub mod testing {
    use rand::RngCore;
    use zcash_note_encryption::Domain;

    use crate::{
        keys::OutgoingViewingKey,
        memo::ZcashMemo,
        note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho},
        value::NoteValue,
        Address, Note,
    };

    use super::{CompactAction, OrchardDomain, OrchardNoteEncryption};

    /// Creates a fake `CompactAction` paying the given recipient the specified value.
    ///
    /// Returns the `CompactAction` and the new note.
    pub fn fake_compact_action<R: RngCore>(
        rng: &mut R,
        nf_old: Nullifier,
        recipient: Address,
        value: NoteValue,
        ovk: Option<OutgoingViewingKey>,
    ) -> (CompactAction, Note) {
        let rho = Rho::from_nf_old(nf_old);
        let rseed = {
            loop {
                let mut bytes = [0; 32];
                rng.fill_bytes(&mut bytes);
                let rseed = RandomSeed::from_bytes(bytes, &rho);
                if rseed.is_some().into() {
                    break rseed.unwrap();
                }
            }
        };
        let note = Note::from_parts(recipient, value, rho, rseed).unwrap();
        let encryptor = OrchardNoteEncryption::<ZcashMemo>::new(ovk, note, [0u8; 512]);
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ephemeral_key = OrchardDomain::<ZcashMemo>::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();

        (
            CompactAction {
                nullifier: nf_old,
                cmx,
                ephemeral_key,
                enc_ciphertext: enc_ciphertext.as_ref()[..52].try_into().unwrap(),
            },
            note,
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;
    use zcash_note_encryption::{
        note_bytes::NoteBytesData, try_compact_note_decryption, try_note_decryption,
        try_output_recovery_with_ovk, EphemeralKeyBytes,
    };

    use super::{prf_ock_orchard, CompactAction, OrchardDomain, OrchardNoteEncryption};
    use crate::{
        action::Action,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
            OutgoingViewingKey, PreparedIncomingViewingKey,
        },
        memo::ZcashMemo,
        note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho, TransmittedNoteCiphertext},
        primitives::redpallas,
        value::{NoteValue, ValueCommitment},
        Address, Note,
    };

    #[test]
    fn test_vectors() {
        let test_vectors = crate::test_vectors::note_encryption::test_vectors();

        for tv in test_vectors {
            //
            // Load the test vector components
            //

            // Recipient key material
            let ivk = PreparedIncomingViewingKey::new(
                &IncomingViewingKey::from_bytes(&tv.incoming_viewing_key).unwrap(),
            );
            let ovk = OutgoingViewingKey::from(tv.ovk);
            let d = Diversifier::from_bytes(tv.default_d);
            let pk_d = DiversifiedTransmissionKey::from_bytes(&tv.default_pk_d).unwrap();

            // Received Action
            let cv_net = ValueCommitment::from_bytes(&tv.cv_net).unwrap();
            let nf_old = Nullifier::from_bytes(&tv.nf_old).unwrap();
            let rho = Rho::from_nf_old(nf_old);
            let cmx = ExtractedNoteCommitment::from_bytes(&tv.cmx).unwrap();

            let esk = EphemeralSecretKey::from_bytes(&tv.esk).unwrap();
            let ephemeral_key = EphemeralKeyBytes(tv.ephemeral_key);

            // Details about the expected note
            let value = NoteValue::from_raw(tv.v);
            let rseed = RandomSeed::from_bytes(tv.rseed, &rho).unwrap();

            //
            // Test the individual components
            //

            let shared_secret = esk.agree(&pk_d);
            assert_eq!(shared_secret.to_bytes(), tv.shared_secret);

            let k_enc = shared_secret.kdf_orchard(&ephemeral_key);
            assert_eq!(k_enc.as_bytes(), tv.k_enc);

            let ock = prf_ock_orchard(&ovk, &cv_net, &cmx.to_bytes(), &ephemeral_key);
            assert_eq!(ock.as_ref(), tv.ock);

            let recipient = Address::from_parts(d, pk_d);
            let note = Note::from_parts(recipient, value, rho, rseed).unwrap();
            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action: Action<(), ZcashMemo> = Action::from_parts(
                // nf_old is the nullifier revealed by the receiving Action.
                nf_old,
                // We don't need a real rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext::from_parts(
                    ephemeral_key.0,
                    NoteBytesData(tv.c_enc),
                    tv.c_out,
                ),
                cv_net.clone(),
                (),
            )
            .expect("a key returned by VerificationKey::dummy() is vanishingly unlikely to be the identity");

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = OrchardDomain::<ZcashMemo>::for_action(&action);

            match try_note_decryption(&domain, &ivk, &action) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Note decryption failed"),
            }

            match try_compact_note_decryption(&domain, &ivk, &CompactAction::from(&action)) {
                Some((decrypted_note, decrypted_to)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                }
                None => panic!("Compact note decryption failed"),
            }

            match try_output_recovery_with_ovk(&domain, &ovk, &action, &cv_net, &tv.c_out) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Output recovery failed"),
            }

            //
            // Test encryption
            //

            let ne =
                OrchardNoteEncryption::<ZcashMemo>::new_with_esk(esk, Some(ovk), note, tv.memo);

            assert_eq!(ne.encrypt_note_plaintext().as_ref(), &tv.c_enc[..]);
            assert_eq!(
                &ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut OsRng)[..],
                &tv.c_out[..]
            );
        }
    }

    #[test]
    fn pczt_trial_decryption_is_memo_size_generic() {
        use crate::{
            builder::{Builder, BundleType},
            constants::MERKLE_DEPTH_ORCHARD,
            keys::{FullViewingKey, Scope, SpendingKey},
            memo::DashMemo,
            tree::EMPTY_ROOTS,
        };

        let mut rng = OsRng;
        let sk = SpendingKey::random(&mut rng);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        // Run the Creator and Constructor roles with a non-Zcash memo size.
        let mut builder: Builder<DashMemo> = Builder::new(
            BundleType::DEFAULT,
            EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
        );
        let memo = [42u8; 36];
        builder
            .add_output(None, recipient, NoteValue::from_raw(5000), memo)
            .unwrap();
        let pczt_bundle = builder.build_for_pczt(&mut rng).unwrap().0;

        // A Signer role must be able to decrypt the output it is being asked
        // to authorize, without knowing which action carries the real note.
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
        let (note, address, decrypted_memo) = pczt_bundle
            .actions()
            .iter()
            .find_map(|action| {
                let domain = OrchardDomain::for_pczt_action(action);
                try_note_decryption(&domain, &ivk, action)
            })
            .expect("recipient can trial-decrypt their output");

        assert_eq!(note.value(), NoteValue::from_raw(5000));
        assert_eq!(address, recipient);
        assert_eq!(decrypted_memo, memo);
    }
}
