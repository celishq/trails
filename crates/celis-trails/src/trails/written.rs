//! What one write of the trails gives back.

use crate::{Decision, Notice};
use celis_nostr::Note;

/// What one write gives back: the records it wrote or found, the decisions the work must keep, and the
/// notices for the agent.
#[derive(Default)]
pub struct Written {
    pub records: Vec<Note>,
    pub decisions: Vec<(Note, Decision)>,
    pub notices: Vec<Notice>,
}
