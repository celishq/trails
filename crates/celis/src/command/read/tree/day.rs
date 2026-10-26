//! The day groups of a tree: a line with the day and its counts, then its trails by project, kept
//! in the layout while nothing they show changes.

use super::layout::{self, Group, Key};
use super::row::Row;
use super::trail::counts;
use super::{live, Frame, Item, Tree, DAY, PROJECT, TRAILS};
use crate::failure::Failure;
use crate::progress::{Phase, Tally};
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::terminal::look::look;
use crate::text::clock::{day, weekday};
use celis_trails::step::steps::LIFE;
use celis_trails::Step;
use ratatui::style::Style;
use ratatui::text::Span;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

impl Tree<'_> {
    /// What it printed as `text`.
    pub(super) fn frame(self, text: Vec<Styled>) -> Frame {
        Frame {
            text,
            next: self.next.get(),
            spun: self.spun.get(),
            order: self.order.take(),
            picked: self.picked.get(),
        }
    }

    /// Prints the day group of `trails` from the day `name` of `at` as `day` does, or as it last did
    /// when its layout kept it and nothing it shows changed.
    pub(super) fn group(
        &self,
        (at, name): (u64, String),
        trails: Vec<Step>,
        out: &mut Vec<Styled>,
    ) -> Result<(), Failure> {
        let Some(layout) = self.layout else {
            return self.day(at, trails, out);
        };
        let key = Key {
            trails: trails.iter().map(|s| s.id().to_string()).collect(),
            width: self.width,
            full: self.view.full,
            all: self.view.all,
        };
        if let Some(group) = layout.get(&name, &key, self.view, self.steps.now()) {
            if cfg!(debug_assertions) {
                let blank = self.blank();
                let mut text = Vec::new();
                blank.day(at, trails, &mut text)?;
                assert_eq!(
                    blank.frame(text).text,
                    group.part.text,
                    "a kept group is old"
                );
            }
            return self.add(&group.part, out);
        }
        let blank = self.blank();
        let mut text = Vec::new();
        blank.day(at, trails, &mut text)?;
        let part = blank.frame(text);
        self.add(&part, out)?;
        let ids: HashSet<String> = part
            .order
            .iter()
            .flat_map(|i| std::iter::once(&i.id).chain(&i.under))
            .cloned()
            .collect();
        let group = Group {
            key,
            flipped: layout::flipped(&ids, self.view),
            selected: layout::selected(&ids, self.view),
            ids,
            part,
        };
        layout.keep(name, group);
        Ok(())
    }

    /// A tree like it that has printed nothing.
    pub(super) fn blank(&self) -> Tree<'_> {
        Tree {
            spun: Cell::default(),
            order: RefCell::default(),
            printed: Cell::default(),
            picked: Cell::default(),
            next: Cell::default(),
            ..*self
        }
    }

    /// Prints `part` after what it printed.
    pub(super) fn add(&self, part: &Frame, out: &mut Vec<Styled>) -> Result<(), Failure> {
        out.extend(part.text.iter().cloned());
        self.order.borrow_mut().extend(part.order.iter().cloned());
        if let (None, Some(picked)) = (self.picked.get(), part.picked) {
            self.picked.set(Some(self.printed.get() + picked));
        }
        self.printed.set(self.printed.get() + part.text.len());
        if let Some(next) = part.next {
            self.until(next);
        }
        self.spun.set(self.spun.get() || part.spun);
        Ok(())
    }

    /// Prints the day group of `trails` from the day of `at`: a line with the day and its counts, then
    /// its active trails and the rest by project; in a watch, after `🗓️`, each project under its 📁
    /// heading under the date.
    pub(super) fn day(
        &self,
        at: u64,
        trails: Vec<Step>,
        out: &mut Vec<Styled>,
    ) -> Result<(), Failure> {
        let now = self.steps.now();
        for note in trails.iter().filter(|s| self.view.watch && live(**s, now)) {
            self.until(note.newest() + LIFE);
        }
        let active = trails.iter().filter(|s| self.lively(**s)).count();
        let counts = format!(" · {active} active · {} done", trails.len() - active);
        let mut head = vec![paint(BOLD, &weekday(at)), paint(DIM, &counts)];
        if self.view.watch {
            head.insert(0, Span::raw(CALENDAR));
        }
        let mut lines = vec![Row::Mark(false, Box::new(Row::Text(Styled::from(head))))];
        match self.view.watch {
            true => {
                self.projects(trails, &mut lines);
                nest(&mut lines[1..], DATED);
            }
            false => self.active_first(trails, &mut lines),
        }
        self.show(lines, out)
    }

    /// Adds `trails` as `day` does off a box: its active trails, then the rest by project.
    fn active_first(&self, trails: Vec<Step>, lines: &mut Vec<Row>) {
        let (active, rest): (Vec<Step>, Vec<Step>) = trails.into_iter().partition(|s| s.active());
        let mut after = false;
        if !active.is_empty() {
            lines.push(self.gap());
        }
        for note in active {
            let block = self.view.full || !self.view.all;
            self.trail(note, block, &mut after, lines);
        }
        for (name, trails) in by_project(rest) {
            lines.push(self.gap());
            lines.push(Row::Mark(
                false,
                Box::new(Row::Text(Styled::from(paint(PROJECT, &name)))),
            ));
            let mut after = false;
            for note in trails {
                let block = self.view.full || (!self.view.all && !self.view.watch);
                self.trail(note, block, &mut after, lines);
            }
        }
    }

    /// Adds `trails` by project, a project with a live trail first: each after a blank line, `📁` and its
    /// name, and a blank line, then one level in its live trails as blocks and the rest folded into one line.
    fn projects(&self, trails: Vec<Step>, lines: &mut Vec<Row>) {
        let mut projects = by_project(trails);
        projects.sort_by_key(|(_, trails)| !trails.iter().any(|s| self.lively(*s)));
        for (name, trails) in projects {
            let (active, rest): (Vec<Step>, Vec<Step>) =
                trails.into_iter().partition(|s| self.lively(*s));
            lines.push(self.gap());
            lines.push(project(&name, PROJECT));
            lines.push(self.gap());
            let (start, mut after) = (lines.len(), false);
            for note in active {
                self.trail(note, true, &mut after, lines);
            }
            if !rest.is_empty() {
                if after {
                    lines.push(self.gap());
                }
                self.closed_trails(rest, lines);
            }
            nest(&mut lines[start..], FILED);
        }
    }

    /// Adds the line `▸ ✓ <n> closed` of the closed `trails` of a project, their count by state at the
    /// right; open, each of them as a line one level in that opens as a block.
    fn closed_trails(&self, trails: Vec<Step>, lines: &mut Vec<Row>) {
        let id = format!("{TRAILS}{}", trails[0].id());
        let open = self.view.flipped.contains(&id);
        let first = lines.len();
        let done = look(Phase::Done);
        lines.push(Row::Lined {
            lead: arrow(open),
            marks: Styled::from(paint(done.text, done.mark)),
            title: format!("{} closed", trails.len()),
            tone: Style::new(),
            meta: Styled::from(counts(Tally::of(&trails))),
            whole: None,
        });
        let item = Item {
            id: id.clone(),
            open,
            parent: None,
            under: trails.iter().map(|s| s.id().to_string()).collect(),
        };
        self.pick(item, lines, first);
        if !open {
            return;
        }
        let mut after = false;
        for note in trails {
            let start = lines.len();
            self.trail(note, self.view.full, &mut after, lines);
            nest(&mut lines[start..], FOLDED);
            after = false;
        }
    }

    /// Prints the days before today with trails not among `shown`, the `EARLIER` newest of them or with
    /// all every one, packed: each one line `▸ <day>` with its trails counted by state; open, after a
    /// blank line, its trails by project, each project under its date and its trails under its name, each
    /// a line that opens as a block. Whether more days lie past those it printed.
    pub(super) fn earlier(&self, shown: &[Step], out: &mut Vec<Styled>) -> Result<bool, Failure> {
        let today = day(self.steps.now());
        let most = match self.view.all {
            true => usize::MAX,
            false => EARLIER,
        };
        let mut days: Vec<(u64, String, Vec<Step>)> = vec![];
        let mut more = false;
        for note in self.steps.roots().into_iter().rev() {
            let at = note.note().created_at;
            let name = day(at);
            if name == today || shown.iter().any(|s| s.id() == note.id()) {
                continue;
            }
            match days.iter().position(|(_, n, _)| *n == name) {
                Some(found) => days[found].2.push(note),
                None if days.len() < most => days.push((at, name, vec![note])),
                None => more = true,
            }
        }
        let mut lines = vec![];
        let mut before = None;
        for (at, name, trails) in days {
            let id = format!("{DAY}{name}");
            let open = self.view.flipped.contains(&id);
            if before.is_none_or(|b| b) || open {
                lines.push(self.gap());
            }
            before = Some(open);
            let first = lines.len();
            lines.push(Row::Lined {
                lead: dated(open),
                marks: Styled::default(),
                title: weekday(at),
                tone: DIM.patch(BOLD),
                meta: Styled::from(counts(Tally::of(&trails))),
                whole: None,
            });
            let item = Item {
                id,
                open,
                parent: None,
                under: trails.iter().map(|s| s.id().to_string()).collect(),
            };
            self.pick(item, &mut lines, first);
            if !open {
                continue;
            }
            for (name, trails) in by_project(trails) {
                lines.push(self.gap());
                let start = lines.len();
                lines.push(project(&name, DIM.patch(BOLD)));
                let inner = lines.len();
                for note in trails {
                    let mut after = false;
                    self.trail(note, self.view.full, &mut after, &mut lines);
                }
                nest(&mut lines[inner..], FILED);
                nest(&mut lines[start..], DATED);
            }
        }
        self.show(lines, out)?;
        Ok(more)
    }

    /// Whether `note` heads its day: in a watch while it is a live session, else while it is active.
    fn lively(&self, note: Step) -> bool {
        match self.view.watch {
            true => live(note, self.steps.now()),
            false => note.active(),
        }
    }

    /// A blank line, which no element holds.
    pub(super) fn gap(&self) -> Row {
        Row::Mark(false, Box::new(Row::Text(Styled::default())))
    }
}

