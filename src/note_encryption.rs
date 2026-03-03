//! In-band secret distribution for Orchard bundles.

use alloc::vec::Vec;
#[cfg(feature = "hybrid-kem")]
use alloc::boxed::Box;
use core::fmt;
use core::marker::PhantomData;

use blake2b_simd::{Hash, Params};
use group::ff::PrimeField;
#[cfg(not(feature = "hybrid-kem"))]
use zcash_note_encryption::{OutPlaintextBytes, OUT_CIPHERTEXT_SIZE, OUT_PLAINTEXT_SIZE};

use zcash_note_encryption::{
    note_bytes::{NoteBytes, NoteBytesData},
    BatchDomain, Domain, EphemeralKeyBytes, OutgoingCipherKey, ShieldedOutput,
};

use crate::{
    action::Action,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        OutgoingViewingKey, PreparedEphemeralPublicKey, PreparedIncomingViewingKey,
    },
    memo::{MemoSize, ZcashMemo, COMPACT_NOTE_SIZE},
    note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho},
    value::{NoteValue, ValueCommitment},
    Address, Note,
};

#[cfg(not(feature = "hybrid-kem"))]
use crate::keys::SharedSecret;

#[cfg(feature = "hybrid-kem")]
use crate::{
    hybrid_kem::{self, PQ_CT_SIZE},
    keys::{HybridSharedSecret, PqEncapsulationKey},
};

const PRF_OCK_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_Orchardock";

// Hybrid out plaintext/ciphertext sizes (pk_d:32 + esk:32 + ss_pq:32 = 96, + 16 tag = 112)
#[cfg(feature = "hybrid-kem")]
const HYBRID_OUT_PLAINTEXT_SIZE: usize = 96;
#[cfg(feature = "hybrid-kem")]
const HYBRID_OUT_CIPHERTEXT_SIZE: usize = 112;

/// Outgoing plaintext bytes for hybrid mode: pk_d(32) || esk(32) || ss_pq(32).
#[cfg(feature = "hybrid-kem")]
#[derive(Clone, Debug)]
pub struct HybridOutPlaintextBytes(pub [u8; HYBRID_OUT_PLAINTEXT_SIZE]);

#[cfg(feature = "hybrid-kem")]
impl AsRef<[u8]> for HybridOutPlaintextBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(feature = "hybrid-kem")]
impl AsMut<[u8]> for HybridOutPlaintextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Wrapper for the `DiversifiedTransmissionKey` Domain associated type in hybrid mode.
///
/// For encryption, carries the PQ encapsulation key from the recipient's address.
/// For output recovery, carries the PQ shared secret from the decrypted out_plaintext.
#[cfg(feature = "hybrid-kem")]
#[derive(Clone, Debug)]
pub struct OrchardDTK {
    pub(crate) inner: DiversifiedTransmissionKey,
    pub(crate) pq: OrchardDTKPq,
}

#[cfg(feature = "hybrid-kem")]
#[derive(Clone, Debug)]
pub(crate) enum OrchardDTKPq {
    /// Normal encryption: encapsulate using this key.
    Encapsulate(Box<PqEncapsulationKey>),
    /// Output recovery: use the stored shared secret and ciphertext directly.
    Recovery {
        ss_pq: [u8; 32],
        ct_pq: Box<[u8; PQ_CT_SIZE]>,
    },
    /// No PQ context (classic-mode address or compact decryption).
    None,
}

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
    // Hybrid mode uses 0x03, classic uses 0x02. Accept both for decryption
    // (the version byte distinguishes transition-period notes).
    #[cfg(feature = "hybrid-kem")]
    if plaintext[0] != 0x02 && plaintext[0] != 0x03 {
        return None;
    }
    #[cfg(not(feature = "hybrid-kem"))]
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
    let note = Option::from(Note::from_parts(recipient.clone(), value, domain.rho, rseed))?;
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
}

