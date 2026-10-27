//! What `celis step done` prints when a step closes: its review, and on an ask its end.

mod end;
mod learn;
mod review;

use crate::command::Command;
use crate::failure::Failure;
use celis_patterns::{Pattern, PatternStore};
use celis_store::records::id::short;
use celis_store::{ProofFile, Store};
pub(crate) use end::End;
pub(crate) use learn::Learn;
pub(crate) use review::Review;
use std::io::Write;

impl Command {
    /// Prints `review` on `err`: the step's plan against what it did, the open steps beside it, the blobs
    /// of `trails` for its files and the files beside it, the patterns of the `changed` files, then the
    /// next move; on an ask, only its end; with no `linked` folder, local blobs.
    pub(crate) fn review(
        &self,
        review: &Review,
        (trails, changed): (&Store, &[String]),
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
        lines.extend(self.changed_patterns(trails, changed));
        lines.extend(review.learn.iter().flat_map(Learn::lines));
        let under = review.under.as_deref().map(short);
        lines.extend(match (review.as_planned, under) {
            (false, Some(u)) => Some(format!("celis: it went otherwise than planned; replace or drop each open step it voids, then add one step under {u}.")),
            (true, Some(u)) if review.open.is_empty() => Some(format!("celis: no step under {u} is open; when its work is done, close it with celis step done {u}, else add one step under it.")),
            (true, Some(u)) => Some(format!("celis: take an open step beside it, add one step under {u}, or close {u} when its work is done.")),
            (_, None) => None,
        });
        lines
            .iter()
            .try_for_each(|line| crate::command::put(err, line))
    }

    /// The changed files of a `done` that names `files`: those git shows changed or new, and `files`.
    pub(crate) fn changed(&self, files: &[String]) -> Vec<String> {
        let mut changed = crate::command::record::files::changed(&self.folder);
        for file in files {
            if !changed.contains(file) {
                changed.push(file.clone());
            }
        }
        changed
    }

    /// The live patterns in scope that the `files` match, as `Patterns::matching` orders them.
    fn matched<'a>(&self, trails: &'a Store, files: &[String]) -> Vec<Pattern<'a>> {
        let project = self.project();
        let paths: Vec<String> = files
            .iter()
            .flat_map(|f| crate::command::record::files::paths(&project, &self.folder, f))
            .collect();
        trails
            .patterns()
            .view(&project.anchor.project)
            .matching(&paths)
    }

    /// The lines of the patterns that the `changed` files match, after a line that asks to check the
    /// change against each; none when none matches.
    fn changed_patterns(&self, trails: &Store, changed: &[String]) -> Vec<String> {
        let patterns = self.matched(trails, changed);
        if patterns.is_empty() {
            return vec![];
        }
        let head = "celis: check the change against each pattern of the files git shows changed:";
        let lines = patterns
            .iter()
            .map(|pattern| crate::text::line::pattern(pattern, trails));
        std::iter::once(head.to_string()).chain(lines).collect()
    }
}
