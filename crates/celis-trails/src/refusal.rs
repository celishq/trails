//! Why a write or a read of the trails did not go through.

/// Why a write or a read did not go through, by the error names of CIP-02: each with the id, text or
/// output it is about.
#[derive(Debug)]
pub enum Refusal {
    Usage(String),
    NoStep(String),
    NoPattern(String),
    NoTrail(String),
    Closed(String),
    OpenSteps(String),
    CheckFailed(String),
    Error(anyhow::Error),
}

impl From<anyhow::Error> for Refusal {
    fn from(error: anyhow::Error) -> Self {
        Refusal::Error(error)
    }
}