impl OrchardDomain<ZcashMemo> {
    /// Constructs a domain that can be used to trial-decrypt a PCZT action's output note.
    pub fn for_pczt_action(act: &crate::pczt::Action) -> Self {
        Self {
            base: OrchardDomainBase {
                rho: Rho::from_nf_old(act.spend().nullifier),
            },
            _memo: PhantomData,
        }
    }
}

impl<M: MemoSize> Domain for OrchardDomain<M> {
    type EphemeralSecretKey = EphemeralSecretKey;
    type EphemeralPublicKey = EphemeralPublicKey;
    type PreparedEphemeralPublicKey = PreparedEphemeralPublicKey;
    #[cfg(feature = "hybrid-kem")]
    type SharedSecret = HybridSharedSecret;
    #[cfg(not(feature = "hybrid-kem"))]
    type SharedSecret = SharedSecret;
    type SymmetricKey = Hash;
    type Note = Note;
    type Recipient = Address;
    #[cfg(feature = "hybrid-kem")]
    type DiversifiedTransmissionKey = OrchardDTK;
    #[cfg(not(feature = "hybrid-kem"))]
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
    #[cfg(feature = "hybrid-kem")]
    type OutPlaintextBytes = HybridOutPlaintextBytes;
    #[cfg(not(feature = "hybrid-kem"))]
    type OutPlaintextBytes = OutPlaintextBytes;
    #[cfg(feature = "hybrid-kem")]
    type OutCiphertextBytes = [u8; HYBRID_OUT_CIPHERTEXT_SIZE];
    #[cfg(not(feature = "hybrid-kem"))]
    type OutCiphertextBytes = [u8; OUT_CIPHERTEXT_SIZE];

    fn zero_out_plaintext_bytes() -> Self::OutPlaintextBytes {
        #[cfg(feature = "hybrid-kem")]
        {
            HybridOutPlaintextBytes([0u8; HYBRID_OUT_PLAINTEXT_SIZE])
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            OutPlaintextBytes([0u8; OUT_PLAINTEXT_SIZE])
        }
    }

    fn zero_out_ciphertext_bytes() -> Self::OutCiphertextBytes {
        #[cfg(feature = "hybrid-kem")]
        {
            [0u8; HYBRID_OUT_CIPHERTEXT_SIZE]
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            [0u8; OUT_CIPHERTEXT_SIZE]
        }
    }

    fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey> {
        Some(note.esk())
    }

    fn get_pk_d(note: &Self::Note) -> Self::DiversifiedTransmissionKey {
        #[cfg(feature = "hybrid-kem")]
        {
            OrchardDTK {
                inner: *note.recipient().pk_d(),
                pq: match note.recipient().ek_pq() {
                    Some(ek) => OrchardDTKPq::Encapsulate(Box::new(ek.clone())),
                    None => OrchardDTKPq::None,
                },
            }
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            *note.recipient().pk_d()
        }
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
        #[cfg(feature = "hybrid-kem")]
        {
            let ecdh_secret = esk.agree(&pk_d.inner);
            match &pk_d.pq {
                OrchardDTKPq::Encapsulate(ek_pq) => {
                    // Encapsulation can only fail if the ek is structurally invalid,
                    // which should not happen for keys derived from a valid spending key.
                    // In the unlikely event of failure, fall back to ECDH-only.
                    match hybrid_kem::encapsulate_deterministic(&ek_pq.0, &esk.pq_randomness) {
                        Ok((ct_pq, pq_ss)) => HybridSharedSecret {
                            ecdh: ecdh_secret.inner(),
                            pq_ss,
                            ct_pq,
                        },
                        Err(_) => HybridSharedSecret {
                            ecdh: ecdh_secret.inner(),
                            pq_ss: [0u8; 32],
                            ct_pq: [0u8; PQ_CT_SIZE],
                        },
                    }
                }
                OrchardDTKPq::Recovery { ss_pq, ct_pq } => {
                    // Recovery path: use stored ss_pq and ct_pq from the output
                    HybridSharedSecret {
                        ecdh: ecdh_secret.inner(),
                        pq_ss: *ss_pq,
                        ct_pq: **ct_pq,
                    }
                }
                OrchardDTKPq::None => {
                    // Fallback: ECDH-only shared secret with zero PQ values
                    HybridSharedSecret {
                        ecdh: ecdh_secret.inner(),
                        pq_ss: [0u8; 32],
                        ct_pq: [0u8; PQ_CT_SIZE],
                    }
                }
            }
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            esk.agree(pk_d)
        }
    }

