//! The day groups of a tree: a line with the day and its counts, then its trails by project, kept
//! in the layout while nothing they show changes.

use super::layout::{self, Group, Key};
use super::row::Row;
use super::{Frame, Tree, PROJECT};
use crate::failure::Failure;
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::text::clock::weekday;
use celis_trails::Step;
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
    /// its active trails, then the rest by project.
    pub(super) fn day(
        &self,
        at: u64,
        trails: Vec<Step>,
        out: &mut Vec<Styled>,
    ) -> Result<(), Failure> {
        let active = trails.iter().filter(|s| s.active()).count();
        let counts = format!(" · {active} active · {} done", trails.len() - active);
        let head = Styled::from(vec![paint(BOLD, &weekday(at)), paint(DIM, &counts)]);
        let mut lines = vec![Row::Mark(false, Box::new(Row::Text(head)))];
        let (active, rest): (Vec<Step>, Vec<Step>) = trails.into_iter().partition(|s| s.active());
        let mut last = None;
        if !active.is_empty() {
            lines.push(self.gap());
        }
        for note in active {
            let block = self.view.full || !self.view.all;
            self.trail(note, block, &mut last, &mut lines);
        }
        let mut projects: Vec<(&str, Vec<Step>)> = vec![];
        for note in rest {
            let project = note.record().anchor.project.as_str();
            match projects.iter_mut().find(|(p, _)| *p == project) {
                Some((_, trails)) => trails.push(note),
                None => projects.push((project, vec![note])),
            }
        }
        for (_, trails) in projects {
            let name = &trails[0].record().anchor.name;
            lines.push(self.gap());
            lines.push(Row::Mark(
                false,
                Box::new(Row::Text(Styled::from(paint(PROJECT, name)))),
            ));
            let mut last = None;
            for note in trails {
                let block = self.view.full || (!self.view.all && !self.view.watch);
                self.trail(note, block, &mut last, &mut lines);
            }
        }
        self.show(lines, out)
    }

    /// A blank line, which no element holds.
    pub(super) fn gap(&self) -> Row {
        Row::Mark(false, Box::new(Row::Text(Styled::default())))
    }
}
