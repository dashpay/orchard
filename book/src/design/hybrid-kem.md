# Hybrid post-quantum key encapsulation

When the `hybrid-kem` feature is enabled, Orchard note encryption uses a hybrid key
encapsulation mechanism (KEM) that combines classical elliptic-curve Diffie-Hellman (ECDH)
with ML-KEM-768, a NIST-standardized post-quantum KEM based on the Module Learning With
Errors (MLWE) problem.

This provides defense against "harvest now, decrypt later" attacks, where an adversary
records encrypted traffic today and decrypts it later using a cryptographically relevant
quantum computer.

## Key derivation

The post-quantum key material is derived deterministically from the spending key using
a two-level scheme:

1. The spending key $\mathsf{sk}$ is expanded via BLAKE2b-512 into a 64-byte master
   PQ seed:
   ```
   pq_seed = BLAKE2b-512("DashPQ_KeyDerive", sk)
   ```

2. For each diversifier $d$, a per-diversifier seed is derived:
   ```
   pq_seed_d = BLAKE2b-512("DashPQ_DivSeed__", pq_seed || d)
   ```

3. The per-diversifier seed is split into two 32-byte halves ($d'$, $z$), which are
   used as the deterministic inputs to ML-KEM-768 `KeyGen`, producing a per-diversifier
   **encapsulation key** $\mathsf{ek\_{pq\_d}}$ (1184 bytes) and **decapsulation key**
   $\mathsf{dk\_{pq\_d}}$ (2400 bytes).

The master `pq_seed` is stored in the full viewing key (and propagated to the incoming
viewing key) instead of pre-computed keypairs. Each diversified address derives its own
unique PQ keypair on demand.

### Per-diversifier PQ keys

In the original single-keypair design, all diversified addresses from the same spending
key shared the same 1184-byte $\mathsf{ek\_{pq}}$. This broke address unlinkability:
two colluding senders could compare addresses and see the same $\mathsf{ek\_{pq}}$,
linking them to the same wallet.

With per-diversifier keys, each address carries a unique $\mathsf{ek\_{pq\_d}}$ derived
from the master seed and the diversifier. Two addresses from the same wallet are
computationally indistinguishable from addresses belonging to different wallets.

## Address structure

Orchard addresses have two internal representations:

- **`RawAddress`** (43 bytes): the diversifier $d$ and diversified transmission key
  $\mathsf{pk\_d}$. This is the on-chain form used in note commitments, nullifier
  derivation, and encrypted note plaintexts. It is identical to the classical Orchard
  address format.

- **`Address`**: wraps a `RawAddress` together with a mandatory $\mathsf{ek\_{pq\_d}}$.
  This is what payment addresses encode and what the transaction builder requires.
  The $\mathsf{ek\_{pq\_d}}$ is now per-diversifier.

The split enforces at the type level that the builder always has the PQ encapsulation
key available when constructing outputs, while internal paths that reconstruct notes
from chain data (decryption, PCZT parsing) use only the 43-byte `RawAddress`.

When the `hybrid-kem` feature is not enabled, `RawAddress` is a type alias for `Address`
and the two types are interchangeable.

## Address encoding

A serialized payment address contains both the 43-byte `RawAddress` and the 1184-byte
$\mathsf{ek\_{pq\_d}}$, for a total of 1227 bytes. This is encoded using Base58 for
consistency with existing Dash address formats, producing an address string of
approximately 1678 characters.

### Encoding considerations

Several encodings were evaluated:

| Encoding | Text length | QR mode        | QR data (bits) |
|----------|-------------|----------------|----------------|
| Raw binary | ---       | Byte           | 9,816          |
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
   the per-diversifier $\mathsf{ek\_{pq\_d}}$.
2. Classical ECDH is performed using $\mathsf{pk\_d}$ from `RawAddress`, producing a
   shared secret $\mathsf{ss\_{ecdh}}$.
3. ML-KEM-768 encapsulation is performed using $\mathsf{ek\_{pq\_d}}$, producing a
   ciphertext $\mathsf{ct\_{pq}}$ (1088 bytes) and a shared secret
   $\mathsf{ss\_{pq}}$.
4. A **diversifier hint** is computed:
   ```
   mask = BLAKE2b-256("DashPQ_DivHint__", ss_ecdh_bytes || epk_bytes)[..11]
   hint = diversifier XOR mask
   ```
   This 11-byte hint is transmitted on-chain alongside the PQ ciphertext.
5. Both shared secrets are combined via a hybrid KDF (BLAKE2b) to produce the
   final symmetric key.

## Diversifier hint

The diversifier hint is an 11-byte value included with each action that allows the
recipient to recover which diversifier was used, and thus which per-diversifier
decapsulation key to derive.

The hint is encrypted using a mask derived from the ECDH shared secret and the
ephemeral public key. Since XOR is self-inverse, encryption and decryption are the
same operation.

**No MAC is included on the hint.** This is a deliberate design choice for quantum
privacy: a quantum adversary who breaks ECDH can recover the diversifier from the
hint, but cannot verify whether it corresponds to a valid diversifier without
performing the full ML-KEM KeyGen + decapsulation + AEAD chain. The AEAD tag on
the encrypted note implicitly authenticates everything, including the hint.

## Scanning flow

For each on-chain transaction, the recipient performs:

1. $\mathsf{ss\_{ecdh}} = \mathsf{ivk} \cdot \mathsf{epk}$ (ECDH, same as today)
2. Decrypt the hint to recover the candidate diversifier:
   $d = \text{hint} \oplus \text{BLAKE2b}(\mathsf{ss\_{ecdh}} \| \mathsf{epk})[..11]$
3. Derive the per-diversifier seed:
   $\mathsf{pq\_seed\_d} = \text{BLAKE2b}(\mathsf{pq\_seed} \| d)$
4. Generate the per-diversifier keypair:
   $(\mathsf{ek\_{pq\_d}}, \mathsf{dk\_{pq\_d}}) = \text{ML-KEM-768.KeyGen}(\mathsf{pq\_seed\_d})$
   (approximately 1ms)
5. Decapsulate:
   $\mathsf{ss\_{pq}} = \text{ML-KEM.Decapsulate}(\mathsf{dk\_{pq\_d}}, \mathsf{ct\_{pq}})$
6. Derive the symmetric key via hybrid KDF:
   $k = \text{BLAKE2b}(\mathsf{ss\_{ecdh}} \| \mathsf{ss\_{pq}} \| \mathsf{ct\_{pq}} \| \mathsf{epk})$
7. Attempt AEAD decryption. If the tag validates, this note belongs to us.

### Performance

The ML-KEM-768 KeyGen in step 4 is the bottleneck, taking approximately 1ms per
transaction scanned. This is 3--6x slower than classical scanning. The trade-off
is full quantum privacy: without a MAC on the hint, a quantum adversary cannot
identify which transactions belong to a wallet without attempting the full chain.

## Security properties

- **Hybrid security**: the scheme is secure as long as *either* ECDH on Pallas *or*
  ML-KEM-768 remains unbroken. A quantum computer that breaks ECDH but not MLWE
  cannot recover the symmetric key, and vice versa.
- **Address unlinkability**: each diversified address carries a unique
  $\mathsf{ek\_{pq\_d}}$ derived from the master seed and the diversifier, so
  colluding senders cannot link addresses to the same wallet.
- **Quantum privacy of the hint**: a quantum adversary can recover the diversifier
  from the hint (by breaking ECDH), but cannot verify whether it is valid without
  performing ML-KEM KeyGen + decapsulation. No transaction graph information leaks
  to quantum adversaries.
- **Classical privacy**: a classical adversary cannot recover the diversifier from
  the hint (ECDH is secure), so the hint reveals nothing.
- **Deterministic encapsulation**: the ML-KEM randomness is derived from the note's
  $\mathsf{rseed}$ and $\rho$, ensuring that encryption is reproducible for proof
  generation without requiring additional random state.
- **Implicit rejection**: ML-KEM-768 uses implicit rejection --- invalid ciphertexts
  produce a pseudorandom shared secret rather than an error, preventing chosen-ciphertext
  attacks.

## Size impact

The hybrid KEM adds overhead to transactions:

| Component               | Classical | Hybrid      | Increase     |
|-------------------------|-----------|-------------|--------------|
| Payment address         | 43 bytes  | 1,227 bytes | +1,184 bytes |
| Ciphertext per action   | 0 bytes   | 1,088 bytes | +1,088 bytes |
| Diversifier hint/action | 0 bytes   | 11 bytes    | +11 bytes    |

The on-chain note format (`RawAddress`, note commitment, nullifier) is unchanged.
The additional ciphertext $\mathsf{ct\_{pq}}$ and 11-byte diversifier hint are
included alongside each action's encrypted output.

## Trade-offs

| Decision | Benefit | Cost |
|----------|---------|------|
| No MAC on hint | Full quantum privacy | ~3--6x slower scanning (ML-KEM KeyGen per tx) |
| Per-diversifier keys | Address unlinkability restored | ML-KEM KeyGen (~1ms) per address derivation |
| `pq_seed` in FVK/IVK | 64 bytes vs 3584 bytes (ek+dk) | On-demand key derivation instead of pre-computed |
| 11-byte hint on-chain | Negligible vs 1088-byte ct_pq | --- |
