//! The total of an ask when it closes.

use super::Learn;
use crate::text::line::plural;
use celis_nostr::Note;
use celis_patterns::Patterns;
use celis_store::StepId;
use celis_trails::{Done, Step};

/// The total of an ask when it closes: its done steps, the patterns written in its project since it
/// began, and what it taught.
pub(crate) struct End {
    pub ask: StepId,
    pub steps: usize,
    pub extracted: usize,
    pub learn: Learn,
}

impl End {
    /// The end of the ask `step`, closed `done`, from `records`.
    pub fn of(records: &[Note], step: Step, done: &Done) -> End {
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
            steps: closed.len(),
            extracted: patterns
                .every()
                .iter()
                .filter(|p| &p.record.anchor.project == project && p.note.created_at >= from)
                .count(),
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

    /// Its total line: the steps, then the patterns used and extracted, each pattern part left out
    /// when it is zero.
    pub fn total(&self) -> String {
        let steps = plural(self.steps, "step");
        let used = self.learn.patterns.len();
        let patterns: Vec<String> = [
            (used > 0).then(|| format!("{} used", plural(used, "pattern"))),
            (self.extracted > 0).then(|| format!("{} extracted", self.extracted)),
        ]
        .into_iter()
        .flatten()
        .collect();
        let mut parts = vec!["celis".to_string(), steps];
        if !patterns.is_empty() {
            parts.push(patterns.join(", "));
        }
        parts.join(" · ")
    }
}
