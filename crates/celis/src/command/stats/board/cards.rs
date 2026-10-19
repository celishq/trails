//! The cards of a board: every card with the picked one framed, an opened card's table, and the
//! empty card of a store with no closed step.

use super::super::sheet::{Sheet, CARDS};
use super::{head, legend, Board};
use crate::terminal::canvas::{paint, Styled, BOLD};
use crate::terminal::card::{grid, Card};
use crate::terminal::live::Page;
use crate::text::clock::weekday;
use celis_trails::records::id::short;
use celis_trails::Steps;

impl Board<'_> {
    /// The page of each card under `status`: the effect across `widths.0`, the others `widths.1` wide.
    pub(super) fn cards(
        &self,
        status: &str,
        sheets: &[Sheet; CARDS],
        (full, card): (usize, usize),
        width: usize,
    ) -> Page {
        let mut lines = head(None, status, width);
        let markdown = sheets[self.picked]
            .table
            .markdown(&sheets[self.picked].title);
        let cards: Vec<Card> = sheets
            .iter()
            .enumerate()
            .map(|(i, s)| Card {
                title: s.title.clone(),
                rows: s.face.clone(),
                picked: i == self.picked,
            })
            .collect();
        let mut line = None;
        let first = &cards[0];
        if self.picked == 0 {
            line = Some(lines.len());
        }
        lines.extend(first.lines(full, first.rows.len() + 2));
        for (r, row) in cards[1..].chunks(self.per_row).enumerate() {
            if self.picked > 0 && (self.picked - 1) / self.per_row == r {
                line = Some(lines.len());
            }
            lines.extend(grid(row, self.per_row, card));
        }
        Page {
            lines,
            picked: line,
            keys: "←→↑↓ move · enter open · m copy markdown · q quit".into(),
            markdown: Some(markdown),
            ..Page::default()
        }
    }

    /// The page of the opened card `sheet`: its table, a row a line, with its head in view when the
    /// first row is selected.
    pub(super) fn opened(&mut self, sheet: &Sheet, width: usize) -> Page {
        let ids: Vec<Option<String>> = sheet.table.rows.iter().map(|r| r.id.clone()).collect();
        self.row = self.row.min(ids.len().saturating_sub(1));
        let mut lines = head(Some(&sheet.title), "", width);
        let (table, first) = sheet.table.lines(width, Some(self.row));
        let picked = match self.row {
            0 => Some(0),
            row => first.get(row).map(|f| lines.len() + f),
        };
        lines.extend(table);
        let copy = ids.get(self.row).cloned().flatten();
        self.rows = ids;
        Page {
            lines,
            picked,
            keys: legend("↑↓ select · ← back", copy.as_deref()),
            copy: copy.map(|id| short(&id)),
            markdown: Some(sheet.table.markdown(&sheet.title)),
            ..Page::default()
        }
    }
}

/// The page of a store with no closed step, `right` in its header: a card that says how stats fill.
pub(super) fn empty(right: &str, width: usize) -> Page {
    let card = Card {
        title: "NO STATS YET".into(),
        rows: vec![
            "Stats show when your agent finishes its first step.".into(),
            Styled::from(vec![
                "Run ".into(),
                paint(BOLD, "celis stats --demo"),
                " to see sample data.".into(),
            ]),
        ],
        picked: false,
    };
    let mut lines = head(None, right, width);
    lines.extend(card.lines(width, card.rows.len() + 2));
    Page {
        lines,
        keys: "q quit".into(),
        ..Page::default()
    }
}

/// `● <n> active · <weekday>` of `steps`.
pub(super) fn status(steps: &Steps) -> String {
    let active = steps
        .roots()
        .into_iter()
        .filter(|s| s.state().active())
        .count();
    format!("● {active} active · {}", weekday(steps.now()))
}
