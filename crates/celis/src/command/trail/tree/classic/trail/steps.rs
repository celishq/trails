//! The steps of a level of the tree: each with its branch, its details and its own level under it.

use crate::command::trail::tree::classic::level::Level;
use crate::command::trail::tree::classic::row::Row;
use crate::command::trail::tree::classic::{after, Item, Tree, NOTES, PATTERNS};
use crate::command::trail::tree::meta::Meta;
use crate::progress::Phase;
use crate::terminal::canvas::{paint, Styled, DIM};
use crate::terminal::classic::{look, MAGENTA};
use celis_nostr::Note;
use celis_trails::step::steps::LIFE;
use celis_trails::{State, Step};
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds `level` under the line `above` after `rail`: its active steps, the patterns line of `patterns`,
    /// the notes line of `notes`, then its closed line.
    pub(super) fn level(
        &self,
        level: Level,
        patterns: Vec<&Note>,
        notes: Vec<&Note>,
        rail: &Styled,
        above: &str,
        lines: &mut Vec<Row>,
    ) {
        let more = !patterns.is_empty() || !notes.is_empty() || !level.ended.is_empty();
        self.steps(level.active, rail, above, more, lines);
        if !patterns.is_empty() {
            let last = notes.is_empty() && level.ended.is_empty();
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
            let last = level.ended.is_empty();
            self.records_line(notes, (paint(DIM, NOTES), "note"), rail, last, above, lines);
        }
        if !level.ended.is_empty() {
            self.closed_line(level.ended, rail, above, lines);
        }
    }

    /// Adds the steps `notes` under the line `above` after `rail`, the last of their level unless `more`,
    /// each followed by its own one level down, each with its title wrapped and its details under it; an
    /// open one with the lines `--full` prints under it, a closed done one with no time and its steps
    /// folded.
    pub(in crate::command::trail::tree::classic) fn steps(
        &self,
        notes: Vec<Step>,
        rail: &Styled,
        above: &str,
        more: bool,
        lines: &mut Vec<Row>,
    ) {
        let all = notes.len();
        for (n, note) in notes.into_iter().enumerate() {
            let step = note.record();
            let state = Phase::of(&note);
            if note.active() {
                self.until(note.trail().newest() + LIFE);
            }
            let open = self.open(note.id());
            let last = n + 1 == all && !more;
            let (lead, rail) = self.branch(rail, last, self.opens(note).then_some(open));
            let size = note.tree().len() - 1;
            let level = Level::of(note.children());
            let folded = note.state() == State::Done && size > 0 && !open;
            let below = match note.children().is_empty() || folded {
                true => after(&rail, Span::raw("  ")),
                false => after(&rail, paint(DIM, "│ ")),
            };
            let first = lines.len();
            lines.push(Row::Titled {
                lead,
                marks: self.marked(note),
                title: step.title().to_string(),
                tone: self.tone(look(state).title, note.newest()),
                rest: below.clone(),
            });
            let meta = Meta {
                done: if folded { size } else { 0 },
                all: if folded { size } else { 0 },
                worked: match note.state() != State::Open {
                    true => None,
                    false => self.worked(&note.tree()),
                },
            };
            self.details(&below, meta, lines);
            let item = Item {
                id: note.id().to_string(),
                open,
                parent: Some(above.to_string()),
                under: level.ids(note.id()),
            };
            self.pick(item, lines, first);
            if open {
                self.closing(note, &below, true, lines);
            }
            if !folded {
                let (used, notes) = match open {
                    true => (self.uses(note), self.notes.on(note.id()).to_vec()),
                    false => (vec![], vec![]),
                };
                self.level(level, used, notes, &rail, note.id(), lines);
            }
        }
    }

    /// Adds `meta` after `rail` as one dim line; none when it is empty.
    pub(super) fn details(&self, rail: &Styled, meta: Meta, lines: &mut Vec<Row>) {
        let text = meta.to_string();
        if !text.is_empty() {
            lines.push(Row::Note {
                rail: rail.clone(),
                label: String::new(),
                text,
                tone: DIM,
            });
        }
    }

    /// The lead of a line of a level after `rail`, `├─ `, or `└─ ` when it is the `last`, its `─` a `▸`
    /// or `▾` as `open` says when it opens, and the rail of the level under it.
    pub(super) fn branch(&self, rail: &Styled, last: bool, open: Option<bool>) -> (Styled, Styled) {
        let (corner, under) = match last {
            true => ('└', "   "),
            false => ('├', "│  "),
        };
        let chevron = match open {
            None => '─',
            Some(false) => '▸',
            Some(true) => '▾',
        };
        (
            after(rail, paint(DIM, &format!("{corner}{chevron} "))),
            after(rail, paint(DIM, under)),
        )
    }

    /// Whether opening `note` shows something: lines under it, its notes, or steps under a closed one.
    fn opens(&self, note: Step) -> bool {
        let lines = note.record().detail().next().is_some()
            || note.done_record().is_some()
            || note.dropped().is_some()
            || !self.notes.on(note.id()).is_empty();
        lines || (note.state() != State::Open && !note.children().is_empty())
    }

    /// Whether the step or pattern `id` shows what `--full` shows: with `--full`, unless the watch closed
    /// it; else when the watch opened it.
    pub(super) fn open(&self, id: &str) -> bool {
        self.view.full != self.view.flipped.contains(id)
    }
}
