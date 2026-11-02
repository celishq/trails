//! The total of an ask when it closes.

use super::Learn;
use crate::text::line::plural;
use celis_nostr::Note;
use celis_patterns::Patterns;
use celis_store::StepId;
use celis_trails::{Done, Step};
use std::collections::HashSet;

/// The total of an ask when it closes: the patterns it used, the patterns written in its project since
/// it began, the patterns that misled it, and what it taught.
pub(crate) struct End {
    pub ask: StepId,
    pub extracted: usize,
    pub misled: usize,
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
        let misled: HashSet<&String> = closed.iter().flat_map(|(_, d)| &d.misled).collect();
        End {
            ask: step.step_id(),
            extracted: patterns
                .every()
                .iter()
                .filter(|p| &p.record.anchor.project == project && p.note.created_at >= from)
                .count(),
            misled: misled.len(),
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

    /// Its total line: how many of the person's patterns it used and how many it wrote, each left out
    /// when it is zero, then how many misled when any did.
    pub fn total(&self) -> String {
        let used = self.learn.patterns.len();
        let learned = self.extracted;
        let mut line = match (used, learned) {
            (0, 0) => "celis had no pattern for this ask.".to_string(),
            (0, n) => format!("celis learned {}.", plural(n, "new pattern")),
            (u, 0) => format!("celis used {u} of your patterns."),
            (u, n) => format!("celis used {u} of your patterns and learned {n}."),
        };
        if self.misled > 0 {
            line.push_str(&format!(" {} led the work wrong.", self.misled));
        }
        line
    }
}
