//! The route of a step on a board: the steps it follows, the step, and the steps that follow it.

use super::super::table::Table;
use super::{head, legend, Board, NARROW};
use crate::progress::Model;
use crate::terminal::canvas::{paint, Styled, DIM};
use crate::terminal::live::Page;
use celis_store::records::id::short;
use celis_trails::Step;
use face::face;
use link::Links;

impl Board<'_> {
    /// The page of the route of `step`: the steps it follows, the step, and the steps that follow it.
    pub(super) fn route(&mut self, model: &Model, step: Step, width: usize) -> Page {
        let reuse = model.reuse;
        let up = reuse.up(step);
        let down = reuse.down(step);
        let count = up.len() + down.len();
        self.row = self.row.min(count.saturating_sub(1));
        let name = &step.record().anchor.name;
        let mut links = Links::new(head(Some(&short(step.id())), name, width), self.row, width);
        if !up.is_empty() {
            links.lines.push(paint(DIM, "  FOLLOWS").into());
            for link in &up {
                links.add("follows", link.source, link.depth);
            }
            links.lines.push(Styled::default());
        }
        links.lines.extend(face(step, width, width < NARROW));
        if !down.is_empty() {
            let followed = paint(DIM, "  FOLLOWED BY").into();
            links.lines.extend([Styled::default(), followed]);
            for link in &down {
                links.add("followed by", link.step, link.depth);
            }
        }
        if count == 0 {
            links.lines.extend([
                Styled::default(),
                paint(DIM, "  it follows no step, and no step follows it").into(),
            ]);
        }
        let copy = links.rows.get(self.row).cloned().flatten();
        let markdown = Table {
            head: vec!["Link", "Step", "Project", "Took"],
            rows: links.table,
            wide: 1,
        }
        .markdown(&format!("{} {}", short(step.id()), step.title()));
        self.rows = links.rows;
        Page {
            lines: links.lines,
            picked: links.picked,
            keys: legend("↑↓ select · enter go to · ← back", copy.as_deref()),
            copy: copy.map(|id| short(&id)),
            markdown: Some(markdown),
            ..Page::default()
        }
    }
}

mod face;
mod link;
