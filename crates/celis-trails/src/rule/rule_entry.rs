//! One rule as a store holds it.

use crate::{Rule, RuleEnd};
use celis_nostr::Note;

/// One rule as a store holds it: its record, what it says, and how it ended, if it did.
#[derive(Clone)]
pub struct RuleEntry<'a> {
    pub note: &'a Note,
    pub rule: Rule,
    pub end: Option<RuleEnd<'a>>,
}

impl<'a> RuleEntry<'a> {
    /// Whether it has not ended.
    pub fn is_live(&self) -> bool {
        self.end.is_none()
    }

    /// Its record and what it says.
    pub fn said(&self) -> (&'a Note, Rule) {
        (self.note, self.rule.clone())
    }
}
