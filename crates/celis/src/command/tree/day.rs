//! The day groups of a tree: a line with the day and its counts, then its trails by project, kept
//! in the layout while nothing they show changes.

use super::layout::{self, Group, Key};
use super::row::Row;
use super::{Frame, Tree, PROJECT};
use crate::failure::Failure;
use crate::terminal::canvas::{clip, paint, visible, Styled, BOLD, DIM, FRAME};
use crate::text::clock::weekday;
use celis_trails::Step;
use ratatui::text::Span;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

impl Tree<'_> {
    /// What it printed as `text`.
    pub(super) fn frame(self, text: Vec<Styled>) -> Frame {
        Frame {
            text,
            remark: None,
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
    /// its active trails and the rest by project; in a watch, in a box under a rule, each project under
    /// its 📁 heading.
    pub(super) fn day(
        &self,
        at: u64,
        trails: Vec<Step>,
        out: &mut Vec<Styled>,
    ) -> Result<(), Failure> {
        let active = trails.iter().filter(|s| s.state().active()).count();
        let counts = format!(" · {active} active · {} done", trails.len() - active);
        let boxed = self.view.watch;
        let mut head = vec![paint(BOLD, &weekday(at)), paint(DIM, &counts)];
        if boxed {
            head.insert(0, Span::raw("📅 "));
        }
        let mut lines = vec![Row::Mark(false, Box::new(Row::Text(Styled::from(head))))];
        if boxed {
            lines.push(Row::Mark(false, Box::new(Row::Text(Styled::from(RULED)))));
        }
        if boxed {
            self.projects(trails, &mut lines);
        } else {
            self.active_first(trails, &mut lines);
        }
        if !boxed {
            return self.show(lines, out);
        }
        self.put(out, edge("╭", "╮", self.width))?;
        let mut body = vec![];
        self.show(lines, &mut body)?;
        out.extend(body.into_iter().map(|line| boxed_line(line, self.width)));
        self.put(out, edge("╰", "╯", self.width))
    }

    /// Adds `trails` as `day` does off a box: its active trails, then the rest by project.
    fn active_first(&self, trails: Vec<Step>, lines: &mut Vec<Row>) {
        let (active, rest): (Vec<Step>, Vec<Step>) =
            trails.into_iter().partition(|s| s.state().active());
        let mut last = None;
        if !active.is_empty() {
            lines.push(self.gap());
        }
        for note in active {
            let block = self.view.full || !self.view.all;
            self.trail(note, block, &mut last, lines);
        }
        for (name, trails) in by_project(rest) {
            lines.push(self.gap());
            lines.push(Row::Mark(
                false,
                Box::new(Row::Text(Styled::from(paint(PROJECT, &name)))),
            ));
            let mut last = None;
            for note in trails {
                let block = self.view.full || (!self.view.all && !self.view.watch);
                self.trail(note, block, &mut last, lines);
            }
        }
    }

    /// Adds `trails` in a day's box by project, a project with an active trail first: each after a
    /// blank line, `📁` and its name, and a blank line, its active trails as blocks first.
    fn projects(&self, trails: Vec<Step>, lines: &mut Vec<Row>) {
        let mut projects = by_project(trails);
        projects.sort_by_key(|(_, trails)| !trails.iter().any(|s| s.state().active()));
        for (name, trails) in projects {
            let (active, rest): (Vec<Step>, Vec<Step>) =
                trails.into_iter().partition(|s| s.state().active());
            lines.push(self.gap());
            let title = vec![Span::raw("📁 "), paint(PROJECT, &name)];
            lines.push(Row::Mark(false, Box::new(Row::Text(Styled::from(title)))));
            lines.push(self.gap());
            let mut last = None;
            for note in active {
                self.trail(note, self.view.full || !self.view.all, &mut last, lines);
            }
            for note in rest {
                self.trail(note, self.view.full, &mut last, lines);
            }
        }
    }

    /// A blank line, which no element holds.
    pub(super) fn gap(&self) -> Row {
        Row::Mark(false, Box::new(Row::Text(Styled::default())))
    }
}

/// What stands for the rule under a day's line until its box draws it.
const RULED: &str = "\u{E010}";

/// A day box's top or bottom border, from `left` to `right`, round a line `width` wide.
fn edge(left: &str, right: &str, width: Option<usize>) -> Styled {
    let across = "─".repeat(width.unwrap_or(0) + 2);
    Styled::from(vec![
        Span::raw("  "),
        paint(FRAME, &format!("{left}{across}{right}")),
    ])
}

/// `line` in a day's box `width` wide: its cursor column, then `│ `, its text cut and padded to the width, and
/// ` │`; the rule under the day's line as `├─…─┤`.
fn boxed_line(line: Styled, width: Option<usize>) -> Styled {
    let width = width.unwrap_or(0);
    let mut spans = line.spans.into_iter();
    let cursor = spans.next().unwrap_or_else(|| Span::raw("  "));
    let text: Vec<Span<'static>> = spans.collect();
    if text.iter().any(|s| s.content == RULED) {
        let rule = vec![
            cursor,
            paint(FRAME, &format!("├{}┤", "─".repeat(width + 2))),
        ];
        return Styled::from(rule);
    }
    let text = clip(&Styled::from(text), width).spans;
    let used: usize = text.iter().map(|s| visible(&s.content)).sum();
    let mut row = vec![cursor, paint(FRAME, "│ ")];
    row.extend(text);
    row.push(Span::raw(" ".repeat(width.saturating_sub(used))));
    row.push(paint(FRAME, " │"));
    Styled::from(row)
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
