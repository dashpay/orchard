//! End-to-end coverage for the Dash memo size (36-byte memos, 104-byte note
//! ciphertexts), exercised through the public API exactly as a downstream
//! wallet would use it.

#![cfg(feature = "circuit")]

use incrementalmerkletree::Hashable;
use orchard::{
    builder::{Builder, BundleType},
    bundle::Flags,
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey},
    memo::DashMemo,
    note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho},
    note_encryption::{CompactAction, OrchardDomain, OrchardNoteEncryption},
    tree::MerkleHashOrchard,
    value::NoteValue,
    Note,
};
use rand::rngs::OsRng;
use zcash_note_encryption::{
    try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk, Domain,
};

const MEMO: [u8; 36] = [0xa5; 36];

/// Builds a shielding bundle carrying a 36-byte memo and runs every receiver-
/// and sender-side decryption path against it.
#[test]
fn dash_memo_bundle_round_trip() {
    let mut rng = OsRng;

    let sk = SpendingKey::from_bytes([7; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);
    let ovk = fvk.to_ovk(Scope::External);
    let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));

    let value = NoteValue::from_raw(5000);
    let anchor = MerkleHashOrchard::empty_root(32.into()).into();
    let mut builder: Builder<DashMemo> = Builder::new(
        BundleType::Transactional {
            flags: Flags::SPENDS_DISABLED,
            bundle_required: false,
        },
        anchor,
    );
    builder
        .add_output(Some(ovk.clone()), recipient, value, MEMO)
        .unwrap();
    let (bundle, _) = builder.build::<i64>(&mut rng).unwrap().unwrap();

    // The txid digest slices the 104-byte ciphertexts; it must not panic and
    // must be computable for a Dash-sized bundle.
    let _ = bundle.commitment();

    // Receiver: full trial decryption.
    let (note, address, memo) = bundle
        .actions()
        .iter()
        .find_map(|action| {
            let domain = OrchardDomain::for_action(action);
            try_note_decryption(&domain, &ivk, action)
        })
        .expect("recipient can decrypt their output");
    assert_eq!(note.value(), value);
    assert_eq!(address, recipient);
    assert_eq!(memo, MEMO);

    // Light client: compact decryption (no memo).
    let (compact_note, compact_address) = bundle
        .actions()
        .iter()
        .find_map(|action| {
            let compact = CompactAction::from(action);
            let domain = OrchardDomain::<DashMemo>::for_compact_action(&compact);
            try_compact_note_decryption(&domain, &ivk, &compact)
        })
        .expect("recipient can decrypt the compact output");
    assert_eq!(compact_note.value(), value);
    assert_eq!(compact_address, recipient);

    // Sender: output recovery with the outgoing viewing key.
    let (recovered_note, recovered_address, recovered_memo) = bundle
        .actions()
        .iter()
        .find_map(|action| {
            let domain = OrchardDomain::for_action(action);
            try_output_recovery_with_ovk(
                &domain,
                &ovk,
                action,
                action.cv_net(),
                &action.encrypted_note().out_ciphertext,
            )
        })
        .expect("sender can recover their output");
    assert_eq!(recovered_note.value(), value);
    assert_eq!(recovered_address, recipient);
    assert_eq!(recovered_memo, MEMO);
}

/// Golden vector pinning the Dash note ciphertext wire format (52-byte compact
/// part + 36-byte memo + 16-byte AEAD tag = 104 bytes). The encryption is
/// fully deterministic because esk and epk are derived from rseed. A change
/// to this vector is a consensus-visible wire format change.
#[test]
fn dash_memo_note_encryption_golden_vector() {
    let sk = SpendingKey::from_bytes([0x0d; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let nf = Nullifier::from_bytes(&[0x01; 32]).unwrap();
    // Same canonical field-element bytes as the nullifier, so this equals the
    // `CompactAction::rho()` the decryption domain below will derive from `nf`.
    let rho = Rho::from_bytes(&[0x01; 32]).unwrap();
    let rseed = RandomSeed::from_bytes([0x02; 32], &rho).unwrap();
    let note = Note::from_parts(recipient, NoteValue::from_raw(97_000), rho, rseed).unwrap();

    let ne = OrchardNoteEncryption::<DashMemo>::new(None, note, MEMO);
    let epk_bytes = OrchardDomain::<DashMemo>::epk_bytes(ne.epk());
    let enc = ne.encrypt_note_plaintext();
    let enc: &[u8] = enc.as_ref();

    assert_eq!(enc.len(), 104);
    assert_eq!(
        hex::encode(enc),
        "04f9d90d13a47ac7b61ccfa1f701bc3139845e541c111c767b0fdbd5c2871ccc\
         7cd71bc85af1d2fb4620270a254e32a6408e6254cfc7d2013adeda78ced47200\
         7c053ecd96d1d20e0f7cadef195e22ffb91e844d75001e9ef0fd1d378d31dba1\
         5cffdf8c3eaacc49"
    );

    // The golden ciphertext round-trips through compact decryption.
    let cmx = ExtractedNoteCommitment::from(note.commitment());
    let compact = CompactAction::from_parts(nf, cmx, epk_bytes, enc[..52].try_into().unwrap());
    let domain = OrchardDomain::<DashMemo>::for_compact_action(&compact);
    let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
    let (decrypted_note, decrypted_address) =
        try_compact_note_decryption(&domain, &ivk, &compact).expect("golden vector decrypts");
    assert_eq!(decrypted_note.value(), NoteValue::from_raw(97_000));
    assert_eq!(decrypted_address, recipient);
    assert_eq!(
        ExtractedNoteCommitment::from(decrypted_note.commitment()).to_bytes(),
        cmx.to_bytes()
    );
}
