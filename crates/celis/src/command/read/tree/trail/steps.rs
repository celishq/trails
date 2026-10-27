//! The steps of a level of the tree: each with its branch, its details and its own level under it.

use crate::command::read::tree::level::Level;
use crate::command::read::tree::row::Row;
use crate::command::read::tree::{after, Item, Tree, NOTES, PATTERNS};
use crate::progress::Phase;
use crate::terminal::canvas::{paint, Styled, DIM, MAGENTA};
use crate::terminal::look::{look, SHIMMER};
use celis_nostr::Note;
use celis_trails::step::steps::LIFE;
use celis_trails::Step;
use ratatui::style::Style;
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds `level` under the line `above` after `rail`: its steps in the order they came, closed ones
    /// too; then the patterns line of `patterns` and the notes line of `notes`.
    pub(super) fn level(
        &self,
        level: Level,
        patterns: Vec<&Note>,
        notes: Vec<&Note>,
        rail: &Styled,
        above: &str,
        lines: &mut Vec<Row>,
    ) {
        let more = !patterns.is_empty() || !notes.is_empty();
        self.steps(level.order, rail, above, more, lines);
        if !patterns.is_empty() {
            let last = notes.is_empty();
            self.records_line(
                patterns,
                (paint(MAGENTA, PATTERNS), "pattern"),
                rail,
                last,
                above,
                lines,
            );
        }
        if !notes.is_empty() {
            self.records_line(notes, (paint(DIM, NOTES), "note"), rail, true, above, lines);
        }
    }

    /// Adds the steps `notes` under the line `above` after `rail`, the last of their level unless `more`:
    /// each its mark and title wrapped, done/all of the steps under it at the right, and when opened the
    /// tree line down from its mark over its steps one level down; with full, the lines
    /// `--full` prints under it.
    pub(in crate::command::read::tree) fn steps(
        &self,
        notes: Vec<Step>,
        rail: &Styled,
        above: &str,
        more: bool,
        lines: &mut Vec<Row>,
    ) {
        let all = notes.len();
        let colour = self.lane();
        for (n, note) in notes.into_iter().enumerate() {
            let step = note.record();
            let state = Phase::of(&note);
            if state.active() {
                self.until(note.trail().newest() + LIFE);
            }
            let open = self.open(note.id());
            let last = n + 1 == all && !more;
            let level = Level::of(note.children());
            let notes = self.notes.on(note.id()).to_vec();
            let opens = !note.children().is_empty() || !notes.is_empty();
            let toggle = opens.then_some(open);
            let (lead, rail) = self.branch(rail, last, toggle, colour);
            let first = lines.len();
            let tone = match self.busy(note) {
                true => look(Phase::Started).title,
                false => look(state).title,
            };
            lines.push(Row::Lined {
                lead,
                marks: self.marked(note),
                title: step.title().to_string(),
                tone: self.shone(note, self.tone(tone, note.newest())),
                meta: Styled::from(paint(DIM, &self.meta(note).to_string())),
                whole: Some(match open && opens {
                    true => after(&rail, paint(self.lane(), "│")),
                    false => rail.clone(),
                }),
            });
            let item = Item {
                id: note.id().to_string(),
                open,
                parent: Some(above.to_string()),
                under: level.ids(),
            };
            self.pick(item, lines, first);
            if open && self.view.full {
                let below = after(&rail, Span::raw("  "));
                self.closing(note, &below, true, lines);
            }
            if open {
                self.level(level, self.uses(note), notes, &rail, note.id(), lines);
            }
        }
    }

    /// The lead of a line of a level after `rail`: `├`, or `└` when it is the `last`, in the look
    /// `colour` of the line it hangs from, then `── `, or ` ▸ ` or ` ▾ ` as `open` says when it opens,
    /// the arrow where the line's last dash stands; and the rail of the level under it, whose corners
    /// stand under this line's mark.
    pub(super) fn branch(
        &self,
        rail: &Styled,
        last: bool,
        open: Option<bool>,
        colour: Style,
    ) -> (Styled, Styled) {
        let (corner, under) = match last {
            true => ("└", "    "),
            false => ("├", "│   "),
        };
        let mut lead = after(rail, paint(colour, corner));
        match open {
            None => lead.push_span(paint(colour, "──")),
            Some(false) => lead.push_span(paint(DIM, " ▸")),
            Some(true) => lead.push_span(paint(DIM, " ▾")),
        }
        lead.push_span(" ");
        (lead, after(rail, paint(colour, under)))
    }

    /// The look `tone` of the title of `note`, marked to shimmer in a watch while it works with no step
    /// under it.
    fn shone(&self, note: Step, tone: Style) -> Style {
        match self.view.spin && Phase::of(&note) == Phase::Started && note.children().is_empty() {
            true => {
                self.spun.set(true);
                tone.add_modifier(SHIMMER)
            }
            false => tone,
        }
    }

    /// Whether `note` or a step under it works.
    pub(in crate::command::read::tree) fn busy(&self, note: Step) -> bool {
        note.tree().iter().any(|s| Phase::of(s) == Phase::Started)
    }

    /// Whether the step or pattern `id` shows what `--full` shows: with `--full`, unless the watch closed
    /// it; else when the watch opened it.
    pub(super) fn open(&self, id: &str) -> bool {
        self.view.full != self.view.flipped.contains(id)
    }
}