/// The calendar before today in a watch, where an earlier day's arrow stands: one with no date on it,
/// two spaces after it.
const CALENDAR: &str = "🗓\u{fe0f}  ";

/// What a row under a day starts with in a watch, so its icon stands under the date.
const DATED: &str = "    ";

/// What a trail under a project's heading starts with, so its mark stands under the project's name.
const FILED: &str = "  ";

/// What a trail under a fold of closed trails starts with, so its mark stands a level, four cells, right
/// of the fold's.
const FOLDED: &str = "    ";

/// The most days before today a watch lists, each folded, until Show more lists them all.
const EARLIER: usize = 5;

/// `rows` moved right by `inset`.
fn nest(rows: &mut [Row], inset: &str) {
    for row in rows {
        row.indent(inset);
    }
}

/// The arrow before a line that opens, `▾` when `open`, and a space.
fn arrow(open: bool) -> Styled {
    Styled::from(vec![
        paint(DIM, if open { "▾" } else { "▸" }),
        Span::raw(" "),
    ])
}

/// The arrow before a day or Show more, `▾` when `open`, and as many spaces as the calendar before
/// today leaves, so its text starts where today's date does.
pub(super) fn dated(open: bool) -> Styled {
    let mut lead = arrow(open);
    lead.push_span("  ");
    lead
}

/// The heading of the project `name` in the look `tone`: `📁` and two spaces.
fn project(name: &str, tone: Style) -> Row {
    let title = vec![Span::raw("📁  "), paint(tone, name)];
    Row::Mark(false, Box::new(Row::Text(Styled::from(title))))
}

/// `trails` by project, each project's name with its trails, in the order they first come.
fn by_project(trails: Vec<Step>) -> Vec<(String, Vec<Step>)> {
    let mut projects: Vec<(String, String, Vec<Step>)> = vec![];
    for note in trails {
        let anchor = &note.record().anchor;
        match projects.iter_mut().find(|(p, ..)| *p == anchor.project) {
            Some((.., trails)) => trails.push(note),
            None => projects.push((anchor.project.clone(), anchor.name.clone(), vec![note])),
        }
    }
    projects
        .into_iter()
        .map(|(_, name, trails)| (name, trails))
        .collect()
}
