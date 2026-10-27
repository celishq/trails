//! The agent view of `celis tree`: every trail, note and step with its id, plain and whole.

use super::{Level, Row, Styled, Tree};
use crate::progress::{Flags, Phase};
use crate::terminal::look::{signs, SLOT};
use celis_nostr::Note;
use celis_store::records::id::short;
use celis_trails::{Step, Trail};

impl Tree<'_> {
    /// Adds `note` as a block: its mark, id and title, then two spaces in its steps and time,
    /// its own lines, its active steps, its notes, then its other steps.
    pub(super) fn agent_block(&self, note: Step, lines: &mut Vec<Row>) {
        let work = self.work(note);
        let summary = self.summary(note, work);
        let heading = match self.grouped && note.active() {
            true => format!("{} · {}", self.heading(note), note.record().anchor.name),
            false => self.heading(note),
        };
        lines.push(Row::Text(heading.into()));
        let progress = summary.meta.to_string();
        if !progress.is_empty() {
            lines.push(Row::Text(format!("  {progress}").into()));
        }
        self.below(work, "  ", lines);
        let level = Level::of(Trail::new(note).parts());
        self.agent_steps(level.active, "  ", lines);
        for pattern in summary.notes {
            lines.push(pattern_line("  ", pattern));
        }
        self.agent_steps(level.ended, "  ", lines);
    }

    /// Adds `note` as one line: its mark, id and title, then its steps and time.
    pub(super) fn agent_line(&self, note: Step, lines: &mut Vec<Row>) {
        let summary = self.summary(note, self.work(note));
        let line = match summary.meta.to_string() {
            progress if progress.is_empty() => self.heading(note),
            progress => format!("{} · {progress}", self.heading(note)),
        };
        lines.push(Row::Text(line.into()));
    }

    /// Adds the steps `notes` after `indent`, each as its mark, id and title, its first place, its steps
    /// and time, then its lines and its level two spaces further in.
    fn agent_steps(&self, notes: Vec<Step>, indent: &str, lines: &mut Vec<Row>) {
        for note in notes {
            let mut line = format!("{indent}{}", self.heading(note));
            if let Some(at) = note.record().at() {
                line.push_str(&format!(" · {at}"));
            }
            let meta = self.meta(note);
            if !meta.to_string().is_empty() {
                line.push_str(&format!(" · {meta}"));
            }
            lines.push(Row::Text(line.into()));
            let under = format!("{indent}  ");
            self.below(note, &under, lines);
            if self.view.full {
                for pattern in self.uses(note) {
                    lines.push(pattern_line(&under, pattern));
                }
            }
            let level = Level::of(note.children());
            self.agent_steps(level.active, &under, lines);
            self.agent_steps(level.ended, &under, lines);
        }
    }

    /// Adds after `indent` and under the title the lines of `note`: with `--full` all `--full` prints,
    /// else what changed when done, or why when dropped.
    fn below(&self, note: Step, indent: &str, lines: &mut Vec<Row>) {
        if self.view.full {
            let indent = Styled::from(indent.to_string());
            self.closing(note, &indent, true, lines);
            return;
        }
        let indent = format!("{indent}{}", " ".repeat(SLOT - 1));
        if let Some(changed) = note.changed() {
            lines.push(Row::Text(format!("{indent}~ {changed}").into()));
        }
        if let Some(reason) = note.dropped() {
            lines.push(Row::Text(
                format!("{indent}dropped {}", reason.content).into(),
            ));
        }
    }

    /// The mark and flags of `note` in one slot, its id and title.
    fn heading(&self, note: Step) -> String {
        let title = note.record().title();
        let flags = signs(Flags::of(note)).into_iter().map(|(s, _)| s);
        let marks: String = std::iter::once(self.mark(Phase::of(&note)).content)
            .chain(flags.map(Into::into))
            .collect();
        format!("{marks:<SLOT$} {} {title}", short(note.id()))
    }
}

/// The line of `pattern` after `indent`: `§` in its slot, its id, and its first line.
fn pattern_line(indent: &str, pattern: &Note) -> Row {
    let when = pattern.content.lines().next().unwrap_or_default();
    let line = format!("{indent}{:<SLOT$} {} {when}", "§", short(&pattern.id));
    Row::Text(line.into())
}
