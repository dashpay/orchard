//! Hybrid KEM (ML-KEM-768 + ECDH) for quantum-resistant note encryption.
//!
//! This module provides the post-quantum key encapsulation mechanism used alongside
//! classical ECDH to derive symmetric keys for note encryption. The hybrid approach
//! ensures that breaking either mechanism alone is insufficient to decrypt notes.
//!
//! The design follows the X-Wing security proof approach: the KDF binds all context
//! (both shared secrets, the PQ ciphertext, and the ephemeral public key) into the
//! derived key.

use blake2b_simd::Params;
use ml_kem::kem::Decapsulate;
use ml_kem::{array::Array, B32, EncapsulateDeterministic, EncodedSizeUser, KemCore, MlKem768};

/// Size of an ML-KEM-768 encapsulation key (public key).
pub const PQ_EK_SIZE: usize = 1184;

/// Size of an ML-KEM-768 decapsulation key (private key).
pub const PQ_DK_SIZE: usize = 2400;

/// Size of an ML-KEM-768 ciphertext.
pub const PQ_CT_SIZE: usize = 1088;

/// Size of an ML-KEM-768 shared secret.
pub const PQ_SS_SIZE: usize = 32;

/// BLAKE2b personalization for the hybrid KDF.
pub(crate) const HYBRID_KDF_PERSONALIZATION: &[u8; 16] = b"DashPltfrm_HyKDF";

/// BLAKE2b personalization for deriving PQ encapsulation randomness.
const PQ_ENCAPS_RAND_PERSONALIZATION: &[u8; 16] = b"DashPQ_EncapRand";

/// BLAKE2b personalization for deriving the PQ seed from a spending key.
const PQ_KEY_DERIVE_PERSONALIZATION: &[u8; 16] = b"DashPQ_KeyDerive";

/// Derives a 64-byte PQ seed from a 32-byte spending key.
///
/// The seed is split into two 32-byte halves: `d = seed[..32]` and `z = seed[32..]`,
/// which are used as the deterministic inputs to ML-KEM-768 KeyGen.
pub fn derive_pq_seed(sk: &[u8; 32]) -> [u8; 64] {
    let hash = Params::new()
        .hash_length(64)
        .personal(PQ_KEY_DERIVE_PERSONALIZATION)
        .hash(sk);
    let mut seed = [0u8; 64];
    seed.copy_from_slice(hash.as_bytes());
    seed
}

/// Generates a deterministic ML-KEM-768 keypair from a 64-byte seed.
///
/// Returns `(encapsulation_key, decapsulation_key)` where:
/// - `encapsulation_key` is the 1184-byte public key
/// - `decapsulation_key` is the 2400-byte private key
///
/// The seed bytes `[..32]` are used as `d` and `[32..]` as `z` for KeyGen.
pub fn generate_pq_keypair(seed: &[u8; 64]) -> ([u8; PQ_EK_SIZE], [u8; PQ_DK_SIZE]) {
    let d: [u8; 32] = seed[..32].try_into().expect("seed slice is 32 bytes");
    let z: [u8; 32] = seed[32..].try_into().expect("seed slice is 32 bytes");

    let (dk, ek) = MlKem768::generate_deterministic(&d.into(), &z.into());

    let ek_bytes = ek.as_bytes();
    let dk_bytes = dk.as_bytes();

    let mut ek_out = [0u8; PQ_EK_SIZE];
    let mut dk_out = [0u8; PQ_DK_SIZE];
    ek_out.copy_from_slice(ek_bytes.as_slice());
    dk_out.copy_from_slice(dk_bytes.as_slice());

    (ek_out, dk_out)
}

