//! **What a relay said about one note** — NIP-01's `OK` frame.

/// Whether a relay took the note, and its own reason when it did not.
///
/// The reason travels as the relay wrote it. NIP-01 puts the machine-readable part in a prefix and leaves
/// the rest for a human, so a reader of the reason reads the prefix and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub accepted: bool,
    pub reason: String,
}
