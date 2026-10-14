//! Why a command did not finish.

/// A command that did not finish: the name `--json` prints, the message a person reads, and the exit code.
#[derive(Debug)]
pub struct Failure {
    pub name: String,
    pub message: String,
    pub code: i32,
}

impl Failure {
    /// A command used wrongly.
    pub fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", 2, message.into())
    }

    /// No step in the store has this id.
    pub fn no_step(id: &str) -> Self {
        Self::new(
            "no-step",
            1,
            format!("no step has the id {id}; find it with `celis tree --all`"),
        )
    }

    /// No live decision has this id.
    pub fn no_decision(id: &str) -> Self {
        Self::new(
            "no-decision",
            1,
            format!("no live decision has the id {id}; see the live ones with `celis past`"),
        )
    }

    /// No trail in the store has this id.
    pub fn no_trail(id: &str) -> Self {
        Self::new(
            "no-trail",
            1,
            format!("no trail has the id {id}; find it with `celis tree --all`"),
        )
    }

    /// The step's check fails; `output` is the end of what it printed.
    pub fn check_failed(output: &str) -> Self {
        Self::new(
            "check-failed",
            1,
            format!("the step's check fails:\n{output}"),
        )
    }

    /// The step has an open step under it.
    pub fn open_steps(id: &str) -> Self {
        Self::new(
            "open-steps",
            1,
            format!(
                "the step {id} has an open step under it; see it with `celis tree {}`",
                crate::id::short(id)
            ),
        )
    }

    /// The step is done.
    pub fn closed(id: &str) -> Self {
        Self::new(
            "closed",
            1,
            format!("the step {id} is closed; add a new step with `celis step add`"),
        )
    }

    fn new(name: &str, code: i32, message: String) -> Self {
        Self {
            name: name.into(),
            message,
            code,
        }
    }
}

impl From<anyhow::Error> for Failure {
    fn from(error: anyhow::Error) -> Self {
        Self::new("error", 1, format!("{error:#}"))
    }
}

#[cfg(test)]
mod tests;
