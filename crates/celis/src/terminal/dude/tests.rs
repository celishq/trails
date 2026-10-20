use super::*;

/// The dude is three rows of seven cells: its body a white background, its edges white quadrants.
#[test]
fn the_dude_is_three_rows_of_seven_cells() {
    let rows: Vec<String> = dude(0)
        .iter()
        .map(|row| row.spans.iter().map(|s| s.content.as_ref()).collect())
        .collect();
    assert_eq!(rows, ["▄    ▙▖", "       ", " ▀    ▀"]);
}

/// The dude blinks: its eyes shut for a tick near the end of each blink's span, then open again.
#[test]
fn the_dude_blinks_and_opens_its_eyes_again() {
    let (open, shut) = (dude(0), dude(BLINK.1 - 1));
    assert_ne!(open[1], shut[1]);
    assert_eq!(open, dude(BLINK.1));
}
