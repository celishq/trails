//! A table of measures on the grid: each measure in a cell on a track column, its change at one offset.

use super::sheet::Group;
use crate::terminal::canvas::visible;
use crate::terminal::grid::{Grid, GUTTER, TRACKS};
use ratatui::text::Span;

/// A cell of a table of measures: its lead, and the change or share after it, empty when it has none.
pub(super) type Cell = (Group, Group);

/// Where the cells of a table of measures stand on the grid: each cell's column and row, the column of
/// its change, a gap after the widest lead of its column, and the rows they take.
pub(super) struct Measures {
    pub places: Vec<(u16, u16)>,
    pub changes: Vec<u16>,
    pub rows: u16,
}

impl Measures {
    /// The places of `cells` on `grid` from `x`, before `right`: every cell the same number of tracks
    /// wide, the fewest that hold the widest cell or `least` and a gutter, as many to a row as end before
    /// `right`, at least one.
    pub fn of(grid: Grid, (x, right): (u16, u16), cells: &[Cell], least: u16) -> Measures {
        let lead = cells.iter().map(|(l, _)| wide(l)).max().unwrap_or(0);
        let change = cells
            .iter()
            .map(|(_, c)| wide(&trimmed(c)))
            .max()
            .unwrap_or(0);
        let cell = match change {
            0 => lead,
            _ => lead + 2 + change,
        }
        .max(least);
        let first = grid.track(x);
        let pitch = (0..TRACKS)
            .map(|k| grid.edge(k + 1) - grid.edge(k))
            .min()
            .unwrap_or(1)
            .max(1);
        let tracks = (cell + GUTTER).div_ceil(pitch).max(1);
        let per = (0..TRACKS)
            .take_while(|n| {
                let k = first + tracks * n;
                k < TRACKS && grid.column(k) + cell <= right
            })
            .count()
            .max(1) as u16;
        let places: Vec<(u16, u16)> = (0..cells.len() as u16)
            .map(|n| (grid.column(first + tracks * (n % per)).max(x), n / per))
            .collect();
        let widest = |column: usize| {
            cells
                .iter()
                .skip(column)
                .step_by(usize::from(per))
                .map(|(l, _)| wide(l))
                .max()
                .unwrap_or(0)
        };
        let changes = places
            .iter()
            .enumerate()
            .map(|(n, (at, _))| at + widest(n % usize::from(per)) + 2)
            .collect();
        Measures {
            places,
            changes,
            rows: (cells.len() as u16).div_ceil(per).max(1),
        }
    }
}

/// The columns `group` takes.
fn wide(group: &Group) -> u16 {
    group.iter().map(|s| visible(&s.content) as u16).sum()
}

/// `group` with the spaces before its first text left out.
pub(super) fn trimmed(group: &Group) -> Group {
    let mut group = group.clone();
    if let Some(first) = group.first_mut() {
        *first = Span::styled(first.content.trim_start().to_string(), first.style);
    }
    group
}

#[cfg(test)]
mod tests;
