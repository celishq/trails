//! How a rule stopped holding.

use crate::records::id::short;
use celis_nostr::Note;

/// How a rule stopped holding: a rule replaced it, or a drop named it (CIP-05, "Rules").
#[derive(Clone, Copy)]
pub enum RuleEnd<'a> {
    Replaced(&'a Note),
    Dropped(&'a Note),
}

impl RuleEnd<'_> {
    /// The word for it that `grep` gives: `replaced-by <id8>` or `dropped`.
    pub fn text(self) -> String {
        match self {
            RuleEnd::Replaced(by) => format!("replaced-by {}", short(&by.id)),
            RuleEnd::Dropped(_) => "dropped".to_string(),
        }
    }
}
