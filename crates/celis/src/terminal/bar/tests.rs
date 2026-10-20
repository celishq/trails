use super::*;

/// A bar is as wide as its cells, green for done, cyan for working, the rest dim.
#[test]
fn a_bar_fills_its_parts_in_their_colours() {
    let line = bar(0.5, 0.25, 8);
    assert_eq!(line.to_string().chars().count(), 8);
    assert_eq!(line.spans[0].style, GREEN);
    assert_eq!(line.spans[7].style, DIM);
    assert!(line.spans.iter().any(|s| s.style == SPINNER));
}

/// A half-cell gap stands between the green and the cyan, also where they meet at a cell's edge.
#[test]
fn a_gap_stands_between_done_and_working() {
    let text = bar(0.5, 0.25, 8).to_string();
    assert_eq!(text, "━━━━╺━━━", "{text:?}");
    let text = bar(0.25, 0.25, 8).to_string();
    assert!(text.contains('╺') || text.contains('╸'), "{text:?}");
}

/// Where the cyan starts at a cell's edge and ends in that cell, the gap still stands before it.
#[test]
fn a_gap_stands_before_a_half_cell_of_working() {
    let line = bar(38.0 / 40.0, 1.0 / 40.0, 20);
    assert!(line.to_string().ends_with("━╺"), "{line:?}");
    assert_eq!(line.spans[19].style, SPINNER);
}
