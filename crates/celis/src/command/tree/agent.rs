//! The agent view of `celis tree`: every trail, rule and step with its id, plain and whole.

use super::{Line, Summary, Tree};
use crate::command::clock::took;
use celis_trails::id::short;
use celis_trails::{Step, Trail};

impl Tree<'_> {
    /// Adds `note` as a block: its mark, id and title, then two spaces in its progress and worked time,
    /// its own lines, its rules, then its steps.
    pub(super) fn plain(&self, note: Step, lines: &mut Vec<Line>) {
        let work = self.work(note);
        let summary = self.summary(note, work);
        let heading = match self.grouped && note.state().active() {
            true => format!("{} · {}", self.heading(note), note.record().anchor.name),
            false => self.heading(note),
        };
        lines.push(Line::Text(heading));
        let progress = progress(&summary);
        if !progress.is_empty() {
            lines.push(Line::Text(format!("  {progress}")));
        }
        self.below(work, "  ", lines);
        for (rule, decision) in &summary.rules {
            let line = format!("  § {} {}", short(&rule.id), decision.content);
            lines.push(Line::Text(line));
        }
        self.steps(Trail::new(note).parts(), "  ", lines);
    }

    /// Adds `note` as one line: its mark, id and title, then its progress and worked time.
    pub(super) fn plain_line(&self, note: Step, lines: &mut Vec<Line>) {
        let summary = self.summary(note, self.work(note));
        let line = match progress(&summary) {
            progress if progress.is_empty() => self.heading(note),
            progress => format!("{} · {progress}", self.heading(note)),
        };
        lines.push(Line::Text(line));
    }

    /// Adds the steps `notes` after `indent`, each as its mark, id and title, its first place and
    /// worked time, then its lines and its steps two spaces further in.
    fn steps(&self, notes: Vec<Step>, indent: &str, lines: &mut Vec<Line>) {
        for note in notes {
            let mut line = format!("{indent}{}", self.heading(note));
            if let Some(at) = note.record().at() {
                line.push_str(&format!(" · {at}"));
            }
            if let Some(worked) = self.worked(&note.tree()) {
                line.push_str(&format!(" · {}", took(worked)));
            }
            lines.push(Line::Text(line));
            let under = format!("{indent}  ");
            self.below(note, &under, lines);
            self.steps(note.children(), &under, lines);
        }
    }

    /// Adds after `indent` the lines of `note`: with `--full` all `--full` prints, else what changed
    /// when done, or why when dropped.
    fn below(&self, note: Step, indent: &str, lines: &mut Vec<Line>) {
        if self.view.full {
            self.closing(note, indent, true, lines);
            self.reused(note, indent, lines);
            return;
        }
        if let Some(changed) = note.done().and_then(|d| d.tag("changed")) {
            lines.push(Line::Text(format!("{indent}~ {changed}")));
        }
        if let Some(reason) = note.dropped() {
            lines.push(Line::Text(format!("{indent}dropped {}", reason.content)));
        }
    }

    /// The mark, id and title of `note`.
    fn heading(&self, note: Step) -> String {
        let title = note.record().title();
        format!("{} {} {title}", self.mark(note.state()), short(note.id()))
    }
}

/// The progress and worked time of a trail, as `1/3 steps · 1m 30s worked`.
fn progress(summary: &Summary) -> String {
    let mut parts = vec![];
    if summary.all > 0 {
        let word = if summary.all == 1 { "step" } else { "steps" };
        parts.push(format!("{}/{} {word}", summary.done, summary.all));
    }
    parts.extend(summary.worked.map(|w| format!("{} worked", took(w))));
    parts.join(" · ")
}
