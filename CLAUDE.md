# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Orchard is a Rust implementation of the Orchard shielded transaction protocol (Zcash). This is the `dashified` fork which adds generic memo size support via the `MemoSize` trait, allowing both Zcash (512-byte memos) and Dash (36-byte memos) to share the same implementation.

- **MSRV**: 1.70 (pinned in `rust-toolchain.toml`)
- **Edition**: 2021
- **`#![no_std]`** by default, with optional `std` feature

## Build & Test Commands

```bash
cargo test --verbose              # Run all tests
cargo test <test_name>            # Run a single test
cargo build --all-features        # Build with all features
cargo fmt -- --check              # Check formatting
cargo clippy                      # Lint (warnings are errors in CI)
cargo doc --all-features --document-private-items  # Build docs (broken intra-doc links are denied)
cargo build --benches             # Build benchmarks (bitrot check)
mdbook test book/                 # Test book examples
```

## Feature Flags

- **`circuit`** (default): Enables Halo2 zk-SNARK circuit and proof system. Implies `std`.
- **`multicore`** (default): Parallel proving via `halo2_proofs/multicore`.
- **`std`** (default): Standard library support.
- **`test-dependencies`**: Enables PropTest generators.
- **`dev-graph`**: Circuit visualization (requires `image`, `plotters`).
- **`unstable-frost`**: Experimental FROST key aggregation.

No-std builds are tested against `wasm32-wasi` and `thumbv7em-none-eabihf` targets.

## Architecture

The crate implements layered cryptographic transaction construction:

**Transaction Building** (`builder.rs`): Constructs `Bundle`s containing `Action`s (each action spends one note and creates another). `pczt/` handles Partially-Created Zcash Transactions with separate parse/verify/prove/sign/finalize stages.

**Bundle & Action** (`bundle.rs`, `action.rs`): `Bundle<V, A>` holds a collection of actions, value balance, and authorization (proofs + signatures). `Action<A, M>` pairs a spend with an output, parameterized by authorization state `A` and memo type `M`.

**Cryptographic Core**: Key hierarchy flows `SpendingKey` → `SpendValidatingKey` / `FullViewingKey` → `Address` (`keys.rs`). Notes (`note/`) carry value and are committed to a 32-level Merkle tree (`tree.rs`). Note encryption (`note_encryption.rs`) uses ChaCha20-Poly1305 AEAD.

**Circuit** (`circuit.rs`, gated by `circuit` feature): Halo2 zk-SNARK proving spend authorization and note validity. Sub-gadgets in `circuit/commit_ivk.rs` and `circuit/note_commit.rs`.

**Memo Generics** (`memo.rs`): The `MemoSize` trait parameterizes memo-dependent sizes throughout the crate. `ZcashMemo` (512 bytes) and `DashMemo` (36 bytes) are the two implementations. Memo encryption happens outside the ZK circuit, so varying memo size doesn't affect the proof system.

## Key Conventions

- All error enums are `#[non_exhaustive]`.
- Constant-time operations are used throughout (via `subtle` crate) for cryptographic security.
- The `spec` module contains protocol-specific mathematical operations matching the Zcash protocol spec.
- Pre-computed fixed base tables live in `constants/fixed_bases/` (~130KB each).
- `zcash_note_encryption` dependency points to a Dash fork (`dashpay/zcash_note_encryption`) for generic memo support.
