use subtle::CtOption;

use crate::{
    keys::{DiversifiedTransmissionKey, Diversifier},
    spec::{diversify_hash, NonIdentityPallasPoint},
};

#[cfg(feature = "hybrid-kem")]
use crate::{hybrid_kem::PQ_EK_SIZE, keys::PqEncapsulationKey};

/// Size of the full serialized hybrid payment address: the 43-byte [`RawAddress`]
/// followed by the 1184-byte PQ encapsulation key.
#[cfg(feature = "hybrid-kem")]
pub const HYBRID_ADDRESS_SIZE: usize = 43 + PQ_EK_SIZE;

// ── RawAddress (hybrid-kem only) ──────────────────────────────────────────

/// The 43-byte on-chain form of an Orchard payment address: `(Diversifier, pk_d)`.
///
/// Used internally by [`Note`], PCZT, and decryption recovery paths.
/// When the `hybrid-kem` feature is disabled, this is a type alias for [`Address`].
///
/// [`Note`]: crate::note::Note
#[cfg(feature = "hybrid-kem")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawAddress {
    d: Diversifier,
    pk_d: DiversifiedTransmissionKey,
}

#[cfg(feature = "hybrid-kem")]
impl RawAddress {
    pub(crate) fn from_parts(d: Diversifier, pk_d: DiversifiedTransmissionKey) -> Self {
        RawAddress { d, pk_d }
    }

    /// Returns the [`Diversifier`] for this `RawAddress`.
    pub fn diversifier(&self) -> Diversifier {
        self.d
    }

    pub(crate) fn g_d(&self) -> NonIdentityPallasPoint {
        diversify_hash(self.d.as_array())
    }

    pub(crate) fn pk_d(&self) -> &DiversifiedTransmissionKey {
        &self.pk_d
    }

    /// Serializes this address to its "raw" encoding as specified in [Zcash Protocol Spec § 5.6.4.2: Orchard Raw Payment Addresses][orchardpaymentaddrencoding]
    ///
    /// [orchardpaymentaddrencoding]: https://zips.z.cash/protocol/protocol.pdf#orchardpaymentaddrencoding
    pub fn to_raw_address_bytes(&self) -> [u8; 43] {
        let mut result = [0u8; 43];
        result[..11].copy_from_slice(self.d.as_array());
        result[11..].copy_from_slice(&self.pk_d.to_bytes());
        result
    }

    /// Parse an address from its "raw" encoding as specified in [Zcash Protocol Spec § 5.6.4.2: Orchard Raw Payment Addresses][orchardpaymentaddrencoding]
    ///
    /// [orchardpaymentaddrencoding]: https://zips.z.cash/protocol/protocol.pdf#orchardpaymentaddrencoding
    pub fn from_raw_address_bytes(bytes: &[u8; 43]) -> CtOption<Self> {
        DiversifiedTransmissionKey::from_bytes(bytes[11..43].try_into().unwrap()).map(|pk_d| {
            let d = Diversifier::from_bytes(bytes[..11].try_into().unwrap());
            Self::from_parts(d, pk_d)
        })
    }
}

// ── Address (hybrid-kem) ──────────────────────────────────────────────────

/// A shielded payment address with a mandatory PQ encapsulation key.
///
/// When `hybrid-kem` is enabled, every public-facing address carries an
/// [`PqEncapsulationKey`] for post-quantum key exchange. Internal paths that
/// don't have the PQ key use [`RawAddress`] instead.
#[cfg(feature = "hybrid-kem")]
#[derive(Clone, Debug)]
pub struct Address {
    raw: RawAddress,
    ek_pq: PqEncapsulationKey,
}

#[cfg(feature = "hybrid-kem")]
impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

#[cfg(feature = "hybrid-kem")]
impl Eq for Address {}

#[cfg(feature = "hybrid-kem")]
impl Address {
    /// Creates an `Address` from a [`RawAddress`] and a [`PqEncapsulationKey`].
    pub(crate) fn from_parts(raw: RawAddress, ek_pq: PqEncapsulationKey) -> Self {
        Address { raw, ek_pq }
    }

    /// Returns the PQ encapsulation key.
    pub fn ek_pq(&self) -> &PqEncapsulationKey {
        &self.ek_pq
    }

    /// Returns the underlying [`RawAddress`].
    pub fn raw(&self) -> &RawAddress {
        &self.raw
    }

