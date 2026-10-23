//! A file of a proof.

/// A file of a proof: its place when the step closed, the sha256 of its bytes in lower-case hex, and its
/// size in bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofFile {
    pub place: String,
    pub sha256: String,
    pub size: u64,
}
