//! The review `celis step done` prints when a step closes.

use super::Command;
use crate::failure::Failure;
use crate::progress::{Gap, Review};
use celis_trails::records::id::short;
use std::io::Write;

impl Command {
    /// Prints `review` on `err`: the step's plan against what it did, the open steps beside it, an ask's
    /// gap, then the next move.
    pub(crate) fn review(&self, review: &Review, err: &mut dyn Write) -> Result<(), Failure> {
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
        if let Some(gap) = &review.gap {
            lines.extend(gap.edited.iter().map(|e| format!("  edited  {e}")));
            for (id, title) in &gap.claims {
                lines.push(format!("  claim   {} {title}", short(id)));
            }
            if !gap.places.is_empty() {
                lines.push(format!("  places  {}", gap.places.join(" · ")));
            }
            lines.push(total(gap, review.changed.len() + gap.edited.len()));
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

/// The total line of `gap` with its `turns`: each count that is not zero, then the minutes.
fn total(gap: &Gap, turns: usize) -> String {
    let counts = [
        (gap.steps, "step", "steps"),
        (turns, "turn", "turns"),
        (gap.checked, "checked", "checked"),
        (gap.judged, "judged", "judged"),
        (gap.person, "person", "person"),
        (gap.claim, "claim", "claims"),
    ];
    let mut parts = vec!["celis".to_string()];
    parts.extend(
        counts
            .iter()
            .filter(|(n, ..)| *n > 0)
            .map(|(n, one, many)| format!("{n} {}", if *n == 1 { one } else { many })),
    );
    parts.push(format!("{}m", gap.minutes));
    parts.join(" · ")
}
