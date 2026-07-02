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
///
/// Re-exported from [`zcash_note_encryption`] so the crate has a single
/// source of truth for this width, which determines where the memo
/// ciphertext ends in [ZIP-244]-style transaction digests.
///
/// [ZIP-244]: https://zips.z.cash/zip-0244
pub use zcash_note_encryption::AEAD_TAG_SIZE;

/// The largest supported memo size (the Zcash 512-byte memo).
///
/// Note encryption assembles plaintexts in a stack buffer of
/// `COMPACT_NOTE_SIZE + MAX_MEMO_SIZE` bytes, so [`MemoSize`] implementations
/// with larger memos are rejected at compile time by [`MemoSize::SIZE_CHECK`].
pub const MAX_MEMO_SIZE: usize = 512;

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
/// | `Memo` | `[u8; MEMO_SIZE]` |
/// | `NotePlaintextBytes` | `COMPACT_NOTE_SIZE + MEMO_SIZE` |
/// | `NoteCiphertextBytes` | `COMPACT_NOTE_SIZE + MEMO_SIZE + AEAD_TAG_SIZE` |
///
/// These relationships are enforced at compile time via [`Self::SIZE_CHECK`].
pub trait MemoSize: Clone + Debug + 'static {
    /// The memo length in bytes (e.g. 512 for Zcash, 36 for Dash).
    const MEMO_SIZE: usize;

    /// The memo bytes type (e.g., `[u8; 512]` for Zcash, `[u8; 36]` for Dash).
    type Memo: AsRef<[u8]> + Clone + Debug + for<'a> TryFrom<&'a [u8]>;

    /// The note plaintext bytes type (`COMPACT_NOTE_SIZE + MEMO_SIZE`).
    type NotePlaintextBytes: NoteBytes;

    /// The note ciphertext bytes type (`COMPACT_NOTE_SIZE + MEMO_SIZE + AEAD_TAG_SIZE`).
    type NoteCiphertextBytes: NoteBytes;

    /// Compile-time validation that the associated types agree with
    /// [`Self::MEMO_SIZE`].
    ///
    /// This constant is referenced by the note encryption and transaction
    /// digest code paths, forcing its evaluation when those paths are
    /// monomorphized: an implementation whose associated types do not
    /// satisfy the size relationships documented on this trait fails to
    /// compile instead of panicking at runtime.
    const SIZE_CHECK: () = {
        assert!(
            Self::MEMO_SIZE <= MAX_MEMO_SIZE,
            "MemoSize::MEMO_SIZE must not exceed MAX_MEMO_SIZE"
        );
        assert!(
            core::mem::size_of::<Self::Memo>() == Self::MEMO_SIZE,
            "MemoSize::Memo must be a byte array of MEMO_SIZE bytes"
        );
        assert!(
            core::mem::size_of::<Self::NotePlaintextBytes>() == COMPACT_NOTE_SIZE + Self::MEMO_SIZE,
            "MemoSize::NotePlaintextBytes must hold COMPACT_NOTE_SIZE + MEMO_SIZE bytes"
        );
        assert!(
            core::mem::size_of::<Self::NoteCiphertextBytes>()
                == COMPACT_NOTE_SIZE + Self::MEMO_SIZE + AEAD_TAG_SIZE,
            "MemoSize::NoteCiphertextBytes must hold COMPACT_NOTE_SIZE + MEMO_SIZE + AEAD_TAG_SIZE bytes"
        );
    };

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
    const MEMO_SIZE: usize = 512;

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
    const MEMO_SIZE: usize = 36;

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

#[cfg(test)]
mod tests {
    use super::{DashMemo, MemoSize, ZcashMemo};

    #[test]
    #[allow(clippy::let_unit_value)]
    fn size_checks_hold_for_provided_impls() {
        // Referencing SIZE_CHECK forces its compile-time evaluation; a
        // mis-sized implementation would fail this test by failing to build.
        let _ = ZcashMemo::SIZE_CHECK;
        let _ = DashMemo::SIZE_CHECK;
        assert_eq!(ZcashMemo::MEMO_SIZE, 512);
        assert_eq!(DashMemo::MEMO_SIZE, 36);
    }
}
