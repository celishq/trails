//! One pattern as a store holds it.

use crate::{Pattern, PatternEnd};
use celis_nostr::Note;

/// One pattern as a store holds it: its record, what it says, and how it ended, if it did.
#[derive(Clone)]
pub struct PatternEntry<'a> {
    pub note: &'a Note,
    pub pattern: Pattern,
    pub end: Option<PatternEnd<'a>>,
}

impl<'a> PatternEntry<'a> {
    /// Whether it has not ended.
    pub fn is_live(&self) -> bool {
        self.end.is_none()
    }

    /// Its record and what it says.
    pub fn said(&self) -> (&'a Note, Pattern) {
        (self.note, self.pattern.clone())
    }
}
