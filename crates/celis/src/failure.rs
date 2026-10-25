//! Why a command did not finish.

use celis_trails::Refusal;

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
                celis_trails::records::id::short(id)
            ),
        )
    }

    /// The ask changed files and names no one who checked it.
    pub fn unjudged(id: &str) -> Self {
        Self::new(
            "unjudged",
            1,
            format!(
                "the ask {} changed files; run the celis:judge agent with the brief above, then close it \
                 --by judge, or --by person with the person's answer",
                celis_trails::records::id::short(id)
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
            Refusal::NoTrail(id) => Self::no_trail(&id),
            Refusal::Closed(id) => Self::closed(&id),
            Refusal::OpenSteps(id) => Self::open_steps(&id),
            Refusal::CheckFailed(output) => Self::check_failed(&output),
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
