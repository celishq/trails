//! Where the cards of a trail's graph stand: chained left to right, wrapped into rows.

use super::card::{Card, WIDTH};
use super::moment::Moment;
use celis_trails::Step;
use std::collections::HashSet;

/// The gap between two cards of a row, where the edge into the second runs.
pub(super) const GAP: u16 = 3;

/// The column the first card of a row starts at.
pub(super) const LEFT: u16 = 2;

/// The cards of a trail's graph: each card's place, and the rows they take.
pub(super) struct Layout<'a> {
    pub places: Vec<Place<'a>>,
    pub height: u16,
}

/// Where one card stands: its step, its column and row, its height, the height of its row, and the
/// columns of cards it spans.
#[derive(Clone, Copy)]
pub(super) struct Place<'a> {
    pub step: Step<'a>,
    pub x: u16,
    pub y: u16,
    pub height: u16,
    pub row: u16,
    pub span: u16,
}

impl<'a> Layout<'a> {
    /// The places of the cards of `tops`, `columns` to a row, each row as tall as its tallest card and
    /// a blank row after it; a card the person `opened` takes a whole row.
    pub fn of(tops: &[Step<'a>], columns: u16, opened: &HashSet<String>, m: &Moment) -> Layout<'a> {
        let columns = columns.max(1);
        let mut rows: Vec<Vec<(Step<'a>, u16, u16)>> = vec![];
        let mut used = columns;
        for step in tops {
            let span = match opened.contains(step.id()) {
                true => columns,
                false => 1,
            };
            if used + span > columns {
                rows.push(vec![]);
                used = 0;
            }
            if let Some(row) = rows.last_mut() {
                row.push((*step, used, span));
            }
            used += span;
        }
        let mut places = vec![];
        let mut y = 0;
        for row in rows {
            let heights: Vec<u16> = row
                .iter()
                .map(|(s, _, span)| Card::height(*s, opened.contains(s.id()), breadth(*span), m))
                .collect();
            let tall = heights.iter().copied().max().unwrap_or(0);
            for ((step, column, span), height) in row.into_iter().zip(heights) {
                places.push(Place {
                    step,
                    x: LEFT + column * (WIDTH + GAP),
                    y,
                    height,
                    row: tall,
                    span,
                });
            }
            y += tall + 1;
        }
        Layout { places, height: y }
    }
}

/// The width of a card that spans `span` columns.
pub(super) fn breadth(span: u16) -> u16 {
    span * WIDTH + span.saturating_sub(1) * GAP
}

/// The columns of cards that fit `width` cells.
pub(super) fn columns(width: u16) -> u16 {
    ((width.saturating_sub(LEFT) + GAP) / (WIDTH + GAP)).max(1)
}
