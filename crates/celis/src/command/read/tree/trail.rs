//! A trail in the tree: a block with its steps, its patterns line and its closed line, or one line.

use super::level::Level;
use super::row::Row;
use super::{Item, Summary, Tree, PATTERNS};
use crate::progress::Phase;
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::terminal::look::look;
use celis_nostr::Note;
use celis_trails::{Step, Trail};
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds the trail `note` as a block when `block`, else as one line, the other way when the view
    /// flipped it, in the agent view off a watch; in a watch, a blank line comes before it `after` a
    /// trail.
    pub(super) fn trail(&self, note: Step, block: bool, after: &mut bool, lines: &mut Vec<Row>) {
        let block = block != self.view.flipped.contains(note.id());
        if self.view.watch && *after {
            lines.push(self.gap());
        }
        match (self.view.watch, block) {
            (true, true) => self.block(note, lines),
            (true, false) => self.line(note, lines),
            (false, true) => self.agent_block(note, lines),
            (false, false) => self.agent_line(note, lines),
        }
        *after = true;
    }

    /// Adds `item` to the order of the lines a watch selects, and starts at `first` of `lines` the lines
    /// of its element, selected when the view selected it.
    pub(super) fn pick(&self, item: Item, lines: &mut [Row], first: usize) {
        let selected = self.view.selected.as_ref() == Some(&item.id);
        let line = std::mem::replace(&mut lines[first], Row::Text(Styled::default()));
        lines[first] = Row::Mark(selected, Box::new(line));
        self.order.borrow_mut().push(item);
    }

    /// The trail `note` as a line a watch selects, `open` when it prints as a block: with every pattern of
    /// `patterns` and every step it prints under it.
    pub(super) fn item(&self, note: Step, open: bool, patterns: &[&Note]) -> Item {
        let folded = (!patterns.is_empty()).then(|| format!("{PATTERNS}{}", note.id()));
        let patterns = patterns.iter().map(|pattern| pattern.id.clone());
        let steps = Level::of(Trail::new(note).parts()).ids();
        Item {
            id: note.id().to_string(),
            open,
            parent: None,
            under: folded.into_iter().chain(patterns).chain(steps).collect(),
        }
    }

    /// Adds `note` as a block: `▾` when something opens under it, its mark and title, wrapped with the tree
    /// line down from its mark when it has, its project with no grouping or while active off a watch, and
    /// done/all or its counts at the right; with full, the lines
    /// `--full` prints but its grade; then its tree: its steps, its patterns, and its closed steps folded when
    /// many.
    pub(super) fn block(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let state = Phase::of(&note);
        let work = self.work(note);
        let summary = self.summary(note, work);
        let level = Level::of(Trail::new(note).parts());
        let under =
            !(level.active.is_empty() && level.ended.is_empty() && summary.records().is_empty());
        let first = lines.len();
        let mut meta = vec![];
        if !self.grouped || (state.active() && !self.view.watch) {
            meta.push(vec![paint(DIM, &step.anchor.name)]);
        }
        meta.extend(came(&summary));
        lines.push(Row::Lined {
            lead: Styled::from(vec![
                paint(DIM, if under { "▾" } else { " " }),
                Span::raw(" "),
            ]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(trail_tone(state), note.newest()),
            meta: joined(meta),
            whole: Some(match under {
                true => Styled::from(vec![Span::raw("  "), paint(self.lane(), "│")]),
                false => Styled::from("  "),
            }),
        });
        self.pick(self.item(note, true, &summary.records()), lines, first);
        let rail = Styled::from("  ");
        if self.view.full {
            let hair = Styled::from(vec![Span::raw("  "), paint(self.lane(), "│ ")]);
            self.closing(work, &hair, false, lines);
        }
        self.level(level, summary.records(), &rail, note.id(), lines);
    }

    /// Adds `note` as one row: `▸` when something opens under it, its mark and title wrapped, and at the
    /// right done/all while active, else its steps counted by state.
    pub(super) fn line(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let state = Phase::of(&note);
        let work = self.work(note);
        let summary = self.summary(note, work);
        let under = !(Trail::new(note).parts().is_empty() && summary.records().is_empty());
        let meta: Vec<_> = came(&summary).into_iter().collect();
        let first = lines.len();
        lines.push(Row::Lined {
            lead: Styled::from(vec![
                paint(DIM, if under { "▸" } else { " " }),
                Span::raw(" "),
            ]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(trail_tone(state), note.newest()),
            meta: joined(meta),
            whole: Some(Styled::from("  ")),
        });
        self.pick(self.item(note, false, &summary.records()), lines, first);
    }
}

/// The look of a trail's title in `state`: bold while active, else as its state's titles look.
fn trail_tone(state: Phase) -> ratatui::style::Style {
    match state.active() {
        true => BOLD,
        false => look(state).title,
    }
}

/// What a trail's steps came to, dim; none with no steps and no time.
fn came(summary: &Summary) -> Option<Vec<Span<'static>>> {
    let text = summary.meta.to_string();
    (!text.is_empty()).then(|| vec![paint(DIM, &text)])
}

/// The `parts` of a row's meta, a dim ` · ` between each two.
fn joined(parts: Vec<Vec<Span<'static>>>) -> Styled {
    let mut spans = vec![];
    for (n, part) in parts.into_iter().enumerate() {
        if n > 0 {
            spans.push(paint(DIM, " · "));
        }
        spans.extend(part);
    }
    Styled::from(spans)
}

mod fold;
pub(super) use fold::counts;
mod note;
mod steps;
