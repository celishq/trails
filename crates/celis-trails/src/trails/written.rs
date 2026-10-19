//! What one write of the trails gives back.

use crate::{Notice, Rule};
use celis_nostr::Note;

/// What one write gives back: the records it wrote or found, the rules the work must keep, and the
/// notices for the agent.
#[derive(Default)]
pub struct Written {
    pub records: Vec<Note>,
    pub rules: Vec<(Note, Rule)>,
    pub notices: Vec<Notice>,
}
