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
        Refusal::Usage(message.into()).into()
    }

    /// A command that printed its own error: it ends with `code` and prints nothing more.
    pub fn quiet(code: i32) -> Self {
        Self {
            name: "error".into(),
            message: String::new(),
            code,
        }
    }
}

impl From<Refusal> for Failure {
    fn from(refusal: Refusal) -> Self {
        let code = match refusal {
            Refusal::Usage(_) => 2,
            _ => 1,
        };
        Self {
            name: refusal.name().into(),
            message: message(refusal),
            code,
        }
    }
}

impl From<anyhow::Error> for Failure {
    fn from(error: anyhow::Error) -> Self {
        Refusal::Error(error).into()
    }
}

/// What a person reads of `refusal`; an error about an id or a state names the command that fixes it.
fn message(refusal: Refusal) -> String {
    match refusal {
        Refusal::Usage(message) => message,
        Refusal::NoStep(id) => format!("no step has the id {id}; find it with `celis tree --all`"),
        Refusal::NoPattern(id) => format!(
            "no live pattern has the id {id}; see the live ones with `celis grep \" live \"`"
        ),
        Refusal::Closed(id) => {
            format!("the step {id} is closed; add a new step with `celis step add`")
        }
        Refusal::OpenSteps(id) => format!(
            "the step {id} has an open step under it; see it with `celis tree {}`",
            celis_store::records::id::short(&id)
        ),
        Refusal::CheckFailed(output) => format!("the step's check fails:\n{output}"),
        Refusal::Secret(what) => format!(
            "{what}, and a record is signed and cannot be deleted; name the secret by where it \
             lives, such as API_KEY in .env, and run the command again"
        ),
        Refusal::Invalid(why) => format!("the cell is not kept: {why}"),
        Refusal::NoCell(name) => {
            format!("no cell is named {name}; see the cells with `celis cells`")
        }
        Refusal::CellYours(name) => format!(
            "the cell {name} is yours, and may be on relays; turn it off with `celis cell off {name}`"
        ),
        Refusal::CellTaken(name) => {
            format!("a cell of another account is named {name}; choose another name")
        }
        Refusal::Error(error) => format!("{error:#}"),
    }
}

#[cfg(test)]
mod tests;