    /// Consumes this address, returning the underlying [`RawAddress`].
    pub fn into_raw(self) -> RawAddress {
        self.raw
    }

    /// Returns the [`Diversifier`] for this `Address`.
    pub fn diversifier(&self) -> Diversifier {
        self.raw.diversifier()
    }

    pub(crate) fn g_d(&self) -> NonIdentityPallasPoint {
        self.raw.g_d()
    }

    pub(crate) fn pk_d(&self) -> &DiversifiedTransmissionKey {
        self.raw.pk_d()
    }

    /// Serializes this address to its "raw" encoding as specified in [Zcash Protocol Spec § 5.6.4.2: Orchard Raw Payment Addresses][orchardpaymentaddrencoding]
    ///
    /// Note: this drops the PQ encapsulation key. Use [`to_bytes`](Self::to_bytes) to
    /// serialize the full hybrid address (including `ek_pq`).
    ///
    /// [orchardpaymentaddrencoding]: https://zips.z.cash/protocol/protocol.pdf#orchardpaymentaddrencoding
    pub fn to_raw_address_bytes(&self) -> [u8; 43] {
        self.raw.to_raw_address_bytes()
    }

    /// Serializes the full hybrid payment address: the 43-byte [`RawAddress`] followed
    /// by the 1184-byte PQ encapsulation key, for [`HYBRID_ADDRESS_SIZE`] (1227) bytes.
    ///
    /// This is the encoding a sender needs: it preserves the per-diversifier `ek_pq`
    /// required to construct hybrid outputs (which
    /// [`to_raw_address_bytes`](Self::to_raw_address_bytes) drops).
    pub fn to_bytes(&self) -> [u8; HYBRID_ADDRESS_SIZE] {
        let mut result = [0u8; HYBRID_ADDRESS_SIZE];
        result[..43].copy_from_slice(&self.raw.to_raw_address_bytes());
        result[43..].copy_from_slice(self.ek_pq.as_bytes());
        result
    }

    /// Parses a full hybrid payment address from its [`HYBRID_ADDRESS_SIZE`]-byte
    /// encoding (43-byte [`RawAddress`] ‖ 1184-byte `ek_pq`), as produced by
    /// [`to_bytes`](Self::to_bytes).
    ///
    /// Returns `None` if the `RawAddress` portion is not a valid address. The `ek_pq`
    /// bytes are not validated here; a malformed key surfaces as an error at
    /// encapsulation time.
    pub fn from_bytes(bytes: &[u8; HYBRID_ADDRESS_SIZE]) -> Option<Self> {
        let raw_bytes: &[u8; 43] = bytes[..43].try_into().expect("43-byte prefix");
        let ek_pq_bytes: [u8; PQ_EK_SIZE] = bytes[43..].try_into().expect("ek_pq suffix");
        let raw: RawAddress = Option::from(RawAddress::from_raw_address_bytes(raw_bytes))?;
        Some(Address::from_parts(
            raw,
            PqEncapsulationKey::from_bytes(ek_pq_bytes),
        ))
    }
}

// ── Address (classic, no hybrid-kem) ──────────────────────────────────────

/// A shielded payment address.
///
/// # Examples
///
/// ```
/// use orchard::keys::{SpendingKey, FullViewingKey, Scope};
///
/// let sk = SpendingKey::from_bytes([7; 32]).unwrap();
/// let address = FullViewingKey::from(&sk).address_at(0u32, Scope::External);
/// ```
#[cfg(not(feature = "hybrid-kem"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    d: Diversifier,
    pk_d: DiversifiedTransmissionKey,
}

/// When `hybrid-kem` is disabled, [`RawAddress`] is a type alias for [`Address`].
#[cfg(not(feature = "hybrid-kem"))]
pub type RawAddress = Address;

#[cfg(not(feature = "hybrid-kem"))]
impl Address {
    pub(crate) fn from_parts(d: Diversifier, pk_d: DiversifiedTransmissionKey) -> Self {
        // We assume here that pk_d is correctly-derived from d. We ensure this for
        // internal APIs. For parsing from raw byte encodings, we assume that users aren't
        // modifying internals of encoded address formats. If they do, that can result in
        // lost funds, but we can't defend against that from here.
        Address { d, pk_d }
    }

