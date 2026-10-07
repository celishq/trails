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
        Self {
            name: "usage".into(),
            message: message.into(),
            code: 2,
        }
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

    /// No cairn in the store has this id.
    pub fn no_cairn(id: &str) -> Self {
        Self {
            name: "no-cairn".into(),
            message: format!("no cairn has the id {id}"),
            code: 1,
        }
    }
}

impl From<anyhow::Error> for Failure {
    fn from(error: anyhow::Error) -> Self {
        Self {
            name: "error".into(),
            message: format!("{error:#}"),
            code: 1,
        }
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
