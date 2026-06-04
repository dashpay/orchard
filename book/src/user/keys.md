# Creating keys and addresses

## Key generation

Orchard keys are derived from a spending key, which is itself derived from a wallet seed
using [ZIP 32] hierarchical deterministic derivation:

```rust,ignore
use orchard::keys::{SpendingKey, FullViewingKey, Scope};
use zip32::AccountId;

let sk = SpendingKey::from_zip32_seed(&seed, coin_type, account)?;
let fvk = FullViewingKey::from(&sk);
```

The spending key deterministically produces all other key material, including the full
viewing key, incoming/outgoing viewing keys, and (when the `hybrid-kem` feature is
enabled) the post-quantum encapsulation keypair.

## Addresses

A payment address is derived from the full viewing key using a diversifier index:

```rust,ignore
let address = fvk.address_at(0u32, Scope::External);
```

### Classical addresses

Without the `hybrid-kem` feature, an address is 43 bytes: an 11-byte diversifier and
a 32-byte diversified transmission key. All diversifier indices produce valid addresses.

### Hybrid post-quantum addresses

With the `hybrid-kem` feature enabled, addresses contain an additional 1184-byte
ML-KEM-768 encapsulation key, for a total of 1227 bytes. This key is required for
the sender to perform hybrid key encapsulation when encrypting a note.

The `Address` type enforces that the encapsulation key is always present. Internal
types that reconstruct notes from on-chain data use `RawAddress`, which contains only
the classical 43-byte components.

When serialized as text (for display, copy-paste, or URLs), addresses use Base58
encoding, producing a string of approximately 1678 characters. For QR codes, wallets
should encode the raw binary directly for the smallest possible QR code.

### Address types

| Type          | Contents                          | Size      | Used for                        |
|---------------|-----------------------------------|-----------|---------------------------------|
| `RawAddress`  | Diversifier + pk_d                | 43 bytes  | On-chain data, note decryption  |
| `Address`     | RawAddress + ek_pq (hybrid only)  | 1,227 bytes | Payment addresses, sending    |

When `hybrid-kem` is not enabled, `RawAddress` is a type alias for `Address` and the
two are interchangeable.

## Scopes

Orchard supports two address scopes:

- **External** (`Scope::External`): for addresses shared with other parties.
- **Internal** (`Scope::Internal`): for change addresses, not shared externally.

Both scopes derive from the same full viewing key but use different incoming viewing
keys.

```rust,ignore
let external_addr = fvk.address_at(0u32, Scope::External);
let internal_addr = fvk.address_at(0u32, Scope::Internal);
```

## Viewing keys

The full viewing key can detect both incoming and outgoing transactions. For read-only
access to incoming funds, use the incoming viewing key:

```rust,ignore
let ivk = fvk.to_ivk(Scope::External);
```

Note that `IncomingViewingKey::address_at` returns a `RawAddress` (without the PQ
encapsulation key), since the incoming viewing key does not have access to the spending
key material needed to derive it. To obtain a full `Address` suitable for sharing with
senders, use `FullViewingKey::address_at`.

[ZIP 32]: https://zips.z.cash/zip-0032