    /// Returns the [`Diversifier`] for this `Address`.
    pub fn diversifier(&self) -> Diversifier {
        self.d
    }

    /// Returns the diversified base point for this address.
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn g_d(&self) -> NonIdentityPallasPoint {
        diversify_hash(self.d.as_array())
    }

    /// Returns the diversified transmission key for this address.
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn pk_d(&self) -> &DiversifiedTransmissionKey {
        &self.pk_d
    }

    /// Serializes this address to its "raw" encoding as specified in [Zcash Protocol Spec § 5.6.4.2: Orchard Raw Payment Addresses][orchardpaymentaddrencoding]
    ///
    /// [orchardpaymentaddrencoding]: https://zips.z.cash/protocol/protocol.pdf#orchardpaymentaddrencoding
    pub fn to_raw_address_bytes(&self) -> [u8; 43] {
        let mut result = [0u8; 43];
        result[..11].copy_from_slice(self.d.as_array());
        result[11..].copy_from_slice(&self.pk_d.to_bytes());
        result
    }

    /// Parse an address from its "raw" encoding as specified in [Zcash Protocol Spec § 5.6.4.2: Orchard Raw Payment Addresses][orchardpaymentaddrencoding]
    ///
    /// [orchardpaymentaddrencoding]: https://zips.z.cash/protocol/protocol.pdf#orchardpaymentaddrencoding
    pub fn from_raw_address_bytes(bytes: &[u8; 43]) -> CtOption<Self> {
        DiversifiedTransmissionKey::from_bytes(bytes[11..43].try_into().unwrap()).map(|pk_d| {
            let d = Diversifier::from_bytes(bytes[..11].try_into().unwrap());
            Self::from_parts(d, pk_d)
        })
    }

    /// Returns a reference to self (for API compatibility with hybrid mode).
    pub fn raw(&self) -> &Address {
        self
    }

    /// Returns self (for API compatibility with hybrid mode).
    pub fn into_raw(self) -> Address {
        self
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use crate::keys::{
        testing::{arb_diversifier_index, arb_spending_key},
        FullViewingKey, Scope,
    };

    use super::Address;

    prop_compose! {
        /// Generates an arbitrary payment address.
        pub(crate) fn arb_address()(sk in arb_spending_key(), j in arb_diversifier_index()) -> Address {
            let fvk = FullViewingKey::from(&sk);
            fvk.address_at(j, Scope::External)
        }
    }
}

#[cfg(all(test, feature = "hybrid-kem"))]
mod hybrid_address_tests {
    use super::RawAddress;
    use crate::keys::{FullViewingKey, Scope, SpendingKey};

    #[test]
    fn address_accessors_and_raw_roundtrip() {
        let fvk = FullViewingKey::from(&SpendingKey::from_bytes([7; 32]).unwrap());
        let addr = fvk.address_at(0u32, Scope::External);

        // The full Address carries a PQ encapsulation key.
        let _ = addr.ek_pq();

        // raw() exposes the 43-byte on-chain form; accessors agree with it.
        let raw = *addr.raw();
        assert_eq!(addr.diversifier(), raw.diversifier());
        assert_eq!(addr.to_raw_address_bytes(), raw.to_raw_address_bytes());

        // RawAddress serialization round-trips.
        let bytes = raw.to_raw_address_bytes();
        let raw2: RawAddress =
            Option::from(RawAddress::from_raw_address_bytes(&bytes)).expect("valid raw address");
        assert_eq!(raw, raw2);

        // into_raw consumes the Address into its RawAddress.
        assert_eq!(addr.into_raw(), raw);
    }

    #[test]
    fn address_full_bytes_roundtrip() {
        let fvk = FullViewingKey::from(&SpendingKey::from_bytes([7; 32]).unwrap());
        let addr = fvk.address_at(0u32, Scope::External);

        let bytes = addr.to_bytes();
        // First 43 bytes are the raw address; the remaining 1184 are ek_pq.
        assert_eq!(&bytes[..43], &addr.to_raw_address_bytes()[..]);
        assert_eq!(&bytes[43..], addr.ek_pq().as_bytes());

        // Full round-trip preserves both the raw address and the PQ key.
        let addr2 = super::Address::from_bytes(&bytes).expect("valid address");
        assert_eq!(addr2, addr);
        assert_eq!(addr2.ek_pq().as_bytes(), addr.ek_pq().as_bytes());
    }
}
