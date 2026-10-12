use crate::Note;

/// What one stream has said since a landing last took it: the notes its socket handed over, and whether
/// the walk from the cursor is owed — a connect says so and carries no note.
#[derive(Debug, Default)]
pub struct Held {
    pub notes: Vec<Note>,
    pub walk: bool,
}

impl Held {
    /// A stream owed its walk, with nothing heard.
    pub fn walking() -> Self {
        Self {
            notes: Vec::new(),
            walk: true,
        }
    }
}
