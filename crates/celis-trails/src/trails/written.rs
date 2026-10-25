//! What one write of the trails gives back.

use crate::{Notice, Pattern};
use celis_nostr::Note;

/// What one write gives back: the records it wrote or found, the patterns the work must keep, and the
/// notices for the agent.
#[derive(Default)]
pub struct Written {
    pub records: Vec<Note>,
    pub patterns: Vec<(Note, Pattern)>,
    pub notices: Vec<Notice>,
}
