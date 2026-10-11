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

    /// Ends the message by naming the help `page`.
    pub fn see(mut self, page: &str) -> Self {
        let stop = if self.message.ends_with('?') { "" } else { "." };
        self.message = format!("{}{stop} See `{page}`.", self.message);
        self
    }

    /// The words name no command; the message names the nearest of `known`, if one is within two edits.
    pub fn unknown<'a>(word: &str, known: impl IntoIterator<Item = &'a str>) -> Self {
        let hint = Self::nearest(word, known)
            .map_or(String::new(), |n| format!("; did you mean \"{n}\"?"));
        Self::usage(format!("no command \"{word}\"{hint}"))
    }

    /// The word of `known` nearest `word`, when two edits or fewer turn one into the other.
    pub fn nearest<'a>(word: &str, known: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
        known
            .into_iter()
            .map(|k| (edits(word, k), k))
            .filter(|(distance, _)| *distance <= 2)
            .min_by_key(|(distance, _)| *distance)
            .map(|(_, k)| k)
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
            format!("no live decision has the id {id}; see the live ones with `celis read`"),
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

/// The Levenshtein distance between two words.
fn edits(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diagonal + usize::from(x != *y))
                .min(above + 1)
                .min(row[j] + 1);
            diagonal = above;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests;
