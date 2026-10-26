//! The agent view of `celis tree`: every trail, note and step with its id, plain and whole.

use super::{Level, Row, Styled, Summary, Tree};
use crate::progress::Flags;
use crate::terminal::classic::signs;
use crate::text::clock::took;
use celis_store::records::id::short;
use celis_trails::{Step, Trail};

impl Tree<'_> {
    /// Adds `note` as a block: its mark, id and title, then two spaces in its progress and worked time,
    /// its own lines, its active steps, its notes, then its other steps.
    pub(super) fn agent_block(&self, note: Step, lines: &mut Vec<Row>) {
        let work = self.work(note);
        let summary = self.summary(note, work);
        let heading = match self.grouped && note.state().active() {
            true => format!("{} · {}", self.heading(note), note.record().anchor.name),
            false => self.heading(note),
        };
        lines.push(Row::Text(heading.into()));
        let progress = progress(&summary);
        if !progress.is_empty() {
            lines.push(Row::Text(format!("  {progress}").into()));
        }
        self.below(work, "  ", lines);
        let level = Level::of(Trail::new(note).parts());
        self.agent_steps(level.active, "  ", lines);
        for pattern in &summary.notes {
            let line = format!("  § {} {}", short(&pattern.id), pattern.content);
            lines.push(Row::Text(line.into()));
        }
        self.agent_steps(level.ended, "  ", lines);
    }

    /// Adds `note` as one line: its mark, id and title, then its progress and worked time.
    pub(super) fn agent_line(&self, note: Step, lines: &mut Vec<Row>) {
        let summary = self.summary(note, self.work(note));
        let line = match progress(&summary) {
            progress if progress.is_empty() => self.heading(note),
            progress => format!("{} · {progress}", self.heading(note)),
        };
        lines.push(Row::Text(line.into()));
    }

    /// Adds the steps `notes` after `indent`, each as its mark, id and title, its first place and
    /// worked time, then its lines and its level two spaces further in.
    fn agent_steps(&self, notes: Vec<Step>, indent: &str, lines: &mut Vec<Row>) {
        for note in notes {
            let mut line = format!("{indent}{}", self.heading(note));
            if let Some(at) = note.record().at() {
                line.push_str(&format!(" · {at}"));
            }
            if let Some(worked) = self.worked(&note.tree()) {
                line.push_str(&format!(" · {}", took(worked)));
            }
            lines.push(Row::Text(line.into()));
            let under = format!("{indent}  ");
            self.below(note, &under, lines);
            let level = Level::of(note.children());
            self.agent_steps(level.active, &under, lines);
            self.agent_steps(level.ended, &under, lines);
        }
    }

    /// Adds after `indent` the lines of `note`: with `--full` all `--full` prints, else what changed
    /// when done, or why when dropped.
    fn below(&self, note: Step, indent: &str, lines: &mut Vec<Row>) {
        if self.view.full {
            let indent = Styled::from(indent.to_string());
            self.closing(note, &indent, true, lines);
            self.followed(note, &indent, lines);
            return;
        }
        if let Some(changed) = note.changed() {
            lines.push(Row::Text(format!("{indent}~ {changed}").into()));
        }
        if let Some(reason) = note.dropped() {
            lines.push(Row::Text(
                format!("{indent}dropped {}", reason.content).into(),
            ));
        }
    }

    /// The mark and flags, id and title of `note`.
    fn heading(&self, note: Step) -> String {
        let title = note.record().title();
        let flags: String = signs(Flags::of(note)).into_iter().map(|(s, _)| s).collect();
        format!(
            "{}{flags} {} {title}",
            self.mark(note.state()).content,
            short(note.id())
        )
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
