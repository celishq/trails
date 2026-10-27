//! Why a command did not finish.

use celis_store::Refusal;

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

    /// No live pattern has this id.
    pub fn no_pattern(id: &str) -> Self {
        Self::new(
            "no-pattern",
            1,
            format!(
                "no live pattern has the id {id}; see the live ones with `celis grep \" live \"`"
            ),
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
                celis_store::records::id::short(id)
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

    /// A record or a kept file holds a secret.
    pub fn secret(what: &str) -> Self {
        Self::new(
            "secret",
            1,
            format!(
                "{what}, and a record is signed and cannot be deleted; name the secret by where it \
                 lives, such as API_KEY in .env, and run the command again"
            ),
        )
    }

    /// No cell holds the name.
    pub fn no_cell(name: &str) -> Self {
        Self::new(
            "no-cell",
            1,
            format!("no cell is named {name}; see the cells with `celis cells`"),
        )
    }

    /// A cell of another account holds the name.
    pub fn cell_taken(name: &str) -> Self {
        Self::new(
            "cell-taken",
            1,
            format!("a cell of another account is named {name}; choose another name"),
        )
    }

    /// A command that printed its own error: it ends with `code` and prints nothing more.
    pub fn quiet(code: i32) -> Self {
        Self::new("error", code, String::new())
    }

    fn new(name: &str, code: i32, message: String) -> Self {
        Self {
            name: name.into(),
            message,
            code,
        }
    }
}

impl From<Refusal> for Failure {
    fn from(refusal: Refusal) -> Self {
        match refusal {
            Refusal::Usage(message) => Self::usage(message),
            Refusal::NoStep(id) => Self::no_step(&id),
            Refusal::NoPattern(id) => Self::no_pattern(&id),
            Refusal::Closed(id) => Self::closed(&id),
            Refusal::OpenSteps(id) => Self::open_steps(&id),
            Refusal::CheckFailed(output) => Self::check_failed(&output),
            Refusal::Secret(what) => Self::secret(&what),
            Refusal::CellTaken(name) => Self::cell_taken(&name),
            Refusal::NoCell(name) => Self::no_cell(&name),
            Refusal::Invalid(why) => {
                Self::new("invalid", 1, format!("the cell is not kept: {why}"))
            }
            Refusal::Error(error) => Self::from(error),
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
