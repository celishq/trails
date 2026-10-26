//! What the person must look at when an ask closes.

use super::{Learn, Proof};
use celis_nostr::Note;
use celis_store::StepId;
use celis_trails::{Done, Step};

/// What the person must look at when an ask closes: each done step with who stands behind its proof and
/// what changed from its plan, the places of its steps, the minutes since the ask began, and what it
/// taught.
pub(crate) struct End {
    pub ask: StepId,
    pub proofs: Vec<(String, Proof)>,
    pub turns: Vec<(String, String)>,
    pub places: Vec<String>,
    pub minutes: u64,
    pub learn: Learn,
}

impl End {
    /// The end of the ask `step`, just closed `done` at `now`, from `records`.
    pub fn of(records: &[Note], step: Step, done: &Done, now: u64) -> End {
        let tree = step.tree();
        let closed: Vec<(Step, Done)> = tree
            .iter()
            .filter_map(|s| match s.id() == step.id() {
                true => Some((*s, done.clone())),
                false => Some((*s, s.done()?)),
            })
            .collect();
        let mut places: Vec<String> = tree
            .iter()
            .flat_map(|s| s.record().places.iter().cloned())
            .collect();
        places.sort();
        places.dedup();
        let from = step.start().unwrap_or(step.note()).created_at;
        End {
            ask: step.step_id(),
            proofs: closed
                .iter()
                .map(|(s, d)| (s.title().to_string(), Proof::of(*s, d)))
                .collect(),
            turns: closed
                .iter()
                .filter_map(|(s, d)| Some((s.title().to_string(), d.changed.clone()?)))
                .collect(),
            places,
            minutes: now.saturating_sub(from) / 60,
            learn: Learn::of(records, step, done),
        }
    }

    /// The lines to give the agent: what to do, the draft it fills for the person, with a `<…>` for each
    /// part only the agent knows, then the learn brief.
    pub fn lines(&self) -> Vec<String> {
        let outside = match self.places.is_empty() {
            true => String::new(),
            false => format!(
                ", and each file git diff --stat shows changed outside {}",
                self.places.join(", ")
            ),
        };
        let mut lines = vec![
            format!("celis: the ask {} is done. Do the learn below the total line first. Then answer the person with the lines down to the total line alone: fill each <…>, and remove each line you do not need.", &self.ask[..8]),
            String::new(),
            "**<what was done, in one line>**".to_string(),
            String::new(),
            "- [?] **Needs you:** <each decision only the person can take>".to_string(),
        ];
        let claims = self.proofs.iter().filter(|(_, p)| *p == Proof::Claim);
        lines.extend(claims.map(|(title, _)| format!("- [!] **Not proven:** {title}")));
        lines.push(format!(
            "- [!] **Not proven:** <what the judge could not check{outside}>"
        ));
        let turns = self.turns.iter();
        lines.extend(
            turns.map(|(title, changed)| {
                format!("- [+] **I chose:** {title}: {changed} · {DECIDED}")
            }),
        );
        lines.push(format!(
            "- [+] **I chose:** <each choice you took yourself> · {DECIDED}"
        ));
        lines.push("- [+] **Learned:** <each pattern you wrote, as its id and When>".to_string());
        lines.push(String::new());
        lines.push(self.total());
        lines.extend(self.learn.lines());
        lines
    }

    /// Its total line: the steps, each kind of proof that is not zero, or `all` of the one kind, then the
    /// minutes.
    fn total(&self) -> String {
        let steps = self.proofs.len();
        let step = if steps == 1 { "step" } else { "steps" };
        let mut parts = vec!["celis".to_string(), format!("{steps} {step}")];
        let kinds: Vec<(usize, Proof)> = Proof::ALL
            .iter()
            .map(|p| (self.proofs.iter().filter(|(_, q)| q == p).count(), *p))
            .filter(|(n, _)| *n > 0)
            .collect();
        match kinds.as_slice() {
            [(n, proof)] if *n > 1 => parts.push(format!("all {}", proof.word())),
            _ => parts.extend(kinds.iter().map(|(n, p)| format!("{n} {}", p.word()))),
        }
        parts.push(format!("{}m", self.minutes));
        parts.join(" · ")
    }
}

/// What follows each choice of the end: the pattern behind it.
const DECIDED: &str = "<the pattern that decided it, or none>";
