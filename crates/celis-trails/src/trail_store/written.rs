//! What one write of the trails gives back.

use crate::Notice;
use celis_nostr::Note;
use celis_patterns::Pattern;

/// What one write gives back: the records it wrote or found, the patterns and notes the work must keep,
/// and the notices for the agent.
#[derive(Default)]
pub struct Written<'a> {
    pub records: Vec<Note>,
    pub patterns: Vec<Pattern<'a>>,
    pub notes: Vec<&'a Note>,
    pub notices: Vec<Notice>,
}
