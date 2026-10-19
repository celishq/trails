//! One rule as a store holds it.

use crate::Rule;
use celis_nostr::Note;

/// One rule as a store holds it: its record, what it says, the id of the rule it overrides (CIP-07),
/// the rule that replaces it, if one does, and the drop that names it, if one does. The rule that replaces the rule an override overrides
/// replaces the override too.
#[derive(Clone)]
pub struct RuleEntry<'a> {
    pub note: &'a Note,
    pub rule: Rule,
    pub overrides: Option<String>,
    pub by: Option<&'a Note>,
    pub dropped: Option<&'a Note>,
}

impl<'a> RuleEntry<'a> {
    /// Whether no rule replaces it and no drop names it.
    pub fn is_live(&self) -> bool {
        self.by.is_none() && self.dropped.is_none()
    }

    /// Its record and what it says.
    pub fn said(&self) -> (&'a Note, Rule) {
        (self.note, self.rule.clone())
    }
}
