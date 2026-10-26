use super::*;
use crate::command::read::stats::sheet::LEGEND;
use crate::terminal::canvas::paint;
use ratatui::style::Style;

/// A cell of `lead` and `change`.
fn cell(lead: &str, change: &str) -> Cell {
    let change = match change {
        "" => vec![],
        c => vec![paint(Style::new(), c)],
    };
    (vec![paint(Style::new(), lead)], change)
}

/// Cells that wrap stand in the same columns on every row, each on a track's text column, and every
/// change a gap after the widest lead of its column.
#[test]
fn cells_keep_their_columns_from_row_to_row() {
    let grid = Grid::of(79);
    let cells = [
        cell("44 resolved", "▼ 2"),
        cell("40 first pass", "▲ 8 pp"),
        cell("2 reworked", "▼ 6 pp"),
        cell("1 dropped", "▼ 2 pp"),
        cell("1 expired", ""),
    ];
    let table = Measures::of(grid, (4, 76), &cells, 0);
    let tracks: Vec<u16> = (0..TRACKS).map(|k| grid.column(k)).collect();
    assert!(table.rows > 1, "{:?}", table.places);
    let per = table.places.iter().filter(|(_, row)| *row == 0).count();
    for (n, (at, _)) in table.places.iter().enumerate() {
        assert!(tracks.contains(at), "{at}");
        assert_eq!(*at, table.places[n % per].0);
    }
    for (n, change) in table.changes.iter().enumerate() {
        assert_eq!(*change, table.changes[n % per], "{:?}", table.changes);
    }
    assert_eq!(table.changes[0], 4 + 11 + 2);
}

/// Legends of different names stand in the same columns, since every item takes `LEGEND` columns at
/// least.
#[test]
fn every_legend_keeps_the_same_columns() {
    for width in [50, 79, 119] {
        let grid = Grid::of(width);
        let right = width - 3;
        let outcomes = [cell("■ first pass", ""), cell("■ reworked", "")];
        let sources = [cell("■ reused", ""), cell("■ new", "")];
        let a = Measures::of(grid, (grid.column(1), right), &outcomes, LEGEND);
        let b = Measures::of(grid, (grid.column(1), right), &sources, LEGEND);
        assert_eq!(a.places, b.places, "{width}");
    }
}
