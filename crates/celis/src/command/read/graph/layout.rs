//! Where the cards of a trail's graph stand: chained left to right on the grid's tracks, wrapped into
//! rows.

use super::card::{Card, WIDTH};
use super::moment::Moment;
use crate::terminal::grid::{Grid, GUTTER, TRACKS};
use celis_trails::Step;
use std::collections::HashSet;

/// The gap between two cards of a row, where the edge into the second runs: the grid's gutter.
pub(super) const GAP: u16 = GUTTER;

/// The cards of a trail's graph: each card's place, and the rows they take.
pub(super) struct Layout<'a> {
    pub places: Vec<Place<'a>>,
    pub height: u16,
}

/// Where one card stands: its step, its column and row, its width, its height, and the height of its
/// row.
#[derive(Clone, Copy)]
pub(super) struct Place<'a> {
    pub step: Step<'a>,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub height: u16,
    pub row: u16,
}

impl<'a> Layout<'a> {
    /// The places of the cards of `tops` on `grid`, each on the tracks `tracks` gives, as many to a row
    /// as fit, each row as tall as its tallest card and a blank row after it; a card the person
    /// `opened` takes a whole row.
    pub fn of(tops: &[Step<'a>], grid: Grid, opened: &HashSet<String>, m: &Moment) -> Layout<'a> {
        let per = tracks(grid);
        let columns = TRACKS / per;
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
            let spans: Vec<(u16, u16)> = row
                .iter()
                .map(|(_, column, span)| grid.span(column * per, (column + span) * per))
                .collect();
            let heights: Vec<u16> = row
                .iter()
                .zip(&spans)
                .map(|((s, _, _), (_, w))| Card::height(*s, opened.contains(s.id()), *w, m))
                .collect();
            let tall = heights.iter().copied().max().unwrap_or(0);
            for (((step, _, _), (x, w)), height) in row.into_iter().zip(spans).zip(heights) {
                places.push(Place {
                    step,
                    x,
                    y,
                    w,
                    height,
                    row: tall,
                });
            }
            y += tall + 1;
        }
        Layout { places, height: y }
    }
}

/// The tracks a card takes on `grid`: the fewest of three, four and six that hold a small card, else
/// all twelve.
pub(super) fn tracks(grid: Grid) -> u16 {
    [3, 4, 6]
        .into_iter()
        .find(|t| grid.span(0, *t).1 >= WIDTH - 1)
        .unwrap_or(TRACKS)
}