/// Derives deterministic randomness for ML-KEM encapsulation.
///
/// This avoids needing an RNG in `ka_agree_enc`, consistent with Orchard's philosophy
/// of deriving all randomness from `rseed`.
///
/// The derivation binds `rseed`, `rho`, and the recipient's `ek_pq` so that the
/// randomness is specific to a particular note and recipient key pair.
pub fn derive_pq_encaps_randomness(
    rseed: &[u8; 32],
    rho: &[u8; 32],
    ek_pq: &[u8; PQ_EK_SIZE],
) -> [u8; 32] {
    let hash = Params::new()
        .hash_length(32)
        .personal(PQ_ENCAPS_RAND_PERSONALIZATION)
        .to_state()
        .update(rseed)
        .update(rho)
        .update(ek_pq)
        .finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// Errors that can occur during ML-KEM operations.
#[derive(Debug)]
pub enum PqError {
    /// ML-KEM encapsulation failed.
    EncapsulationFailed,
    /// ML-KEM decapsulation failed.
    DecapsulationFailed,
}

/// Performs deterministic ML-KEM-768 encapsulation.
///
/// Returns `(ciphertext, shared_secret)` or an error if the encapsulation key is invalid.
pub fn encapsulate_deterministic(
    ek_bytes: &[u8; PQ_EK_SIZE],
    randomness: &[u8; 32],
) -> Result<([u8; PQ_CT_SIZE], [u8; PQ_SS_SIZE]), PqError> {
    // Array sizes are enforced by the type system — no runtime failure possible here.
    let ek = <MlKem768 as KemCore>::EncapsulationKey::from_bytes(
        &Array::from(*ek_bytes),
    );

    let m: B32 = Array::from(*randomness);
    let (ct, ss) = ek
        .encapsulate_deterministic(&m)
        .map_err(|_| PqError::EncapsulationFailed)?;

    let mut ct_out = [0u8; PQ_CT_SIZE];
    let mut ss_out = [0u8; PQ_SS_SIZE];
    ct_out.copy_from_slice(ct.as_slice());
    ss_out.copy_from_slice(ss.as_slice());

    Ok((ct_out, ss_out))
}

/// Performs ML-KEM-768 decapsulation.
///
/// Returns the shared secret. ML-KEM decapsulation is designed to always succeed
/// (implicit rejection returns a pseudorandom value for invalid ciphertexts),
/// so this function is infallible.
pub fn decapsulate(dk_bytes: &[u8; PQ_DK_SIZE], ct_bytes: &[u8; PQ_CT_SIZE]) -> [u8; PQ_SS_SIZE] {
    let dk = <MlKem768 as KemCore>::DecapsulationKey::from_bytes(
        &Array::from(*dk_bytes),
    );

    let ct = ml_kem::Ciphertext::<MlKem768>::try_from(ct_bytes.as_slice())
        .expect("ct_bytes length matches PQ_CT_SIZE");

    // ML-KEM decapsulation with implicit rejection: invalid ciphertexts produce
    // a pseudorandom shared secret rather than failing. The unwrap is safe.
    let ss = dk.decapsulate(&ct).expect("ML-KEM decapsulation always succeeds (implicit rejection)");

    let mut ss_out = [0u8; PQ_SS_SIZE];
    ss_out.copy_from_slice(ss.as_slice());
    ss_out
}

/// Hybrid KDF combining ECDH and ML-KEM shared secrets.
///
/// Produces a 32-byte symmetric key from:
/// - `ss_ecdh`: the 32-byte ECDH shared secret (Pallas point x-coordinate)
/// - `ss_pq`: the 32-byte ML-KEM shared secret
/// - `ct_pq`: the 1088-byte ML-KEM ciphertext
/// - `epk`: the 32-byte ephemeral public key (ECDH)
///
/// Including `ct_pq` follows the X-Wing security proof approach, binding the PQ
/// ciphertext to the derived key. This prevents transcript binding issues even if
/// ML-KEM's IND-CCA2 guarantee is weakened in future.
pub fn hybrid_kdf(
    ss_ecdh: &[u8; 32],
    ss_pq: &[u8; 32],
    ct_pq: &[u8; PQ_CT_SIZE],
    epk: &[u8; 32],
) -> [u8; 32] {
    let hash = Params::new()
        .hash_length(32)
        .personal(HYBRID_KDF_PERSONALIZATION)
        .to_state()
        .update(ss_ecdh)
        .update(ss_pq)
        .update(ct_pq)
        .update(epk)
        .finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_keypair_generation() {
        let sk = [42u8; 32];
        let seed = derive_pq_seed(&sk);

        let (ek1, dk1) = generate_pq_keypair(&seed);
        let (ek2, dk2) = generate_pq_keypair(&seed);

        assert_eq!(ek1, ek2, "keypair generation must be deterministic");
        assert_eq!(dk1, dk2, "keypair generation must be deterministic");
    }

    #[test]
    fn encaps_decaps_round_trip() {
        let sk = [7u8; 32];
        let seed = derive_pq_seed(&sk);
        let (ek, dk) = generate_pq_keypair(&seed);

        let randomness = [99u8; 32];
        let (ct, ss_enc) = encapsulate_deterministic(&ek, &randomness)
            .expect("encapsulation should succeed");
        let ss_dec = decapsulate(&dk, &ct);

        assert_eq!(
            ss_enc, ss_dec,
            "encapsulation and decapsulation must produce the same shared secret"
        );
    }

    #[test]
    fn deterministic_encapsulation() {
        let sk = [7u8; 32];
        let seed = derive_pq_seed(&sk);
        let (ek, _dk) = generate_pq_keypair(&seed);

        let randomness = [99u8; 32];
        let (ct1, ss1) = encapsulate_deterministic(&ek, &randomness)
            .expect("encapsulation should succeed");
        let (ct2, ss2) = encapsulate_deterministic(&ek, &randomness)
            .expect("encapsulation should succeed");

        assert_eq!(ct1, ct2, "encapsulation must be deterministic");
        assert_eq!(ss1, ss2, "shared secret must be deterministic");
    }

    #[test]
    fn hybrid_kdf_domain_separation() {
        let ss_ecdh = [1u8; 32];
        let ss_pq = [2u8; 32];
        let ct_pq = [3u8; PQ_CT_SIZE];
        let epk = [4u8; 32];

        let key1 = hybrid_kdf(&ss_ecdh, &ss_pq, &ct_pq, &epk);

        // Swapping ECDH and PQ shared secrets must produce a different key
        let key2 = hybrid_kdf(&ss_pq, &ss_ecdh, &ct_pq, &epk);
        assert_ne!(key1, key2, "KDF must be order-dependent on inputs");

        // Different EPK must produce a different key
        let epk2 = [5u8; 32];
        let key3 = hybrid_kdf(&ss_ecdh, &ss_pq, &ct_pq, &epk2);
        assert_ne!(key1, key3, "KDF must bind the ephemeral public key");

        // Different ct_pq must produce a different key
        let ct_pq2 = [4u8; PQ_CT_SIZE];
        let key4 = hybrid_kdf(&ss_ecdh, &ss_pq, &ct_pq2, &epk);
        assert_ne!(key1, key4, "KDF must bind the PQ ciphertext");
    }

    #[test]
    fn pq_encaps_randomness_is_deterministic() {
        let rseed = [10u8; 32];
        let rho = [20u8; 32];
        let ek_pq = [30u8; PQ_EK_SIZE];

        let r1 = derive_pq_encaps_randomness(&rseed, &rho, &ek_pq);
        let r2 = derive_pq_encaps_randomness(&rseed, &rho, &ek_pq);
        assert_eq!(r1, r2, "encaps randomness derivation must be deterministic");
    }

    #[test]
    fn pq_encaps_randomness_varies_with_inputs() {
        let rseed = [10u8; 32];
        let rho1 = [20u8; 32];
        let rho2 = [21u8; 32];
        let ek_pq = [30u8; PQ_EK_SIZE];

        let r1 = derive_pq_encaps_randomness(&rseed, &rho1, &ek_pq);
        let r2 = derive_pq_encaps_randomness(&rseed, &rho2, &ek_pq);
        assert_ne!(
            r1, r2,
            "different rho values must produce different randomness"
        );

        // Different ek_pq must produce different randomness
        let ek_pq2 = [31u8; PQ_EK_SIZE];
        let r3 = derive_pq_encaps_randomness(&rseed, &rho1, &ek_pq);
        let r4 = derive_pq_encaps_randomness(&rseed, &rho1, &ek_pq2);
        assert_ne!(
            r3, r4,
            "different ek_pq values must produce different randomness"
        );
    }

    #[test]
    fn full_hybrid_encrypt_decrypt_round_trip() {
        // Simulate a full hybrid encryption/decryption flow
        let spending_key = [42u8; 32];
        let pq_seed = derive_pq_seed(&spending_key);
        let (ek, dk) = generate_pq_keypair(&pq_seed);

        // Encryption side: derive deterministic randomness and encapsulate
        let rseed = [100u8; 32];
        let rho = [200u8; 32];
        let pq_randomness = derive_pq_encaps_randomness(&rseed, &rho, &ek);
        let (ct_pq, ss_pq_enc) = encapsulate_deterministic(&ek, &pq_randomness)
            .expect("encapsulation should succeed");

        // Simulate ECDH shared secret (in real code this comes from ka_orchard)
        let ss_ecdh = [55u8; 32];
        let epk = [66u8; 32];

        // Derive encryption key
        let key_enc = hybrid_kdf(&ss_ecdh, &ss_pq_enc, &ct_pq, &epk);

        // Decryption side: decapsulate
        let ss_pq_dec = decapsulate(&dk, &ct_pq);
        assert_eq!(ss_pq_enc, ss_pq_dec, "PQ shared secrets must match");

        // Derive decryption key (same ECDH shared secret from the other side)
        let key_dec = hybrid_kdf(&ss_ecdh, &ss_pq_dec, &ct_pq, &epk);

        assert_eq!(
            key_enc, key_dec,
            "hybrid KDF must produce the same key for encryption and decryption"
        );
    }
}
