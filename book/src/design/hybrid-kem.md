# Hybrid post-quantum key encapsulation

When the `hybrid-kem` feature is enabled, Orchard note encryption uses a hybrid key
encapsulation mechanism (KEM) that combines classical elliptic-curve Diffie-Hellman (ECDH)
with ML-KEM-768, a NIST-standardized post-quantum KEM based on the Module Learning With
Errors (MLWE) problem.

This provides defense against "harvest now, decrypt later" attacks, where an adversary
records encrypted traffic today and decrypts it later using a cryptographically relevant
quantum computer.

## Key derivation

The post-quantum keypair is derived deterministically from the spending key:

1. The spending key $\mathsf{sk}$ is expanded via BLAKE2b into a 64-byte PQ seed.
2. The PQ seed is split into two 32-byte halves ($d$, $z$), which are used as the
   randomness for ML-KEM-768 `KeyGen`.
3. This produces an **encapsulation key** $\mathsf{ek\_{pq}}$ (1184 bytes) and a
   **decapsulation key** $\mathsf{dk\_{pq}}$ (2400 bytes).

Because derivation is deterministic, the same spending key always produces the same
PQ keypair. The encapsulation key is stored in the full viewing key alongside the
existing Orchard key components.

## Address structure

Orchard addresses have two internal representations:

- **`RawAddress`** (43 bytes): the diversifier $d$ and diversified transmission key
  $\mathsf{pk\_d}$. This is the on-chain form used in note commitments, nullifier
  derivation, and encrypted note plaintexts. It is identical to the classical Orchard
  address format.

- **`Address`**: wraps a `RawAddress` together with a mandatory $\mathsf{ek\_{pq}}$.
  This is what payment addresses encode and what the transaction builder requires.

The split enforces at the type level that the builder always has the PQ encapsulation
key available when constructing outputs, while internal paths that reconstruct notes
from chain data (decryption, PCZT parsing) use only the 43-byte `RawAddress`.

When the `hybrid-kem` feature is not enabled, `RawAddress` is a type alias for `Address`
and the two types are interchangeable.

## Address encoding

A serialized payment address contains both the 43-byte `RawAddress` and the 1184-byte
$\mathsf{ek\_{pq}}$, for a total of 1227 bytes. This is encoded using Base58 for
consistency with existing Dash address formats, producing an address string of
approximately 1678 characters.

### Encoding considerations

Several encodings were evaluated:

| Encoding | Text length | QR mode        | QR data (bits) |
|----------|-------------|----------------|----------------|
| Raw binary | —         | Byte           | 9,816          |
| Base45   | ~1,841      | Alphanumeric   | ~10,131        |
| Base58   | ~1,678      | Byte           | ~13,424        |
| Base64url| ~1,636      | Byte           | ~13,088        |
| Base85   | ~1,534      | Byte           | ~12,272        |
| Bech32m  | ~1,975      | Byte           | ~15,800        |

Base58 was chosen because:

- It is already used for Dash addresses, so wallet developers and users are familiar
  with the format.
- It avoids visually ambiguous characters (`0`/`O`, `l`/`I`), improving manual
  transcription.
- It is URL-safe without percent-encoding.

For QR codes, wallets should encode the raw binary address (1227 bytes) directly in
byte mode rather than encoding the Base58 text. This produces the smallest possible
QR code. The QR payload and the text representation serve different purposes and do
not need to use the same encoding.

## Encryption flow

When constructing a note for a recipient:

1. The sender obtains the recipient's `Address`, which contains both `RawAddress` and
   $\mathsf{ek\_{pq}}$.
2. Classical ECDH is performed using $\mathsf{pk\_d}$ from `RawAddress`, producing a
   shared secret $\mathsf{ss\_{ecdh}}$.
3. ML-KEM-768 encapsulation is performed using $\mathsf{ek\_{pq}}$, producing a
   ciphertext $\mathsf{ct\_{pq}}$ (1088 bytes) and a shared secret
   $\mathsf{ss\_{pq}}$.
4. Both shared secrets are combined via a hybrid KDF (BLAKE2b) to produce the
   final symmetric key.

Decryption reverses this: the recipient uses their classical private key for ECDH and
$\mathsf{dk\_{pq}}$ for ML-KEM decapsulation, then combines both shared secrets
through the same KDF.

## Security properties

- **Hybrid security**: the scheme is secure as long as *either* ECDH on Pallas *or*
  ML-KEM-768 remains unbroken. A quantum computer that breaks ECDH but not MLWE
  cannot recover the symmetric key, and vice versa.
- **Deterministic encapsulation**: the ML-KEM randomness is derived from the note's
  $\mathsf{rseed}$ and $\rho$, ensuring that encryption is reproducible for proof
  generation without requiring additional random state.
- **Implicit rejection**: ML-KEM-768 uses implicit rejection — invalid ciphertexts
  produce a pseudorandom shared secret rather than an error, preventing chosen-ciphertext
  attacks.

## Size impact

The hybrid KEM adds overhead to transactions:

| Component            | Classical | Hybrid    | Increase    |
|----------------------|-----------|-----------|-------------|
| Payment address      | 43 bytes  | 1,227 bytes | +1,184 bytes |
| Ciphertext per action| 0 bytes   | 1,088 bytes | +1,088 bytes |

The on-chain note format (`RawAddress`, note commitment, nullifier) is unchanged.
The additional ciphertext $\mathsf{ct\_{pq}}$ is included alongside each action's
encrypted output.
