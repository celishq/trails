//! The good and the bad step of a rule.

use crate::StepId;
use celis_nostr::Note;

/// The good and the bad step of a rule: the done steps that show by example what its text means
/// (CIP-05, "Pairs").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pair {
    pub good: Option<StepId>,
    pub bad: Option<StepId>,
}

impl Pair {
    /// Each step it names, after its tag name: `good`, then `bad`.
    pub fn sides(&self) -> impl Iterator<Item = (&'static str, &StepId)> {
        [("good", &self.good), ("bad", &self.bad)]
            .into_iter()
            .filter_map(|(side, id)| Some((side, id.as_ref()?)))
    }

    /// The pair of the rule `note` holds.
    pub fn read(note: &Note) -> Pair {
        Pair {
            good: note.tag("good").map(StepId::new),
            bad: note.tag("bad").map(StepId::new),
        }
    }
}
