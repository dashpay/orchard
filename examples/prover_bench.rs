//! Orchard prover benchmark, as used to measure Dash shielded send latency.
//!
//! Builds a bundle with `--actions N` outputs (padded to the minimum of two actions) using
//! the 36-byte Dash memo, proves it, and verifies the proof outside the timed region.
//!
//! ```text
//! cargo run --release --example prover_bench -- --mode cold
//! cargo run --release --example prover_bench -- --mode prove --actions 2 --iters 5
//! cargo run --release --example prover_bench -- --mode seeded --actions 6 --iters 3
//! ```
//!
//! - `cold`: time `ProvingKey::build` and `VerifyingKey::build` (run in a fresh process).
//!   `VerifyingKey::build` runs second, so its timing excludes Rayon thread-pool startup.
//! - `prove`: build the keys, prove one untimed bundle, then time `--iters` proofs.
//! - `seeded`: prove with a seeded RNG and print each proof's BLAKE2b-256 digest, so two
//!   builds of the prover can be checked for byte-identical output. Test-only randomness:
//!   never use a seeded RNG for real proofs.
//!
//! Output is one JSON object per line. Set `RAYON_NUM_THREADS` to vary the thread count.

use std::time::Instant;

use orchard::{
    builder::{Builder, BundleType, InProgress, Unauthorized, Unproven},
    circuit::{Instance, ProvingKey, VerifyingKey},
    keys::{FullViewingKey, Scope, SpendingKey},
    memo::DashMemo,
    value::NoteValue,
    Anchor, Bundle,
};
use rand::{rngs::OsRng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

type UnprovenBundle = Bundle<InProgress<Unproven, Unauthorized>, i64, DashMemo>;

fn arg(name: &str, default: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| default.to_string())
}

fn bundle(outputs: usize, rng: &mut impl RngCore) -> (UnprovenBundle, Vec<Instance>) {
    let sk = SpendingKey::from_bytes([7; 32]).unwrap();
    let recipient = FullViewingKey::from(&sk).address_at(0u32, Scope::External);
    let mut builder =
        Builder::<DashMemo>::new(BundleType::DEFAULT, Anchor::from_bytes([0; 32]).unwrap());
    for _ in 0..outputs {
        builder
            .add_output(None, recipient, NoteValue::from_raw(10), [0; 36])
            .unwrap();
    }
    let bundle: UnprovenBundle = builder.build(&mut *rng).unwrap().unwrap().0;
    let instances = bundle
        .actions()
        .iter()
        .map(|a| a.to_instance(*bundle.flags(), *bundle.anchor()))
        .collect();
    (bundle, instances)
}

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1e3
}

fn main() {
    let mode = arg("--mode", "prove");
    let outputs: usize = arg("--actions", "2").parse().unwrap();
    let iters: usize = arg("--iters", "5").parse().unwrap();
    let threads = std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "default".into());

    match mode.as_str() {
        "cold" => {
            let start = Instant::now();
            let pk = ProvingKey::build();
            let pk_ms = ms(start);
            let start = Instant::now();
            let vk = VerifyingKey::build();
            let vk_ms = ms(start);
            drop((pk, vk));
            println!(
                r#"{{"mode":"cold","threads":"{threads}","pk_build_ms":{pk_ms:.1},"vk_build_ms":{vk_ms:.1}}}"#
            );
        }
        "prove" => {
            let (pk, vk) = (ProvingKey::build(), VerifyingKey::build());
            let mut rng = OsRng;
            let (warmup, instances) = bundle(outputs, &mut rng);
            warmup
                .authorization()
                .create_proof(&pk, &instances, &mut rng)
                .unwrap();
            for iter in 0..iters {
                let (bundle, instances) = bundle(outputs, &mut rng);
                let start = Instant::now();
                let proof = bundle
                    .authorization()
                    .create_proof(&pk, &instances, &mut rng)
                    .unwrap();
                let prove_ms = ms(start);
                assert!(
                    proof.verify(&vk, &instances).is_ok(),
                    "proof does not verify"
                );
                println!(
                    r#"{{"mode":"prove","threads":"{threads}","actions":{},"iter":{iter},"prove_ms":{prove_ms:.1}}}"#,
                    bundle.actions().len()
                );
            }
        }
        "seeded" => {
            let (pk, vk) = (ProvingKey::build(), VerifyingKey::build());
            for seed in 0..iters as u64 {
                let mut rng = ChaCha20Rng::seed_from_u64(seed);
                let (bundle, instances) = bundle(outputs, &mut rng);
                let proof = bundle
                    .authorization()
                    .create_proof(&pk, &instances, &mut rng)
                    .unwrap();
                assert!(
                    proof.verify(&vk, &instances).is_ok(),
                    "proof does not verify"
                );
                let digest = blake2b_simd::Params::new()
                    .hash_length(32)
                    .hash(proof.as_ref());
                println!(
                    r#"{{"mode":"seeded","actions":{},"seed":{seed},"proof_blake2b":"{}"}}"#,
                    bundle.actions().len(),
                    digest.to_hex()
                );
            }
        }
        other => panic!("unknown --mode {other}"),
    }
}