    fn ka_agree_dec(
        ivk: &Self::IncomingViewingKey,
        epk: &Self::PreparedEphemeralPublicKey,
    ) -> Self::SharedSecret {
        #[cfg(feature = "hybrid-kem")]
        {
            let ecdh_secret = epk.agree(ivk);
            // ECDH-only fallback for ka_agree_dec; the PQ path uses ka_agree_dec_with_pq
            HybridSharedSecret {
                ecdh: ecdh_secret.inner(),
                pq_ss: [0u8; 32],
                ct_pq: [0u8; PQ_CT_SIZE],
            }
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            epk.agree(ivk)
        }
    }

    #[cfg(feature = "hybrid-kem")]
    fn ka_agree_dec_with_pq(
        ivk: &Self::IncomingViewingKey,
        epk: &Self::PreparedEphemeralPublicKey,
        pq_ct: Option<&[u8]>,
    ) -> Self::SharedSecret {
        let ecdh_secret = epk.agree(ivk);
        if let Some(ct_bytes) = pq_ct {
            if let (Some(pq_dk), Ok(ct_arr)) = (
                ivk.pq_dk.as_ref(),
                <&[u8; PQ_CT_SIZE]>::try_from(ct_bytes),
            ) {
                let pq_ss = hybrid_kem::decapsulate(&pq_dk.0, ct_arr);
                let mut ct_pq = [0u8; PQ_CT_SIZE];
                ct_pq.copy_from_slice(ct_bytes);
                return HybridSharedSecret {
                    ecdh: ecdh_secret.inner(),
                    pq_ss,
                    ct_pq,
                };
            }
        }
        // Fallback: ECDH only
        HybridSharedSecret {
            ecdh: ecdh_secret.inner(),
            pq_ss: [0u8; 32],
            ct_pq: [0u8; PQ_CT_SIZE],
        }
    }

