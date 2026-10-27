//! The total of an ask when it closes.

use super::{Learn, Proof};
use crate::text::line::plural;
use celis_nostr::Note;
use celis_patterns::Patterns;
use celis_store::StepId;
use celis_trails::{Done, Step};

/// The total of an ask when it closes: each done step with who stands behind its proof, the patterns it
/// used and those written in its project since it began, the minutes since it began, and what it taught.
pub(crate) struct End {
    pub ask: StepId,
    pub proofs: Vec<(String, Proof)>,
    pub extracted: usize,
    pub minutes: u64,
    pub learn: Learn,
}

impl End {
    /// The end of the ask `step`, closed `done` at `now`, from `records`.
    pub fn of(records: &[Note], step: Step, done: &Done, now: u64) -> End {
        let tree = step.tree();
        let closed: Vec<(Step, Done)> = tree
            .iter()
            .filter_map(|s| match s.id() == step.id() {
                true => Some((*s, done.clone())),
                false => Some((*s, s.done()?)),
            })
            .collect();
        let from = step.start().unwrap_or(step.note()).created_at;
        let project = &step.record().anchor.project;
        let patterns = Patterns::new(records);
        End {
            ask: step.step_id(),
            proofs: closed
                .iter()
                .map(|(s, d)| (s.title().to_string(), Proof::of(*s, d)))
                .collect(),
            extracted: patterns
                .every()
                .iter()
                .filter(|p| &p.record.anchor.project == project && p.note.created_at >= from)
                .count(),
            minutes: now.saturating_sub(from) / 60,
            learn: Learn::of(records, step, done.changed.clone()),
        }
    }

    /// The lines to give the agent: the learn brief, the next move, then the total.
    pub fn lines(&self) -> Vec<String> {
        let ask = &self.ask[..8];
        let mut lines = self.learn.lines();
        lines.push(format!(
            "celis: the ask {ask} is done. When you have kept what it taught, celis stats {ask} gives its total."
        ));
        lines.push(self.total());
        lines
    }

    /// Its total line: the steps, each kind of proof that is not zero, or `all` of the one kind, the
    /// patterns used and extracted, then the minutes.
    pub fn total(&self) -> String {
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
        parts.push(format!(
            "{} used",
            plural(self.learn.patterns.len(), "pattern")
        ));
        parts.push(format!("{} extracted", self.extracted));
        parts.push(format!("{}m", self.minutes));
        parts.join(" · ")
    }
}
