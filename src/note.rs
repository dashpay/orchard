//! Data structures used for note construction.
use core::fmt;
use core::marker::PhantomData;
use memuse::DynamicUsage;

use ff::PrimeField;
use group::GroupEncoding;
use pasta_curves::pallas;
use rand::RngCore;
use subtle::CtOption;

use crate::{
    address::RawAddress,
    keys::{EphemeralSecretKey, FullViewingKey, Scope, SpendingKey},
    memo::{MemoSize, ZcashMemo},
    spec::{to_base, to_scalar, NonZeroPallasScalar, PrfExpand},
    value::NoteValue,
    Address,
};

#[cfg(feature = "hybrid-kem")]
use crate::keys::PqEncapsulationKey;

#[cfg(not(feature = "unstable-voting-circuits"))]
pub(crate) mod commitment;
#[cfg(feature = "unstable-voting-circuits")]
pub mod commitment;
#[cfg(feature = "unstable-voting-circuits")]
pub use self::commitment::NoteCommitTrapdoor;
pub use self::commitment::{ExtractedNoteCommitment, NoteCommitment};

#[cfg(not(feature = "unstable-voting-circuits"))]
pub(crate) mod nullifier;
#[cfg(feature = "unstable-voting-circuits")]
pub mod nullifier;
pub use self::nullifier::Nullifier;

/// The randomness used to construct a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rho(pallas::Base);

// We know that `pallas::Base` doesn't allocate internally.
memuse::impl_no_dynamic_usage!(Rho);

impl Rho {
    /// Deserialize the rho value from a byte array.
    ///
    /// This should only be used in cases where the components of a `Note` are being serialized and
    /// stored individually. Use [`Action::rho`] or [`CompactAction::rho`] to obtain the [`Rho`]
    /// value otherwise.
    ///
    /// [`Action::rho`]: crate::action::Action::rho
    /// [`CompactAction::rho`]: crate::note_encryption::CompactAction::rho
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Base::from_repr(*bytes).map(Rho)
    }

    /// Serialize the rho value to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_repr()
    }

    /// Constructs the [`Rho`] value to be used to construct a new note from the revealed nullifier
    /// of the note being spent in the [`Action`] under construction.
    ///
    /// [`Action`]: crate::action::Action
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn from_nf_old(nf: Nullifier) -> Self {
        Rho(nf.inner())
    }

    /// Consumes `self` and returns the inner field element.
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn into_inner(self) -> pallas::Base {
        self.0
    }
}

/// The ZIP 212 seed randomness for a note.
#[derive(Copy, Clone, Debug)]
pub struct RandomSeed([u8; 32]);

impl RandomSeed {
    pub(crate) fn random(rng: &mut impl RngCore, rho: &Rho) -> Self {
        loop {
            let mut bytes = [0; 32];
            rng.fill_bytes(&mut bytes);
            let rseed = RandomSeed::from_bytes(bytes, rho);
            if rseed.is_some().into() {
                break rseed.unwrap();
            }
        }
    }

    /// Reads a note's random seed from bytes, given the note's rho value.
    ///
    /// Returns `None` if the rho value is not for the same note as the seed.
    pub fn from_bytes(rseed: [u8; 32], rho: &Rho) -> CtOption<Self> {
        let rseed = RandomSeed(rseed);
        let esk = rseed.esk_inner(rho);
        CtOption::new(rseed, esk.is_some())
    }

