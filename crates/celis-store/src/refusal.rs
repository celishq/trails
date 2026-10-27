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

impl Refusal {
    /// Its error name (`spec/02-cli.md`, "Output and errors").
    pub fn name(&self) -> &'static str {
        match self {
            Refusal::Usage(_) => "usage",
            Refusal::NoStep(_) => "no-step",
            Refusal::NoPattern(_) => "no-pattern",
            Refusal::Closed(_) => "closed",
            Refusal::OpenSteps(_) => "open-steps",
            Refusal::CheckFailed(_) => "check-failed",
            Refusal::Secret(_) => "secret",
            Refusal::Invalid(_) => "invalid",
            Refusal::NoCell(_) => "no-cell",
            Refusal::CellYours(_) => "cell-yours",
            Refusal::CellTaken(_) => "cell-taken",
            Refusal::Error(_) => "error",
        }
    }
}

impl From<anyhow::Error> for Refusal {
    fn from(error: anyhow::Error) -> Self {
        Refusal::Error(error)
    }
}
