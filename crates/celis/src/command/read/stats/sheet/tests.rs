use super::*;
use crate::terminal::canvas::CYAN;

/// The text of each group `place` lays out, by row.
fn rows(groups: &[Group], w: u16) -> Vec<String> {
    let (placed, count) = place(groups, w);
    let mut rows = vec![String::new(); usize::from(count)];
    for (x, row, span) in placed {
        let line = &mut rows[usize::from(row)];
        let pad = usize::from(x).saturating_sub(visible(line));
        line.push_str(&" ".repeat(pad));
        line.push_str(&span.content);
    }
    rows
}

/// A group that passes the width moves whole to the next row, which drops the separator that would
/// lead it.
#[test]
fn a_group_that_does_not_fit_moves_whole_without_its_separator() {
    let groups = vec![
        vec![paint(BOLD, "48%"), paint(DIM, " reuse rate")],
        vec![
            paint(DIM, "  ·  "),
            paint(BOLD, "20"),
            paint(DIM, " follows"),
        ],
    ];
    assert_eq!(rows(&groups, 40), ["48% reuse rate  ·  20 follows"]);
    assert_eq!(rows(&groups, 20), ["48% reuse rate", "20 follows"]);
}

/// A stack fills its cells in each part's share, a part of half a share or more a cell at least, and
/// the largest takes what rounding leaves.
#[test]
fn a_stack_fills_its_cells_by_share() {
    let mut s = Sheet::new(20, 1);
    s.stack(0, 0, &[(40.0, CYAN), (2.0, DIM), (1.0, DIM)], 16);
    let line = s.lines(0, 1)[0].to_string();
    assert_eq!(line.trim_end().chars().count(), 16);
    let first = s.buf[(0, 0)].fg;
    let cyan = (0..16).filter(|x| s.buf[(*x, 0)].fg == first).count();
    assert_eq!(cyan, 14);
}

/// Big digits stand three rows high, a digit each four columns, a time's colon between them.
#[test]
fn big_digits_stand_three_rows_high() {
    let mut s = Sheet::new(20, 3);
    let end = s.big(0, 0, "3:49", Style::new());
    assert_eq!(end, 16);
    let rows: Vec<String> = s.lines(0, 3).iter().map(|l| l.to_string()).collect();
    assert_eq!(rows[0].trim_end(), "▀▀█     █ █ █▀█");
    assert_eq!(rows[2].trim_end(), "▀▀▀  ▀    ▀ ▀▀▀");
}

/// A sheet grows to the rows it draws on, and a clip keeps a text between its columns.
#[test]
fn a_sheet_grows_and_clips() {
    let mut s = Sheet::new(10, 1);
    s.clip = Some((2, 6));
    s.put(0, 40, "abcdefgh", Style::new());
    let row = s.lines(40, 41)[0].to_string();
    assert_eq!(row.trim_end(), "  cdef");
}

/// A card's name in its top border takes its own look, not the border's.
#[test]
fn a_card_name_keeps_its_own_look_over_the_border() {
    let mut s = Sheet::new(30, 4);
    s.card((0, 0, 30, 4), "Reuse rate", "");
    let name = &s.buf[(3, 0)];
    assert_eq!(name.symbol(), "R");
    assert_eq!(name.fg, ratatui::style::Color::Reset);
    assert_eq!(s.buf[(0, 0)].fg, EDGE.fg.unwrap());
}
