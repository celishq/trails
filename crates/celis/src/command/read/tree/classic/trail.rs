//! A trail in the tree: a block with its steps, its patterns line and its closed line, or one line.

use super::level::Level;
use super::row::Row;
use super::{Item, Tree, PATTERNS, PROJECT};
use crate::progress::Phase;
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::terminal::classic::look;
use crate::text::clock::{stride, took};
use celis_nostr::Note;
use celis_trails::{Step, Trail};
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

    /// The trail `note` as a line a watch selects, `open` when it prints as a block: with every pattern of
    /// `patterns` and every step it prints under it.
    pub(super) fn item(&self, note: Step, open: bool, patterns: &[&Note]) -> Item {
        let folded = (!patterns.is_empty()).then(|| format!("{PATTERNS}{}", note.id()));
        let patterns = patterns.iter().map(|pattern| pattern.id.clone());
        let steps = Level::of(Trail::new(note).parts()).ids(note.id());
        Item {
            id: note.id().to_string(),
            open,
            parent: None,
            under: folded.into_iter().chain(patterns).chain(steps).collect(),
        }
    }

    /// Adds `note` as a block: `▾`, its mark and title, then its lines: its worked time, its last change
    /// and its project while active, its project with no grouping, and the lines `--full` prints but its
    /// grade; then its tree: its active steps, its patterns, then its closed steps.
    pub(super) fn block(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let work = self.work(note);
        let summary = self.summary(note, work);
        let hair = Styled::from(vec![Span::raw("  "), paint(DIM, "│ ")]);
        let first = lines.len();
        lines.push(Row::Titled {
            lead: Styled::from(vec![paint(DIM, "▾"), Span::raw(" ")]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(BOLD, note.newest()),
            rest: hair.clone(),
        });
        self.pick(self.item(note, true, &summary.notes), lines, first);
        if let Some(worked) = summary.meta.worked {
            self.labelled(lines, &hair, "took", &took(worked));
        }
        if self.view.spin && note.active() {
            let seconds = self.steps.now().saturating_sub(note.newest());
            self.until(self.steps.now() + stride(seconds));
            self.labelled(lines, &hair, "active", &format!("{} ago", took(seconds)));
        }
        if !self.grouped || note.active() {
            lines.push(Row::Note {
                rail: hair.clone(),
                label: "project",
                text: step.anchor.name.clone(),
                tone: PROJECT,
            });
        }
        self.closing(work, &hair, false, lines);
        self.used(work, &hair, lines);
        let level = Level::of(Trail::new(note).parts());
        self.level(level, summary.notes, &Styled::from("  "), note.id(), lines);
    }

    /// Adds `note` as one row: `▸`, its mark and wrapped title, then its details: its steps and worked
    /// time.
    pub(super) fn line(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let state = Phase::of(&note);
        let work = self.work(note);
        let summary = self.summary(note, work);
        let first = lines.len();
        let rest = Styled::from("    ");
        lines.push(Row::Titled {
            lead: Styled::from(vec![paint(DIM, "▸"), Span::raw(" ")]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(look(state).title, note.newest()),
            rest: rest.clone(),
        });
        self.details(&rest, summary.meta, lines);
        self.pick(self.item(note, false, &summary.notes), lines, first);
    }
}

mod fold;
mod note;
mod steps;
