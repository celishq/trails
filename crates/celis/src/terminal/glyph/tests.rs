use super::*;

/// A number prints as digits three rows high, and the letters a space after it on the last row; text with
/// no digit prints on the middle row.
#[test]
fn a_number_prints_three_rows_high() {
    assert_eq!(digits("12m"), ["▀█  ▀▀█  ", " █  █▀▀  ", "▀▀▀ ▀▀▀ m"]);
    assert_eq!(digits("4%"), ["█ █  ", "▀▀█  ", "  ▀ %"]);
    assert_eq!(digits("0"), ["█▀█", "█ █", "▀▀▀"]);
    assert_eq!(digits("—"), [" ", "—", " "]);
}
