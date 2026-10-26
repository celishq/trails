//! What `celis step done` prints when a step closes: its review, and on an ask its end.

mod end;
mod proof;
mod review;

use crate::command::Command;
use crate::failure::Failure;
use celis_trails::records::id::short;
use celis_trails::{ProofFile, Step, Trails};
pub(crate) use end::End;
pub(crate) use proof::Proof;
pub(crate) use review::Review;
use std::io::Write;

impl Command {
    /// Prints `review` on `err`: the step's plan against what it did, the open steps beside it, the blobs
    /// of `trails` for its files and the files beside it, the patterns of the `changed` files, then the
    /// next move; on an ask, only its end; with no `linked` folder, its blobs are local only.
    pub(crate) fn review(
        &self,
        review: &Review,
        (trails, changed): (&Trails, &[String]),
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
        lines.extend(self.changed_patterns(trails, changed));
        let steps = trails.steps();
        let ask = review.under.as_deref().and_then(|id| steps.get(id));
        if let Some(ask) =
            ask.filter(|a| a.is_judged() && review.as_planned && review.open.is_empty())
        {
            lines.extend(self.brief(ask, trails, changed));
        }
        let under = review.under.as_deref().map(short);
        lines.extend(match (review.as_planned, under) {
            (false, Some(u)) => Some(format!("celis: it went otherwise than planned; replace or drop each open step it voids, then add one step under {u}.")),
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

    /// The lines of the patterns that the `changed` files match, after a line that asks to check the
    /// change against each; none when none matches.
    fn changed_patterns(&self, trails: &Trails, changed: &[String]) -> Vec<String> {
        let project = self.project();
        let paths: Vec<String> = changed
            .iter()
            .flat_map(|f| crate::command::record::files::paths(&project, &self.folder, f))
            .collect();
        let patterns = trails
            .patterns()
            .fitting(&project.anchor.project, &paths, |_| false);
        if patterns.is_empty() {
            return vec![];
        }
        let head = "celis: check the change against each pattern of the files git shows changed:";
        let lines = patterns
            .iter()
            .map(|pattern| crate::text::line::pattern(pattern, trails));
        std::iter::once(head.to_string()).chain(lines).collect()
    }

    /// What `done` of the ask `ask` with no `--by` prints before it refuses: the patterns of the `changed`
    /// files and the judge brief; none with no changed file.
    pub(crate) fn unjudged(&self, ask: Step, trails: &Trails, changed: &[String]) -> Vec<String> {
        if changed.is_empty() {
            return vec![];
        }
        let mut lines = self.changed_patterns(trails, changed);
        lines.extend(self.brief(ask, trails, changed));
        lines
    }

    /// The judge brief of `ask`, an ask or a step with steps under it: its title, its intent lines, the
    /// patterns that fit its title and the changed files, the notes of its trail, the changed files, and
    /// its check.
    fn brief(&self, ask: Step, trails: &Trails, files: &[String]) -> Vec<String> {
        let record = ask.record();
        let mut lines = vec![
            format!(
                "celis: before you close {} --by judge, run the celis:judge agent with this brief and \
                 where the result is:",
                short(ask.id())
            ),
            format!(
                "  {:<7} {}",
                if ask.is_ask() { "ask" } else { "step" },
                ask.title()
            ),
        ];
        for line in record.detail() {
            lines.push(match line.split_once(": ") {
                Some((key, value)) => format!("  {key:<7} {value}"),
                None => format!("          {line}"),
            });
        }
        let fits = self.fits(trails, ask.title(), files, &mut std::io::sink());
        for pattern in fits.unwrap_or_default() {
            let kind = if pattern.record.binds {
                "rule"
            } else {
                "pattern"
            };
            let text = crate::text::line::pattern(&pattern, trails);
            let mut pattern = text.lines();
            lines.extend(pattern.next().map(|first| format!("  {kind:<7} {first}")));
            lines.extend(pattern.map(|line| format!("        {line}")));
        }
        for note in trails.notes().of(ask.trail()) {
            let mut text = note.content.lines();
            lines.extend(text.next().map(|first| format!("  note    {first}")));
            lines.extend(text.map(|line| format!("        {line}")));
        }
        lines.extend(files.iter().map(|file| format!("  file    {file}")));
        lines.extend(
            record
                .check
                .iter()
                .map(|check| format!("  check   {check}")),
        );
        lines
    }
}
