//! What the person must look at when an ask closes.

use celis_nostr::Note;
use celis_trails::{Mark, Step, StepId};

/// What the person must look at when an ask closes: the edits of the ask, the steps whose proof is the
/// agent's own claim, the places of its steps, and its done steps by who gave the proof, with the minutes
/// since the ask began.
pub struct Gap {
    pub edited: Vec<String>,
    pub claims: Vec<(StepId, String)>,
    pub places: Vec<String>,
    pub steps: usize,
    pub checked: usize,
    pub judged: usize,
    pub person: usize,
    pub claim: usize,
    pub minutes: u64,
}

impl Gap {
    /// The gap of the ask `step`, just done with its proof `by`, at `now`; `records` are the store's
    /// before the done.
    pub fn of(records: &[Note], step: Step, by: Option<&str>, now: u64) -> Gap {
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
        let done: Vec<(Step, Option<String>)> = tree
            .iter()
            .filter_map(|s| match s.id() == step.id() {
                true => Some((*s, by.map(str::to_string))),
                false => Some((*s, s.done()?.tag("by").map(str::to_string))),
            })
            .collect();
        let kind = |s: &Step, by: &Option<String>| match (by.as_deref(), &s.record().check) {
            (Some("judge"), _) => "judged",
            (Some("person"), _) => "person",
            (_, Some(_)) => "checked",
            _ => "claim",
        };
        let count = |k: &str| done.iter().filter(|(s, by)| kind(s, by) == k).count();
        let mut places: Vec<String> = tree
            .iter()
            .flat_map(|s| s.record().places.iter().cloned())
            .collect();
        places.sort();
        places.dedup();
        let from = step.start().unwrap_or(step.note()).created_at;
        Gap {
            edited,
            claims: done
                .iter()
                .filter(|(s, by)| kind(s, by) == "claim")
                .map(|(s, _)| (s.step_id(), s.title().to_string()))
                .collect(),
            places,
            steps: done.len(),
            checked: count("checked"),
            judged: count("judged"),
            person: count("person"),
            claim: count("claim"),
            minutes: now.saturating_sub(from) / 60,
        }
    }
}
