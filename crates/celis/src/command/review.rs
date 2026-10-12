//! The review `celis step done` prints when a step closes.

use super::Command;
use celis_trails::id::short;
use celis_trails::{Failure, Review};
use std::io::Write;

impl Command {
    /// Prints `review` on `err`: the step's plan against what it did, the open steps beside it, then the
    /// next move.
    pub(super) fn review(&self, review: &Review, err: &mut dyn Write) -> Result<(), Failure> {
        let mut lines = vec![format!(
            "celis: {} is done; compare the result with its plan:",
            short(&review.step)
        )];
        lines.extend(review.expect.iter().map(|e| format!("  expect  {e}")));
        let parts = review.planned + review.added.len();
        if parts > 0 {
            lines.push(format!("  planned {} steps", review.planned));
        }
        for (id, title) in &review.added {
            lines.push(format!("  added   {} {title}", short(id)));
        }
        for (id, title, reason) in &review.dropped {
            lines.push(format!("  dropped {} {title} · {reason}", short(id)));
        }
        for (id, text) in &review.changed {
            lines.push(format!("  changed {} {text}", short(id)));
        }
        for text in &review.asked {
            lines.push(format!("  asked   {text}"));
        }
        for (id, title) in &review.open {
            lines.push(format!("  open    {} {title}", short(id)));
        }
        let under = review.under.as_deref().map(short);
        lines.extend(match (review.ask, review.as_planned, under) {
            (true, _, _) => Some("celis: write what a later plan should keep as rules with celis decide.".to_string()),
            (_, false, Some(u)) => Some(format!("celis: it went otherwise than planned; replace or drop each open step it voids, then add one step under {u}.")),
            (_, true, Some(u)) => Some(format!("celis: take an open step beside it, add one step under {u}, or close {u} when its work is done.")),
            (_, _, None) => None,
        });
        lines.iter().try_for_each(|line| super::put(err, line))
    }
}
