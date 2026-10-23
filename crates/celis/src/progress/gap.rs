//! What the person must look at when an ask closes.

use super::Proof;
use celis_nostr::Note;
use celis_trails::{Done, Mark, Step, StepId};

/// What the person must look at when an ask closes: the edits of the ask, each done step with who stands
/// behind its proof, the places of its steps, and the minutes since the ask began.
pub struct Gap {
    pub edited: Vec<String>,
    pub proofs: Vec<(StepId, String, Proof)>,
    pub places: Vec<String>,
    pub minutes: u64,
}

impl Gap {
    /// The gap of the ask `step`, just closed `done` at `now`; `records` are the store's before the done.
    pub fn of(records: &[Note], step: Step, done: &Done, now: u64) -> Gap {
        let edited = records
            .iter()
            .filter_map(|record| match Mark::read(record)? {
                (on, Mark::Edit { text, .. }) if on == step.id() && !text.trim().is_empty() => {
                    Some(text.lines().map(str::trim).collect::<Vec<_>>().join(" · "))
                }
                _ => None,
            })
            .collect();
        let tree = step.tree();
        let proofs = tree
            .iter()
            .filter_map(|s| {
                let closing = match s.id() == step.id() {
                    true => done.clone(),
                    false => s.closing()?,
                };
                Some((s.step_id(), s.title().to_string(), Proof::of(*s, &closing)))
            })
            .collect();
        let mut places: Vec<String> = tree
            .iter()
            .flat_map(|s| s.record().places.iter().cloned())
            .collect();
        places.sort();
        places.dedup();
        let from = step.start().unwrap_or(step.note()).created_at;
        Gap {
            edited,
            proofs,
            places,
            minutes: now.saturating_sub(from) / 60,
        }
    }

    /// The number of its done steps whose proof is `proof`.
    pub fn count(&self, proof: Proof) -> usize {
        self.proofs.iter().filter(|(.., p)| *p == proof).count()
    }
}
