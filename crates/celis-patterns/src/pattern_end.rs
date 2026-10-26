//! How a pattern stopped holding.

use celis_nostr::Note;
use celis_store::records::id::short;

/// How a pattern stopped holding: a pattern replaced it, or a drop named it (CIP-05, "Patterns").
#[derive(Debug, Clone, Copy)]
pub enum PatternEnd<'a> {
    Replaced(&'a Note),
    Dropped(&'a Note),
}

impl PatternEnd<'_> {
    /// The word for it that `grep` gives: `replaced-by <id8>` or `dropped`.
    pub fn text(self) -> String {
        match self {
            PatternEnd::Replaced(by) => format!("replaced-by {}", short(&by.id)),
            PatternEnd::Dropped(_) => "dropped".to_string(),
        }
    }
}
