//! Embedded Lagrange interpolation coefficients for the Orchard fixed bases.
//!
//! Fixed-base scalar multiplication interpolates the $x$-coordinate of each window's
//! multiples of the base. By default `halo2_gadgets` derives the interpolation
//! coefficients from the generator (`compute_lagrange_coeffs`) every time a fixed-base
//! multiplication is synthesized into a key, which costs several hundred milliseconds per
//! key build: a scalar multiplication and affine normalization for every multiple in
//! every window plus an interpolation per window, repeated for each fixed-base
//! multiplication in the circuit, and repeated again because building a proving key
//! synthesizes the circuit twice (`keygen_vk` and `keygen_pk`).
//!
//! The coefficients are public, deterministic functions of the generators, so their
//! canonical encodings are embedded here instead (`lagrange_coeffs/*.bin`: for each window
//! in order, `H` little-endian `pallas::Base` representations of 32 bytes each). Decoding
//! them costs well under a millisecond.
//!
//! `compute_lagrange_coeffs` remains the reference derivation:
//! `fixed_bases::tests::lagrange_coeffs_match_default_derivation` checks every fixed base
//! against it. To regenerate the files (only needed if a generator changes), run:
//!
//! ```text
//! cargo test --release --lib -- --ignored regenerate_embedded
//! ```

use alloc::vec::Vec;
use group::ff::PrimeField;
use pasta_curves::pallas;

use super::{H, NUM_WINDOWS, NUM_WINDOWS_SHORT};

/// Size of the canonical encoding of a `pallas::Base` element.
const FIELD_BYTES: usize = 32;

/// The embedded Lagrange coefficients of one fixed base, for one window count.
pub(super) struct LagrangeTable(&'static [u8]);

impl LagrangeTable {
    /// Wraps the embedded encoding of a table with `num_windows` windows. The length is
    /// checked at compile time, since every table is a `static`.
    const fn new(bytes: &'static [u8], num_windows: usize) -> Self {
        assert!(bytes.len() == num_windows * H * FIELD_BYTES);
        LagrangeTable(bytes)
    }

    /// Decodes the coefficients, in the form returned by
    /// `halo2_gadgets::ecc::chip::FixedPoint::lagrange_coeffs`.
    pub(super) fn coeffs(&self) -> Vec<[pallas::Base; H]> {
        self.0
            .chunks_exact(H * FIELD_BYTES)
            .map(|window| {
                let mut coeffs = [pallas::Base::zero(); H];
                for (coeff, repr) in coeffs.iter_mut().zip(window.chunks_exact(FIELD_BYTES)) {
                    *coeff = pallas::Base::from_repr(repr.try_into().unwrap())
                        .expect("embedded Lagrange coefficients are canonical");
                }
                coeffs
            })
            .collect()
    }
}

/// `CommitIvkR`, full-width scalar windows.
pub(super) static COMMIT_IVK_R: LagrangeTable = LagrangeTable::new(
    include_bytes!("lagrange_coeffs/commit_ivk_r.bin"),
    NUM_WINDOWS,
);
/// `NoteCommitR`, full-width scalar windows.
pub(super) static NOTE_COMMIT_R: LagrangeTable = LagrangeTable::new(
    include_bytes!("lagrange_coeffs/note_commit_r.bin"),
    NUM_WINDOWS,
);
/// `ValueCommitR`, full-width scalar windows.
pub(super) static VALUE_COMMIT_R: LagrangeTable = LagrangeTable::new(
    include_bytes!("lagrange_coeffs/value_commit_r.bin"),
    NUM_WINDOWS,
);
/// `SpendAuthG`, full-width windows. Shared by the full-scalar and base-field-element
/// multiplications, which have the same number of windows.
pub(super) static SPEND_AUTH_G: LagrangeTable = LagrangeTable::new(
    include_bytes!("lagrange_coeffs/spend_auth_g.bin"),
    NUM_WINDOWS,
);
/// `NullifierK`, base-field-element windows.
pub(super) static NULLIFIER_K: LagrangeTable = LagrangeTable::new(
    include_bytes!("lagrange_coeffs/nullifier_k.bin"),
    NUM_WINDOWS,
);
/// `ValueCommitV`, short signed scalar windows.
pub(super) static VALUE_COMMIT_V_SHORT: LagrangeTable = LagrangeTable::new(
    include_bytes!("lagrange_coeffs/value_commit_v_short.bin"),
    NUM_WINDOWS_SHORT,
);
/// `SpendAuthG`, short signed scalar windows.
pub(super) static SPEND_AUTH_G_SHORT: LagrangeTable = LagrangeTable::new(
    include_bytes!("lagrange_coeffs/spend_auth_g_short.bin"),
    NUM_WINDOWS_SHORT,
);

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use group::ff::PrimeField;
    use halo2_gadgets::ecc::chip::compute_lagrange_coeffs;

    use super::super::{
        commit_ivk_r, note_commit_r, nullifier_k, spend_auth_g, value_commit_r, value_commit_v,
        NUM_WINDOWS, NUM_WINDOWS_SHORT,
    };

    /// Rewrites the embedded tables from `compute_lagrange_coeffs`. Verification is done by
    /// `fixed_bases::tests::lagrange_coeffs_match_default_derivation`, which always runs.
    #[test]
    #[ignore = "regenerates the embedded Lagrange coefficient tables"]
    fn regenerate_embedded_lagrange_coeffs() {
        for (file, generator, num_windows) in [
            ("commit_ivk_r.bin", commit_ivk_r::generator(), NUM_WINDOWS),
            ("note_commit_r.bin", note_commit_r::generator(), NUM_WINDOWS),
            (
                "value_commit_r.bin",
                value_commit_r::generator(),
                NUM_WINDOWS,
            ),
            ("spend_auth_g.bin", spend_auth_g::generator(), NUM_WINDOWS),
            ("nullifier_k.bin", nullifier_k::generator(), NUM_WINDOWS),
            (
                "value_commit_v_short.bin",
                value_commit_v::generator(),
                NUM_WINDOWS_SHORT,
            ),
            (
                "spend_auth_g_short.bin",
                spend_auth_g::generator(),
                NUM_WINDOWS_SHORT,
            ),
        ] {
            let bytes: Vec<u8> = compute_lagrange_coeffs(generator, num_windows)
                .iter()
                .flatten()
                .flat_map(|coeff| coeff.to_repr())
                .collect();
            std::fs::write(
                std::format!(
                    "{}/src/constants/fixed_bases/lagrange_coeffs/{}",
                    env!("CARGO_MANIFEST_DIR"),
                    file
                ),
                bytes,
            )
            .expect("can write embedded table");
        }
    }
}
