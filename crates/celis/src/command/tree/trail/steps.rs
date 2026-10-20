//! The steps of a level of the tree: each with its branch, its details and its own level under it.

use crate::command::tree::level::Level;
use crate::command::tree::row::Row;
use crate::command::tree::{after, Item, RuleNote, Tree};
use crate::terminal::canvas::{paint, Styled, DIM};
use crate::terminal::look::{lane, look, SHIMMER};
use celis_trails::step::steps::LIFE;
use celis_trails::{State, Step};
use ratatui::style::Style;
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds `level` under the line `above` after `rail`: its active steps, the rules line of `rules`,
    /// then its closed line.
    pub(super) fn level(
        &self,
        level: Level,
        rules: Vec<RuleNote>,
        rail: &Styled,
        above: &str,
        lines: &mut Vec<Row>,
    ) {
        let more = !rules.is_empty() || !level.ended.is_empty();
        self.steps(level.active, rail, above, more, lines);
        if !rules.is_empty() {
            self.rules_line(rules, rail, level.ended.is_empty(), above, lines);
        }
        if !level.ended.is_empty() {
            self.closed_line(level.ended, rail, above, lines);
        }
    }

    /// Adds the steps `notes` under the line `above` after `rail`, the last of their level unless `more`,
    /// each followed by its own one level down, each with its title wrapped and its details under it; an
    /// open one with the lines `--full` prints under it, a closed done one with no time and its steps
    /// folded.
    pub(in crate::command::tree) fn steps(
        &self,
        notes: Vec<Step>,
        rail: &Styled,
        above: &str,
        more: bool,
        lines: &mut Vec<Row>,
    ) {
        let all = notes.len();
        let colour = self.lane(above);
        for (n, note) in notes.into_iter().enumerate() {
            let step = note.record();
            let state = note.state();
            if state.active() {
                self.until(note.trail().newest() + LIFE);
            }
            let open = self.open(note.id());
            let last = n + 1 == all && !more;
            let size = note.tree().len() - 1;
            let level = Level::of(note.children());
            let folded = state == State::Done && size > 0 && !open;
            let shown = open || (size > 0 && !folded);
            let toggle = (self.opens(note) || size > 0).then_some(shown);
            let (lead, rail) = self.branch(rail, last, toggle, colour);
            let below = match note.children().is_empty() || folded {
                true => after(&rail, Span::raw("  ")),
                false => after(&rail, paint(lane(state), "│ ")),
            };
            let first = lines.len();
            lines.push(Row::Lined {
                lead,
                marks: self.marked(note),
                title: step.title().to_string(),
                tone: self.shone(note, self.tone(look(state).title, note.newest())),
                meta: self.meta(note),
                whole: open.then(|| below.clone()),
            });
            let item = Item {
                id: note.id().to_string(),
                open,
                parent: Some(above.to_string()),
                under: level.ids(note.id()),
            };
            self.pick(item, lines, first);
            if open {
                self.closing(note, &below, true, lines);
                self.followed(note, &below, lines);
            }
            if !folded {
                self.level(level, vec![], &rail, note.id(), lines);
            }
        }
    }

    /// The lead of a line of a level after `rail`: `├─`, or `└─` when it is the `last`, in the look
    /// `colour` of the line it hangs from, then a space and `▸` or `▾` as `open` says when it opens, else
    /// `──`, and a space; and the rail of the level under it, whose corners stand under this line's
    /// marker.
    pub(super) fn branch(
        &self,
        rail: &Styled,
        last: bool,
        open: Option<bool>,
        colour: Style,
    ) -> (Styled, Styled) {
        let (corner, under) = match last {
            true => ("└─", "     "),
            false => ("├─", "│    "),
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
        match self.view.spin && note.state() == State::Working && note.children().is_empty() {
            true => {
                self.spun.set(true);
                tone.add_modifier(SHIMMER)
            }
            false => tone,
        }
    }

    /// Whether opening `note` shows something: lines under it, or steps under a closed one.
    fn opens(&self, note: Step) -> bool {
        let lines = note.record().detail().next().is_some()
            || note.done().is_some()
            || note.dropped().is_some()
            || !self.reuse.sources(note).is_empty()
            || self.reuse.followers(note) > 0;
        lines || (note.state().closed() && !note.children().is_empty())
    }

    /// Whether the step or rule `id` shows what `--full` shows: with `--full`, unless the watch closed
    /// it; else when the watch opened it.
    pub(super) fn open(&self, id: &str) -> bool {
        self.view.full != self.view.flipped.contains(id)
    }
}