    fn kdf(secret: Self::SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Self::SymmetricKey {
        #[cfg(feature = "hybrid-kem")]
        {
            secret.kdf_hybrid(ephemeral_key)
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            secret.kdf_orchard(ephemeral_key)
        }
    }

    fn note_plaintext_bytes(note: &Self::Note, memo: &Self::Memo) -> Self::NotePlaintextBytes {
        let mut np = [0u8; COMPACT_NOTE_SIZE];
        // Version byte: 0x03 for hybrid, 0x02 for classic
        #[cfg(feature = "hybrid-kem")]
        {
            np[0] = 0x03;
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            np[0] = 0x02;
        }
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
    ) -> Self::OutPlaintextBytes {
        #[cfg(feature = "hybrid-kem")]
        {
            let mut op = [0u8; HYBRID_OUT_PLAINTEXT_SIZE];
            op[..32].copy_from_slice(&note.recipient().pk_d().to_bytes());
            op[32..64].copy_from_slice(&esk.ecdh.to_repr());
            // Re-derive ss_pq deterministically from ek_pq + pq_randomness
            if let Some(ek_pq) = note.recipient().ek_pq() {
                if let Ok((_, ss_pq)) =
                    hybrid_kem::encapsulate_deterministic(&ek_pq.0, &esk.pq_randomness)
                {
                    op[64..96].copy_from_slice(&ss_pq);
                }
            }
            HybridOutPlaintextBytes(op)
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            let mut op = [0; OUT_PLAINTEXT_SIZE];
            op[..32].copy_from_slice(&note.recipient().pk_d().to_bytes());
            op[32..].copy_from_slice(&esk.ecdh.to_repr());
            OutPlaintextBytes(op)
        }
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
        #[cfg(feature = "hybrid-kem")]
        {
            orchard_parse_note_plaintext_without_memo(&self.base, plaintext.as_ref(), |_| {
                pk_d.inner
            })
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            orchard_parse_note_plaintext_without_memo(&self.base, plaintext.as_ref(), |_| *pk_d)
        }
    }

    fn split_plaintext_at_memo(
        &self,
        plaintext: &Self::NotePlaintextBytes,
    ) -> Option<(Self::CompactNotePlaintextBytes, Self::Memo)> {
        let bytes = plaintext.as_ref();
        if bytes.len() < COMPACT_NOTE_SIZE {
            return None;
        }
        let compact = NoteBytesData::<COMPACT_NOTE_SIZE>::from_slice(&bytes[..COMPACT_NOTE_SIZE])?;
        let memo = M::memo_from_bytes(&bytes[COMPACT_NOTE_SIZE..])?;
        Some((compact, memo))
    }

    fn extract_pk_d(
        out_plaintext: &Self::OutPlaintextBytes,
    ) -> Option<Self::DiversifiedTransmissionKey> {
        #[cfg(feature = "hybrid-kem")]
        {
            // Use extract_pk_d_for_recovery with no PQ ciphertext
            Self::extract_pk_d_for_recovery(out_plaintext, None)
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap())
                .into()
        }
    }

    #[cfg(feature = "hybrid-kem")]
    fn extract_pk_d_for_recovery(
        out_plaintext: &Self::OutPlaintextBytes,
        pq_ciphertext: Option<&[u8]>,
    ) -> Option<Self::DiversifiedTransmissionKey> {
        let pk_d =
            DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap());
        let pk_d: Option<DiversifiedTransmissionKey> = pk_d.into();
        pk_d.map(|inner| {
            // Extract ss_pq from the out plaintext for recovery
            let mut ss_pq = [0u8; 32];
            ss_pq.copy_from_slice(&out_plaintext.0[64..96]);
            // Extract ct_pq from the output for the KDF
            let mut ct_pq = [0u8; PQ_CT_SIZE];
            if let Some(ct) = pq_ciphertext {
                if ct.len() == PQ_CT_SIZE {
                    ct_pq.copy_from_slice(ct);
                }
            }
            OrchardDTK {
                inner,
                pq: OrchardDTKPq::Recovery { ss_pq, ct_pq: Box::new(ct_pq) },
            }
        })
    }

    fn extract_esk(
        out_plaintext: &Self::OutPlaintextBytes,
    ) -> Option<Self::EphemeralSecretKey> {
        #[cfg(feature = "hybrid-kem")]
        {
            EphemeralSecretKey::from_bytes(out_plaintext.0[32..64].try_into().unwrap()).into()
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            EphemeralSecretKey::from_bytes(
                out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap(),
            )
            .into()
        }
    }
}

impl<M: MemoSize> BatchDomain for OrchardDomain<M> {
    fn batch_kdf<'a>(
        items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
    ) -> Vec<Option<Self::SymmetricKey>> {
        #[cfg(feature = "hybrid-kem")]
        {
            // In hybrid mode, each shared secret may carry PQ data, so we KDF individually.
            items
                .map(|(secret, ephemeral_key)| {
                    secret.map(|s| s.kdf_hybrid(ephemeral_key))
                })
                .collect()
        }
        #[cfg(not(feature = "hybrid-kem"))]
        {
            let (shared_secrets, ephemeral_keys): (Vec<_>, Vec<_>) = items.unzip();

            SharedSecret::batch_to_affine(shared_secrets)
                .zip(ephemeral_keys)
                .map(|(secret, ephemeral_key)| {
                    secret
                        .map(|dhsecret| SharedSecret::kdf_orchard_inner(dhsecret, ephemeral_key))
                })
                .collect()
        }
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

    #[cfg(feature = "hybrid-kem")]
    fn pq_ciphertext(&self) -> Option<&[u8]> {
        Some(&self.encrypted_note().ct_pq)
    }
}