    /// Returns the byte array corresponding to this seed.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn psi(&self, rho: &Rho) -> pallas::Base {
        to_base(PrfExpand::PSI.with(&self.0, &rho.to_bytes()))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk_inner(&self, rho: &Rho) -> CtOption<NonZeroPallasScalar> {
        NonZeroPallasScalar::from_scalar(to_scalar(
            PrfExpand::ORCHARD_ESK.with(&self.0, &rho.to_bytes()),
        ))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk(&self, rho: &Rho) -> NonZeroPallasScalar {
        // We can't construct a RandomSeed for which this unwrap fails.
        self.esk_inner(rho).unwrap()
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn rcm(&self, rho: &Rho) -> commitment::NoteCommitTrapdoor {
        commitment::NoteCommitTrapdoor(to_scalar(
            PrfExpand::ORCHARD_RCM.with(&self.0, &rho.to_bytes()),
        ))
    }
}

/// A discrete amount of funds received by an address.
#[derive(Debug, Clone)]
pub struct Note {
    /// The recipient of the funds (43-byte on-chain form).
    recipient: RawAddress,
    /// The value of this note.
    value: NoteValue,
    /// A unique creation ID for this note.
    ///
    /// This is produced from the nullifier of the note that will be spent in the [`Action`] that
    /// creates this note.
    ///
    /// [`Action`]: crate::action::Action
    rho: Rho,
    /// The seed randomness for various note components.
    rseed: RandomSeed,
    /// The PQ encapsulation key, if available.
    ///
    /// This is `Some` when the note was created via [`Note::new`] from a full [`Address`],
    /// and `None` when reconstructed via [`Note::from_parts`] (e.g. decryption recovery).
    #[cfg(feature = "hybrid-kem")]
    ek_pq: Option<PqEncapsulationKey>,
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        // Notes are canonically defined by their commitments.
        ExtractedNoteCommitment::from(self.commitment())
            .eq(&ExtractedNoteCommitment::from(other.commitment()))
    }
}

impl Eq for Note {}

impl Note {
    /// Creates a `Note` from its component parts.
    ///
    /// Returns `None` if a valid [`NoteCommitment`] cannot be derived from the note.
    ///
    /// # Caveats
    ///
    /// This low-level constructor enforces that the provided arguments produce an
    /// internally valid `Note`. However, it allows notes to be constructed in a way that
    /// violates required security checks for note decryption, as specified in
    /// [Section 4.19] of the Zcash Protocol Specification. Users of this constructor
    /// should only call it with note components that have been fully validated by
    /// decrypting a received note according to [Section 4.19].
    ///
    /// [Section 4.19]: https://zips.z.cash/protocol/protocol.pdf#saplingandorchardinband
    pub fn from_parts(
        recipient: RawAddress,
        value: NoteValue,
        rho: Rho,
        rseed: RandomSeed,
    ) -> CtOption<Self> {
        let note = Note {
            recipient,
            value,
            rho,
            rseed,
            #[cfg(feature = "hybrid-kem")]
            ek_pq: None,
        };
        let has_commitment = note.commitment_inner().is_some();
        CtOption::new(note, has_commitment)
    }

    /// Generates a new note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn new(
        recipient: Address,
        value: NoteValue,
        rho: Rho,
        mut rng: impl RngCore,
    ) -> Self {
        #[cfg(feature = "hybrid-kem")]
        let ek_pq = Some(recipient.ek_pq().clone());
        let raw = recipient.into_raw();

        #[allow(unused_mut)]
        let mut note = loop {
            let note = Note::from_parts(raw, value, rho, RandomSeed::random(&mut rng, &rho));
            if note.is_some().into() {
                break note.unwrap();
            }
        };
        #[cfg(feature = "hybrid-kem")]
        {
            note.ek_pq = ek_pq;
        }
        note
    }

    /// Generates a dummy spent note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn dummy(
        rng: &mut impl RngCore,
        rho: Option<Rho>,
    ) -> (SpendingKey, FullViewingKey, Self) {
        let sk = SpendingKey::random(rng);
        let fvk: FullViewingKey = (&sk).into();
        let recipient = fvk.address_at(0u32, Scope::External);

        let note = Note::new(
            recipient,
            NoteValue::ZERO,
            rho.unwrap_or_else(|| Rho::from_nf_old(Nullifier::dummy(rng))),
            rng,
        );

        (sk, fvk, note)
    }

    /// Returns the recipient of this note as a [`RawAddress`].
    pub fn recipient(&self) -> RawAddress {
        self.recipient
    }

    /// Returns the PQ encapsulation key, if available.
    ///
    /// This is `Some` when the note was created via [`Note::new`] from a full [`Address`],
    /// and `None` when reconstructed via [`Note::from_parts`].
    #[cfg(feature = "hybrid-kem")]
    pub fn ek_pq(&self) -> Option<&PqEncapsulationKey> {
        self.ek_pq.as_ref()
    }

    /// Returns the value of this note.
    pub fn value(&self) -> NoteValue {
        self.value
    }

    /// Returns the rseed value of this note.
    pub fn rseed(&self) -> &RandomSeed {
        &self.rseed
    }

    /// Derives the ephemeral secret key for this note.
    pub(crate) fn esk(&self) -> EphemeralSecretKey {
        EphemeralSecretKey {
            ecdh: self.rseed.esk(&self.rho),
            #[cfg(feature = "hybrid-kem")]
            pq_randomness: {
                // Bind encapsulation randomness to the recipient's PQ key.
                // If ek_pq is not available (reconstruction path),
                // use a zero key — encapsulation won't be performed anyway.
                let ek_pq = self
                    .ek_pq
                    .as_ref()
                    .map(|ek| ek.0)
                    .unwrap_or([0u8; crate::hybrid_kem::PQ_EK_SIZE]);
                crate::hybrid_kem::derive_pq_encaps_randomness(
                    self.rseed.as_bytes(),
                    &self.rho.to_bytes(),
                    &ek_pq,
                )
            },
        }
    }

    /// Returns rho of this note.
    pub fn rho(&self) -> Rho {
        self.rho
    }

    /// Derives the commitment to this note.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    pub fn commitment(&self) -> NoteCommitment {
        // `Note` will always have a note commitment by construction.
        self.commitment_inner().unwrap()
    }

    /// Derives the commitment to this note.
    ///
    /// This is the internal fallible API, used to check at construction time that the
    /// note has a commitment. Once you have a [`Note`] object, use `note.commitment()`
    /// instead.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    fn commitment_inner(&self) -> CtOption<NoteCommitment> {
        let g_d = self.recipient.g_d();

        NoteCommitment::derive(
            g_d.to_bytes(),
            self.recipient.pk_d().to_bytes(),
            self.value,
            self.rho.0,
            self.rseed.psi(&self.rho),
            self.rseed.rcm(&self.rho),
        )
    }

    /// Derives the nullifier for this note.
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        Nullifier::derive(
            fvk.nk(),
            self.rho.0,
            self.rseed.psi(&self.rho),
            self.commitment(),
        )
    }
}

