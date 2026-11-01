use super::*;

/// Spans of three, six and twelve tracks end in one column, so tiles, pairs and full cards share their
/// right edge at every width.
#[test]
fn spans_of_every_size_end_in_one_column() {
    for width in 32..=120 {
        let grid = Grid::of(width);
        let end = |(x, w): (u16, u16)| x + w;
        let full = end(grid.span(0, 12));
        assert_eq!(full, width - GUTTER, "{width}");
        assert_eq!(end(grid.span(6, 12)), full, "{width}");
        assert_eq!(end(grid.span(9, 12)), full, "{width}");
        assert_eq!(
            grid.span(0, 6).0 + grid.span(0, 6).1 + GUTTER,
            grid.span(6, 12).0
        );
    }
}

/// Each track starts where the one before it ends plus a gutter, and the first at the gutter.
#[test]
fn tracks_follow_one_another_by_a_gutter() {
    let grid = Grid::of(119);
    assert_eq!(grid.edge(0), GUTTER);
    for k in 0..TRACKS {
        let (x, w) = grid.span(k, k + 1);
        assert_eq!(x + w + GUTTER, grid.edge(k + 1), "{k}");
    }
}

/// The text column of a track is a border and a space past its edge; `after` finds the first at or
/// after a column.
#[test]
fn after_finds_the_next_track_column() {
    let grid = Grid::of(119);
    assert_eq!(grid.after(0), 4);
    assert_eq!(grid.after(4), 4);
    assert_eq!(grid.after(5), grid.edge(1) + GUTTER);
    assert_eq!(grid.after(200), grid.right());
}

/// `inside` gives the text column two tracks into a card, or the first track column past a long label.
#[test]
fn inside_steps_past_a_label_to_a_track() {
    let grid = Grid::of(119);
    let card = grid.edge(6);
    assert_eq!(grid.inside(card, 2, 0), grid.edge(8) + GUTTER);
    assert_eq!(
        grid.inside(card, 2, grid.edge(8) + 3),
        grid.edge(9) + GUTTER
    );
}
