//! Why a write or a read of the store did not go through.

/// Why a write or a read did not go through, by the error names of CIP-02: each with the id, text or
/// output it is about.
#[derive(Debug)]
pub enum Refusal {
    Usage(String),
    NoStep(String),
    NoPattern(String),
    Closed(String),
    OpenSteps(String),
    CheckFailed(String),
    /// What holds a secret, and its kind, such as `the file .env holds an API key`.
    Secret(String),
    /// What makes the records of a cell's folder unfit to keep.
    Invalid(String),
    /// The cell name that no cell holds.
    NoCell(String),
    /// The cell of the account, which only turns off.
    CellYours(String),
    /// The cell name that a cell of another account holds.
    CellTaken(String),
    Error(anyhow::Error),
}

impl From<anyhow::Error> for Refusal {
    fn from(error: anyhow::Error) -> Self {
        Refusal::Error(error)
    }
}
