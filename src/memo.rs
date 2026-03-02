//! Memo size abstraction for protocol-parameterized note encryption.
//!
//! The Orchard protocol encrypts memos outside the ZK circuit (via ChaCha20-Poly1305 AEAD),
//! so the memo size can vary without affecting the proof system. This module provides a
//! [`MemoSize`] trait that parameterizes the memo-dependent sizes, allowing different
//! protocols (e.g., Zcash with 512-byte memos, Dash with 36-byte memos) to share the
//! same Orchard implementation.

use core::fmt::Debug;
use zcash_note_encryption::note_bytes::NoteBytes;

/// The compact note size (version + diversifier + value + rseed), fixed for all Orchard
/// memo sizes.
pub const COMPACT_NOTE_SIZE: usize = 52;

/// The AEAD tag size used by ChaCha20-Poly1305.
const AEAD_TAG_SIZE: usize = 16;

/// Trait defining memo-dependent sizes for the Orchard protocol.
///
/// Implementations of this trait specify the memo type and the corresponding
/// note plaintext and ciphertext sizes. The compact note size (52 bytes) is
/// always the same regardless of memo size.
///
/// # Size relationships
///
/// | Field | Formula |
/// |-------|---------|
/// | `NotePlaintextBytes` | `COMPACT_NOTE_SIZE + memo_len` |
/// | `NoteCiphertextBytes` | `COMPACT_NOTE_SIZE + memo_len + 16` |
pub trait MemoSize: Clone + Debug + 'static {
    /// The memo bytes type (e.g., `[u8; 512]` for Zcash, `[u8; 36]` for Dash).
    type Memo: AsRef<[u8]> + Clone + Debug + for<'a> TryFrom<&'a [u8]>;

    /// The note plaintext bytes type (`COMPACT_NOTE_SIZE + memo_len`).
    type NotePlaintextBytes: NoteBytes;

    /// The note ciphertext bytes type (`COMPACT_NOTE_SIZE + memo_len + AEAD_TAG_SIZE`).
    type NoteCiphertextBytes: NoteBytes;

    /// Returns a zero-filled memo.
    fn empty_memo() -> Self::Memo;

    /// Constructs a memo from a byte slice.
    ///
    /// Returns `None` if the slice length doesn't match the expected memo size.
    fn memo_from_bytes(bytes: &[u8]) -> Option<Self::Memo>;
}

/// Zcash standard memo (512 bytes).
///
/// This is the default memo size, used for Zcash Orchard transactions.
///
/// | Constant | Value |
/// |----------|-------|
/// | MEMO_SIZE | 512 |
/// | NOTE_PLAINTEXT_SIZE | 564 |
/// | ENC_CIPHERTEXT_SIZE | 580 |
#[derive(Clone, Debug)]
pub struct ZcashMemo;

impl MemoSize for ZcashMemo {
    type Memo = [u8; 512];
    type NotePlaintextBytes = zcash_note_encryption::note_bytes::NoteBytesData<564>;
    type NoteCiphertextBytes = zcash_note_encryption::note_bytes::NoteBytesData<580>;

    fn empty_memo() -> Self::Memo {
        [0u8; 512]
    }

    fn memo_from_bytes(bytes: &[u8]) -> Option<Self::Memo> {
        bytes.try_into().ok()
    }
}

/// Dash compact memo (36 bytes).
///
/// Used for Dash Platform's shielded pool, which requires smaller memos.
///
/// | Constant | Value |
/// |----------|-------|
/// | MEMO_SIZE | 36 |
/// | NOTE_PLAINTEXT_SIZE | 88 |
/// | ENC_CIPHERTEXT_SIZE | 104 |
#[derive(Clone, Debug)]
pub struct DashMemo;

impl MemoSize for DashMemo {
    type Memo = [u8; 36];
    type NotePlaintextBytes = zcash_note_encryption::note_bytes::NoteBytesData<88>;
    type NoteCiphertextBytes = zcash_note_encryption::note_bytes::NoteBytesData<104>;

    fn empty_memo() -> Self::Memo {
        [0u8; 36]
    }

    fn memo_from_bytes(bytes: &[u8]) -> Option<Self::Memo> {
        bytes.try_into().ok()
    }
}
