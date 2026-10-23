//! The review `celis step done` prints when a step closes.

use super::Command;
use crate::failure::Failure;
use crate::progress::{Gap, Proof, Review};
use celis_trails::records::id::short;
use celis_trails::{ProofFile, Trails};
use std::io::Write;

impl Command {
    /// Prints `review` on `err`: the step's plan against what it did, the open steps beside it, the blobs
    /// of `trails` for its files and the files beside it, an ask's gap, then the next move; with no
    /// `linked` folder, its blobs are local only.
    pub(crate) fn review(
        &self,
        review: &Review,
        trails: &Trails,
        linked: bool,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let kept = |file: &ProofFile| crate::text::line::blob(file, trails);
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
        let local = if linked { "" } else { " · local only" };
        for file in &review.files {
            lines.push(format!("  file    {} · {}{local}", file.place, kept(file)));
        }
        for (id, title, file) in &review.beside {
            lines.push(format!("  beside  {} {title} · {}", short(id), kept(file)));
        }
        if let Some(gap) = &review.gap {
            lines.extend(gap.edited.iter().map(|e| format!("  edited  {e}")));
            for (id, title, _) in gap.proofs.iter().filter(|(.., p)| *p == Proof::Claim) {
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
    let steps = gap.proofs.len();
    let mut parts = vec!["celis".to_string()];
    let step = if steps == 1 { "step" } else { "steps" };
    let turn = if turns == 1 { "turn" } else { "turns" };
    parts.extend(
        [(steps, step), (turns, turn)]
            .iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, w)| format!("{n} {w}")),
    );
    for proof in Proof::ALL {
        let n = gap.count(proof);
        if n > 0 {
            parts.push(format!("{n} {}", proof.word(n)));
        }
    }
    parts.push(format!("{}m", gap.minutes));
    parts.join(" · ")
}
