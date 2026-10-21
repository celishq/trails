//! A trail in the tree: a block with its steps, its rules line and its closed line, or one line.

use super::level::Level;
use super::row::Row;
use super::{Item, RuleNote, Summary, Tree, RULES};
use crate::terminal::bar::bar;
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::terminal::look::{lane, look, SPINNER};
use crate::text::clock::{stride, took};
use crate::text::line::plural;
use celis_trails::{State, Step, Trail};
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds the trail `note` as a block when `block`, else as one line, the other way when the view
    /// flipped it, in the agent view off a watch; a blank line comes between it and a `last` trail when
    /// either is a block in a watch.
    pub(super) fn trail(
        &self,
        note: Step,
        block: bool,
        last: &mut Option<bool>,
        lines: &mut Vec<Row>,
    ) {
        let block = block != self.view.flipped.contains(note.id());
        if self.view.watch && last.is_some_and(|last| last || block) {
            lines.push(self.gap());
        }
        match (self.view.watch, block) {
            (true, true) => self.block(note, lines),
            (true, false) => self.line(note, lines),
            (false, true) => self.agent_block(note, lines),
            (false, false) => self.agent_line(note, lines),
        }
        *last = Some(block);
    }

    /// Adds `item` to the order of the lines a watch selects, and starts at `first` of `lines` the lines
    /// of its element, selected when the view selected it.
    pub(super) fn pick(&self, item: Item, lines: &mut [Row], first: usize) {
        let selected = self.view.selected.as_ref() == Some(&item.id);
        let line = std::mem::replace(&mut lines[first], Row::Text(Styled::default()));
        lines[first] = Row::Mark(selected, Box::new(line));
        self.order.borrow_mut().push(item);
    }

    /// The trail `note` as a line a watch selects, `open` when it prints as a block: with every rule of
    /// `rules` and every step it prints under it.
    pub(super) fn item(&self, note: Step, open: bool, rules: &[RuleNote]) -> Item {
        let folded = (!rules.is_empty()).then(|| format!("{RULES}{}", note.id()));
        let rules = rules.iter().map(|(rule, _)| rule.id.clone());
        let steps = Level::of(Trail::new(note).parts()).ids(note.id());
        Item {
            id: note.id().to_string(),
            open,
            parent: None,
            under: folded.into_iter().chain(rules).chain(steps).collect(),
        }
    }

    /// Adds `note` as a block: `▾`, its mark and title, its project with no grouping or while active off
    /// a watch, a bar of its steps with done/all and its worked time at the right; then the lines
    /// `--full` prints but its grade; then a rail from its mark down to its tree: its active steps, its
    /// rules, then its closed steps.
    pub(super) fn block(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let state = note.state();
        let work = self.work(note);
        let summary = self.summary(note, work);
        let colour = match self.spins(note) {
            true => SPINNER,
            false => lane(state),
        };
        let hair = Styled::from(vec![Span::raw("  "), paint(colour, "│ ")]);
        let first = lines.len();
        let mut meta = vec![];
        if !self.grouped || (state.active() && !self.view.watch) {
            meta.push(vec![paint(DIM, &step.anchor.name)]);
        }
        meta.extend(came(state, &summary));
        if let Some(worked) = summary.worked {
            meta.push(vec![paint(DIM, &took(worked))]);
        }
        lines.push(Row::Lined {
            lead: Styled::from(vec![paint(DIM, "▾"), Span::raw(" ")]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(BOLD, note.newest()),
            meta: joined(meta),
            whole: Some(hair.clone()),
        });
        self.pick(self.item(note, true, &summary.rules), lines, first);
        if self.view.spin && state.active() {
            let seconds = self.steps.now().saturating_sub(note.newest());
            self.until(self.steps.now() + stride(seconds));
        }
        self.closing(work, &hair, false, lines);
        self.followed(work, &hair, lines);
        let level = Level::of(Trail::new(note).parts());
        if !(level.active.is_empty() && level.ended.is_empty() && summary.rules.is_empty()) {
            let rail = vec![Span::raw("  "), paint(colour, "│")];
            lines.push(Row::Text(Styled::from(rail)));
        }
        self.level(level, summary.rules, &Styled::from("  "), note.id(), lines);
    }

    /// Adds `note` as one row: `▸`, its mark and title, and at the right what it came to: a bar of its
    /// steps with done/all while active, else its steps, how many were done, its state and its time.
    pub(super) fn line(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let state = note.state();
        let work = self.work(note);
        let summary = self.summary(note, work);
        let mut meta: Vec<_> = came(state, &summary).into_iter().collect();
        if let (Some(worked), false) = (summary.worked, state.active()) {
            meta.push(vec![paint(DIM, &took(worked))]);
        }
        let first = lines.len();
        lines.push(Row::Lined {
            lead: Styled::from(vec![paint(DIM, "▸"), Span::raw(" ")]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(look(state).title, note.newest()),
            meta: joined(meta),
            whole: None,
        });
        self.pick(self.item(note, false, &summary.rules), lines, first);
    }
}

/// What a trail's steps came to: a bar of them with done/all while `active`, else their count when all
/// are done, else their count by state.
fn came(state: State, summary: &Summary) -> Option<Vec<Span<'static>>> {
    let (all, done) = (summary.all, summary.done);
    let share = |n: usize| n as f64 / all as f64;
    match (state.active(), all) {
        (_, 0) => None,
        (true, _) => {
            let mut spans = bar(share(done), share(summary.working), 20).spans;
            spans.push(paint(DIM, &format!("  {done}/{all}")));
            Some(spans)
        }
        (false, _) if all == done => Some(vec![paint(DIM, &plural(all, "step"))]),
        (false, _) => Some(fold::counts(summary.tally)),
    }
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
mod note;
mod steps;
