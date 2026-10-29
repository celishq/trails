//! What `celis step done` prints when a step closes: its review, and on an ask its end.

mod end;
mod learn;
mod review;

use crate::command::Command;
use crate::failure::Failure;
use celis_store::records::id::short;
use celis_store::{ProofFile, Store};
pub(crate) use end::End;
pub(crate) use learn::Learn;
pub(crate) use review::Review;
use std::io::Write;

impl Command {
    /// Prints `review` on `err`: the step's plan against what it did, the open steps beside it, the blobs
    /// of `trails` for its files and the files beside it, then the next move; on an ask, only its end;
    /// with no `linked` folder, local blobs.
    pub(crate) fn review(
        &self,
        review: &Review,
        trails: &Store,
        linked: bool,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        if let Some(end) = &review.end {
            return end
                .lines()
                .iter()
                .try_for_each(|line| crate::command::put(err, line));
        }
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
        lines.extend(review.learn.iter().flat_map(Learn::lines));
        let under = review.under.as_deref().map(short);
        let close = |u: &str| match review.ask {
            true => format!(
                "learn from it with celis extract {u} and close it with celis step done {u}"
            ),
            false => format!("close it with celis step done {u}"),
        };
        lines.extend(match (review.as_planned, under) {
            (false, Some(u)) if review.open.is_empty() => Some(format!("celis: it went otherwise than planned, and no step under {u} is open; when its work is done, {}, else add one step under it.", close(&u))),
            (false, Some(u)) => Some(format!("celis: it went otherwise than planned; replace or drop each open step it voids, then add one step under {u}.")),
            (true, Some(u)) if review.open.is_empty() => Some(format!("celis: no step under {u} is open; when its work is done, {}, else add one step under it.", close(&u))),
            (true, Some(u)) => Some(format!("celis: take an open step beside it, add one step under {u}, or close {u} when its work is done.")),
            (_, None) => None,
        });
        lines
            .iter()
            .try_for_each(|line| crate::command::put(err, line))
    }
}
