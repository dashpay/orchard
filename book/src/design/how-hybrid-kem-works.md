# How the hybrid KEM cryptography works

This chapter explains the cryptographic ideas behind Orchard's `hybrid-kem` mode
from first principles. For the exact construction — byte layouts, sizes, APIs, and
trade-offs — see [Hybrid post-quantum key encapsulation](./hybrid-kem.md).

## The problem: "harvest now, decrypt later"

Classical Orchard note encryption derives its symmetric keys from elliptic-curve
Diffie–Hellman (ECDH) on the Pallas curve. ECDH is secure against today's computers,
but a large enough quantum computer running Shor's algorithm would break it. An
adversary doesn't need that computer today — they can **record encrypted
transactions now and decrypt them later**, once quantum hardware exists. Because
shielded-pool privacy is meant to last for decades, "secure for now" is not enough.

The hybrid KEM adds a second, quantum-resistant layer so that breaking the classical
layer alone reveals nothing.

## Building block 1: key agreement (ECDH)

In classical Orchard, the sender picks a random *ephemeral* secret and publishes the
matching ephemeral public key (`epk`) with the transaction. Both sides can then
compute the same value:

- the **sender** combines its ephemeral secret with the recipient's public key
  (`pk_d`);
- the **recipient** combines its private viewing key (`ivk`) with `epk`.

The result is a shared secret `ss_ecdh` known only to sender and recipient — the
classical half of the scheme.

## Building block 2: a KEM

A **Key Encapsulation Mechanism (KEM)** has a different shape from key agreement:

- `Encapsulate(public_key) -> (ciphertext, shared_secret)` — the sender generates a
  fresh shared secret *and* a ciphertext that lets the private-key holder recover it.
- `Decapsulate(private_key, ciphertext) -> shared_secret` — the recipient recovers
  the same shared secret.

Where ECDH needs both parties to contribute a public value, a KEM lets the sender
unilaterally establish a secret using only the recipient's public key. KEMs are the
standard interface for post-quantum key establishment.

## Building block 3: ML-KEM-768

[ML-KEM](https://csrc.nist.gov/pubs/fips/203/final) (FIPS 203, formerly
CRYSTALS-Kyber) is a NIST-standardized KEM. Its security rests on the **Module
Learning With Errors (MLWE)** problem: recovering a secret from deliberately *noisy*
systems of linear equations over a polynomial ring. Lattice problems like MLWE are
believed hard for both classical and quantum computers — there is no known efficient
quantum algorithm against them — which is what makes ML-KEM *post-quantum*. The
`768` parameter set targets roughly the AES-192 security level.

Two properties matter for this design:

- **Implicit rejection (IND-CCA2):** an invalid ciphertext does not produce an
  error; it yields a *pseudorandom* shared secret. This removes decryption-failure
  oracles that chosen-ciphertext attacks rely on.
- **Large objects:** encapsulation keys are 1184 bytes and ciphertexts are 1088
  bytes. This is the source of the scheme's size overhead.

## Putting them together: a hybrid KEM

The core idea is to run **both** key agreements and mix their outputs so the final
key is secure as long as **either** mechanism is unbroken:

1. The sender performs ECDH to get `ss_ecdh`.
2. The sender performs ML-KEM encapsulation against the recipient's encapsulation
   key, producing a ciphertext `ct_pq` and a second shared secret `ss_pq`.
3. Both secrets — plus context — are fed through a key-derivation function (KDF):

   ```text
   K = KDF(ss_ecdh || ss_pq || ct_pq || epk)
   ```

4. `K` is the symmetric key used with the usual AEAD to encrypt the note.

The recipient repeats the same steps: ECDH with `epk` to recover `ss_ecdh`, ML-KEM
*decapsulation* of `ct_pq` to recover `ss_pq`, then the same KDF.

**Why this is secure if either half holds.** Suppose a quantum adversary breaks ECDH
and learns `ss_ecdh`. They still don't know `ss_pq` (ML-KEM is unbroken), so they
cannot compute `K`. Conversely, if ML-KEM were broken but ECDH still holds,
`ss_ecdh` stays secret and so does `K`. An attacker must defeat **both** primitives
to recover the key — a NIST post-quantum standard *and* the elliptic-curve discrete
log problem at the same time.

**Why the KDF binds `ct_pq` and `epk`, not just the two secrets.** This follows the
[X-Wing](https://eprint.iacr.org/2024/039) hybrid-KEM design. Mixing in the full
"transcript" (the ciphertext and ephemeral public key) makes the combiner robust
even if ML-KEM's IND-CCA2 guarantee were weakened in the future: an attacker cannot
take a shared secret and pair it with a *different* ciphertext to derive the same
key.

## Determinism: no extra randomness

Orchard derives all of a note's randomness from its seed (`rseed`) and position
(`rho`) so that proof generation is reproducible. Hybrid mode preserves this: the
randomness used for ML-KEM encapsulation is itself derived deterministically from
`rseed`, `rho`, and the recipient's encapsulation key. Encryption therefore remains
a pure function of the note — reproducible during proving, with no extra random
state to carry — while still being unique per note.

## Privacy: per-diversifier keys and the hint

Each diversified address gets its **own** ML-KEM keypair, derived on demand from a
master seed and the diversifier. This preserves *address unlinkability*: two
addresses from the same wallet look no more related than addresses from different
wallets.

That creates a puzzle for the recipient — which per-diversifier key should they use
to decapsulate? A small **diversifier hint** travels with each output to answer it.
The hint is masked with the ECDH shared secret (so only the recipient, or someone
who has broken ECDH, can read it) and is deliberately **not** MAC'd: a quantum
adversary can recover a candidate diversifier but cannot confirm it without running
the full ML-KEM chain, so transaction-graph metadata stays hidden. Integrity instead
comes from the **bundle (txid) commitment**, which covers the hint so it cannot be
tampered with. See the
[diversifier hint](./hybrid-kem.md#diversifier-hint) and
[security properties](./hybrid-kem.md#security-properties) sections for details.

## What an adversary can and can't do

- **Classical adversary:** learns nothing — both layers are secure.
- **Quantum adversary** (breaks ECDH, not ML-KEM): cannot read note contents (that
  needs `ss_pq`), and cannot identify which transactions belong to a wallet without
  performing a full ML-KEM key generation + decapsulation per output. Note contents
  *and* transaction-graph metadata remain hidden.
- **Adversary who breaks both ECDH and ML-KEM:** the scheme offers no protection —
  but that requires simultaneously defeating elliptic-curve discrete log and a NIST
  post-quantum standard.

## Further reading

- [Hybrid post-quantum key encapsulation](./hybrid-kem.md) — the concrete
  construction: derivations, byte layouts, sizes, serialization, scanning cost, and
  trade-offs.
- [FIPS 203: ML-KEM](https://csrc.nist.gov/pubs/fips/203/final).
- [X-Wing: The Hybrid KEM You've Been Looking For](https://eprint.iacr.org/2024/039).
