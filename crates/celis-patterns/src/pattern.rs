//! CIP-05: a pattern, what to choose in a context and why, that holds in its scope until another replaces
//! it.

use crate::{PatternEnd, PatternRecord};
use celis_nostr::Note;

/// One pattern of a store: its record, what it says, and how it ended, if it did.
#[derive(Debug, Clone)]
pub struct Pattern<'a> {
    pub note: &'a Note,
    pub record: PatternRecord,
    pub end: Option<PatternEnd<'a>>,
}

impl<'a> Pattern<'a> {
    pub fn id(&self) -> &'a str {
        &self.note.id
    }

    /// Whether it has not ended.
    pub fn is_live(&self) -> bool {
        self.end.is_none()
    }
}