impl ShieldedOutput<OrchardDomain<ZcashMemo>> for crate::pczt::Action {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.output().encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.output().cmx()
    }

    fn enc_ciphertext(&self) -> Option<&<ZcashMemo as MemoSize>::NoteCiphertextBytes> {
        Some(&self.output().encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> NoteBytesData<COMPACT_NOTE_SIZE> {
        NoteBytesData::<COMPACT_NOTE_SIZE>::from_slice(
            &self.output().encrypted_note().enc_ciphertext.as_ref()[..COMPACT_NOTE_SIZE],
        )
        .expect("enc_ciphertext is at least COMPACT_NOTE_SIZE bytes")
    }

    #[cfg(feature = "hybrid-kem")]
    fn pq_ciphertext(&self) -> Option<&[u8]> {
        Some(&self.output().encrypted_note().ct_pq)
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
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let encryptor = OrchardNoteEncryption::<ZcashMemo>::new(ovk, note.clone(), [0u8; 512]);
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

// The test_vectors test uses classic Orchard test vectors (version byte 0x02,
// 80-byte out_ciphertext) which are incompatible with hybrid mode sizes.
// Hybrid-specific encryption tests are in Phase 8.
#[cfg(all(test, not(feature = "hybrid-kem")))]
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
            let note = Note::from_parts(recipient.clone(), value, rho, rseed).unwrap();
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
}

#[cfg(all(test, feature = "hybrid-kem"))]
mod hybrid_tests {
    use rand::rngs::OsRng;
    use zcash_note_encryption::{
        try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk,
    };

    use super::{CompactAction, OrchardDomain};
    use crate::{
        keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey},
        memo::ZcashMemo,
        note::{Nullifier, Rho},
        value::NoteValue,
        Note,
    };

    fn test_key_material() -> (SpendingKey, FullViewingKey) {
        let sk = SpendingKey::from_bytes([7; 32]).unwrap();
        let fvk = FullViewingKey::from(&sk);
        (sk, fvk)
    }

    /// Test that hybrid mode encrypts and decrypts correctly with the IVK.
    #[test]
    fn hybrid_round_trip_ivk() {
        let mut rng = OsRng;
        let (_, fvk) = test_key_material();
        let recipient = fvk.address_at(0u32, Scope::External);

        let nf = Nullifier::dummy(&mut rng);
        let rho = Rho::from_nf_old(nf);
        let note = Note::new(recipient, NoteValue::from_raw(42), rho, &mut rng);

        let ne = super::OrchardNoteEncryption::<ZcashMemo>::new(
            Some(fvk.to_ovk(Scope::External)),
            note.clone(),
            [0u8; 512],
        );

        let esk = note.esk();
        let epk = esk.derive_public(note.recipient().g_d());
        let epk_bytes = epk.to_bytes();

        let enc_ciphertext = ne.encrypt_note_plaintext();

        let cmx = crate::note::ExtractedNoteCommitment::from(note.commitment());
        let cv_net = crate::value::ValueCommitment::derive(
            NoteValue::from_raw(42) - NoteValue::zero(),
            crate::value::ValueCommitTrapdoor::random(&mut rng),
        );

        let out_ciphertext = ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut rng);

        // Derive ct_pq deterministically from note
        let note_esk = note.esk();
        let note_recipient = note.recipient();
        let ek_pq = note_recipient.ek_pq().expect("hybrid address has ek_pq");
        let (ct_pq, _) =
            crate::hybrid_kem::encapsulate_deterministic(&ek_pq.0, &note_esk.pq_randomness)
                .expect("encapsulation should succeed");

        let action: crate::action::Action<(), ZcashMemo> = crate::action::Action::from_parts(
            nf,
            crate::primitives::redpallas::VerificationKey::dummy(),
            cmx,
            crate::note::TransmittedNoteCiphertext::from_parts(
                epk_bytes.0,
                enc_ciphertext,
                ct_pq,
                out_ciphertext,
            ),
            cv_net,
            (),
        );

        // Decrypt with IVK (has PQ dk via FVK chain)
        let ivk = fvk.to_ivk(Scope::External);
        let prepared_ivk = PreparedIncomingViewingKey::new(&ivk);
        let domain = OrchardDomain::<ZcashMemo>::for_action(&action);

        let result = try_note_decryption(&domain, &prepared_ivk, &action);
        assert!(result.is_some(), "Hybrid note decryption should succeed");
        let (decrypted_note, decrypted_addr, decrypted_memo) = result.expect("decryption");
        assert_eq!(decrypted_note.value(), note.value());
        assert_eq!(decrypted_addr, note.recipient());
        assert_eq!(&decrypted_memo[..], &[0u8; 512][..]);
    }

    /// Compact decryption uses ECDH-only, which produces a different KDF result
    /// than the hybrid KDF used to encrypt. Compact decryption of hybrid notes
    /// will fail — light clients need full actions with ct_pq.
    #[test]
    fn hybrid_compact_decryption_fails() {
        let mut rng = OsRng;
        let (_, fvk) = test_key_material();
        let recipient = fvk.address_at(0u32, Scope::External);

        let nf = Nullifier::dummy(&mut rng);
        let rho = Rho::from_nf_old(nf);
        let note = Note::new(recipient, NoteValue::from_raw(100), rho, &mut rng);

        let ne = super::OrchardNoteEncryption::<ZcashMemo>::new(None, note.clone(), [0u8; 512]);

        let esk = note.esk();
        let epk = esk.derive_public(note.recipient().g_d());
        let epk_bytes = epk.to_bytes();
        let enc_ciphertext = ne.encrypt_note_plaintext();

        let cmx = crate::note::ExtractedNoteCommitment::from(note.commitment());
        let cv_net = crate::value::ValueCommitment::derive(
            NoteValue::from_raw(100) - NoteValue::zero(),
            crate::value::ValueCommitTrapdoor::random(&mut rng),
        );

        let out_ciphertext = ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut rng);

        let note_esk = note.esk();
        let note_recipient = note.recipient();
        let ek_pq = note_recipient.ek_pq().expect("hybrid address has ek_pq");
        let (ct_pq, _) =
            crate::hybrid_kem::encapsulate_deterministic(&ek_pq.0, &note_esk.pq_randomness)
                .expect("encapsulation should succeed");

        let action: crate::action::Action<(), ZcashMemo> = crate::action::Action::from_parts(
            nf,
            crate::primitives::redpallas::VerificationKey::dummy(),
            cmx,
            crate::note::TransmittedNoteCiphertext::from_parts(
                epk_bytes.0,
                enc_ciphertext,
                ct_pq,
                out_ciphertext,
            ),
            cv_net,
            (),
        );

        let ivk = fvk.to_ivk(Scope::External);
        let prepared_ivk = PreparedIncomingViewingKey::new(&ivk);
        let domain = OrchardDomain::<ZcashMemo>::for_action(&action);

        // Compact decryption should fail for hybrid notes
        let compact = CompactAction::from(&action);
        let result = try_compact_note_decryption(&domain, &prepared_ivk, &compact);
        assert!(
            result.is_none(),
            "Compact decryption should fail for hybrid notes (different KDF)"
        );
    }

    /// Test output recovery with OVK in hybrid mode.
    #[test]
    fn hybrid_output_recovery_ovk() {
        let mut rng = OsRng;
        let (_, fvk) = test_key_material();
        let recipient = fvk.address_at(0u32, Scope::External);
        let ovk = fvk.to_ovk(Scope::External);

        let nf = Nullifier::dummy(&mut rng);
        let rho = Rho::from_nf_old(nf);
        let note = Note::new(recipient, NoteValue::from_raw(500), rho, &mut rng);

        let ne = super::OrchardNoteEncryption::<ZcashMemo>::new(
            Some(ovk.clone()),
            note.clone(),
            [42u8; 512],
        );

        let esk = note.esk();
        let epk = esk.derive_public(note.recipient().g_d());
        let epk_bytes = epk.to_bytes();
        let enc_ciphertext = ne.encrypt_note_plaintext();

        let cmx = crate::note::ExtractedNoteCommitment::from(note.commitment());
        let cv_net = crate::value::ValueCommitment::derive(
            NoteValue::from_raw(500) - NoteValue::zero(),
            crate::value::ValueCommitTrapdoor::random(&mut rng),
        );

        let out_ciphertext = ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut rng);

        let note_esk = note.esk();
        let note_recipient = note.recipient();
        let ek_pq = note_recipient.ek_pq().expect("hybrid address has ek_pq");
        let (ct_pq, _) =
            crate::hybrid_kem::encapsulate_deterministic(&ek_pq.0, &note_esk.pq_randomness)
                .expect("encapsulation should succeed");

        let action: crate::action::Action<(), ZcashMemo> = crate::action::Action::from_parts(
            nf,
            crate::primitives::redpallas::VerificationKey::dummy(),
            cmx,
            crate::note::TransmittedNoteCiphertext::from_parts(
                epk_bytes.0,
                enc_ciphertext,
                ct_pq,
                out_ciphertext,
            ),
            cv_net.clone(),
            (),
        );

        let domain = OrchardDomain::<ZcashMemo>::for_action(&action);
        let result =
            try_output_recovery_with_ovk(&domain, &ovk, &action, &cv_net, &out_ciphertext);
        assert!(
            result.is_some(),
            "Hybrid output recovery with OVK should succeed"
        );
        let (recovered_note, recovered_addr, recovered_memo) = result.expect("recovery");
        assert_eq!(recovered_note.value(), note.value());
        assert_eq!(recovered_addr, note.recipient());
        assert_eq!(&recovered_memo[..], &[42u8; 512][..]);
    }

    /// Test that PQ key derivation from spending key is deterministic.
    #[test]
    fn hybrid_key_derivation_deterministic() {
        let sk = SpendingKey::from_bytes([7; 32]).unwrap();
        let fvk1 = FullViewingKey::from(&sk);
        let fvk2 = FullViewingKey::from(&sk);

        let addr1 = fvk1.address_at(0u32, Scope::External);
        let addr2 = fvk2.address_at(0u32, Scope::External);

        assert_eq!(addr1, addr2);
        assert_eq!(
            addr1.ek_pq().expect("has ek_pq").to_bytes(),
            addr2.ek_pq().expect("has ek_pq").to_bytes(),
        );
    }

    /// Test that different spending keys produce different PQ keys.
    #[test]
    fn hybrid_different_keys_different_pq() {
        let sk1 = SpendingKey::from_bytes([7; 32]).unwrap();
        let sk2 = SpendingKey::from_bytes([8; 32]).unwrap();
        let fvk1 = FullViewingKey::from(&sk1);
        let fvk2 = FullViewingKey::from(&sk2);

        let addr1 = fvk1.address_at(0u32, Scope::External);
        let addr2 = fvk2.address_at(0u32, Scope::External);

        assert_ne!(
            addr1.ek_pq().expect("has ek_pq").to_bytes(),
            addr2.ek_pq().expect("has ek_pq").to_bytes(),
        );
    }
}