/// An encrypted note.
#[derive(Clone)]
pub struct TransmittedNoteCiphertext<M: MemoSize = ZcashMemo> {
    /// The serialization of the ephemeral public key
    pub epk_bytes: [u8; 32],
    /// The encrypted note ciphertext
    pub enc_ciphertext: M::NoteCiphertextBytes,
    /// The ML-KEM-768 ciphertext (hybrid mode only).
    #[cfg(feature = "hybrid-kem")]
    pub ct_pq: [u8; 1088],
    /// An ECDH-encrypted hint that allows the recipient to recover the diversifier
    /// used to derive the per-diversifier PQ keypair.
    #[cfg(feature = "hybrid-kem")]
    pub diversifier_hint: [u8; 11],
    /// An encrypted value that allows the holder of the outgoing cipher
    /// key for the note to recover the note plaintext.
    #[cfg(feature = "hybrid-kem")]
    pub out_ciphertext: [u8; 112],
    /// An encrypted value that allows the holder of the outgoing cipher
    /// key for the note to recover the note plaintext.
    #[cfg(not(feature = "hybrid-kem"))]
    pub out_ciphertext: [u8; 80],
    _memo: PhantomData<M>,
}

impl<M: MemoSize> TransmittedNoteCiphertext<M> {
    /// Constructs a `TransmittedNoteCiphertext` from its parts (classic mode).
    #[cfg(not(feature = "hybrid-kem"))]
    pub fn from_parts(
        epk_bytes: [u8; 32],
        enc_ciphertext: M::NoteCiphertextBytes,
        out_ciphertext: [u8; 80],
    ) -> Self {
        Self {
            epk_bytes,
            enc_ciphertext,
            out_ciphertext,
            _memo: PhantomData,
        }
    }

    /// Constructs a `TransmittedNoteCiphertext` from its parts (hybrid mode).
    #[cfg(feature = "hybrid-kem")]
    pub fn from_parts(
        epk_bytes: [u8; 32],
        enc_ciphertext: M::NoteCiphertextBytes,
        ct_pq: [u8; 1088],
        diversifier_hint: [u8; 11],
        out_ciphertext: [u8; 112],
    ) -> Self {
        Self {
            epk_bytes,
            enc_ciphertext,
            ct_pq,
            diversifier_hint,
            out_ciphertext,
            _memo: PhantomData,
        }
    }
}

impl<M: MemoSize> fmt::Debug for TransmittedNoteCiphertext<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("TransmittedNoteCiphertext");
        s.field("epk_bytes", &self.epk_bytes)
            .field("enc_ciphertext", &hex::encode(self.enc_ciphertext.as_ref()))
            .field("out_ciphertext", &hex::encode(self.out_ciphertext));
        #[cfg(feature = "hybrid-kem")]
        {
            s.field("ct_pq", &hex::encode(self.ct_pq));
            s.field("diversifier_hint", &hex::encode(self.diversifier_hint));
        }
        s.finish()
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use crate::{
        address::testing::arb_address, note::nullifier::testing::arb_nullifier, value::NoteValue,
    };

    use super::{Note, RandomSeed, Rho};

    prop_compose! {
        /// Generate an arbitrary random seed
        pub(crate) fn arb_rseed()(elems in prop::array::uniform32(prop::num::u8::ANY)) -> RandomSeed {
            RandomSeed(elems)
        }
    }

    prop_compose! {
        /// Generate an action without authorization data.
        pub fn arb_note(value: NoteValue)(
            address in arb_address(),
            rho in arb_nullifier().prop_map(Rho::from_nf_old),
            rseed in arb_rseed(),
        ) -> Note {
            #[cfg(feature = "hybrid-kem")]
            {
                let ek_pq = Some(address.ek_pq().clone());
                Note {
                    recipient: address.into_raw(),
                    value,
                    rho,
                    rseed,
                    ek_pq,
                }
            }
            #[cfg(not(feature = "hybrid-kem"))]
            {
                Note {
                    recipient: address,
                    value,
                    rho,
                    rseed,
                }
            }
        }
    }
}
