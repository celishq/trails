use super::*;

/// A number prints as digits three rows high, and the letters a space after it on the last row; text with
/// no digit prints on the middle row (`spec/02-cli.md`, "Cards").
#[test]
fn a_number_prints_three_rows_high() {
    assert_eq!(digits("12m"), ["▀█  ▀▀█  ", " █  █▀▀  ", "▀▀▀ ▀▀▀ m"]);
    assert_eq!(digits("4%"), ["█ █  ", "▀▀█  ", "  ▀ %"]);
    assert_eq!(digits("0"), ["█▀█", "█ █", "▀▀▀"]);
    assert_eq!(digits("—"), [" ", "—", " "]);
}

/// A bar fills its width in the share of `n` to `most`, in eighths of a character
/// (`spec/02-cli.md`, "Cards").
#[test]
fn a_bar_fills_its_share_in_eighths() {
    assert_eq!(bar(4, 4, 4), "████");
    assert_eq!(bar(1, 2, 4), "██  ");
    assert_eq!(bar(1, 3, 4), "█▎  ");
    assert_eq!(bar(0, 0, 3), "   ");
}

/// Columns print a value a column, the tallest the full height, in eighths of a row; a zero as its mark,
/// and any other value at least an eighth (`spec/02-cli.md`, "Cards").
#[test]
fn columns_print_each_value_as_a_column() {
    assert_eq!(columns(&[0, 1, 2, 4], 2, "·"), ["      █", "· ▄ █ █"]);
    assert_eq!(columns(&[0, 0], 2, "·"), ["   ", "· ·"]);
    assert_eq!(columns(&[1, 100], 1, "·"), ["▁ █"]);
}

/// A strip splits its width by the share of each part, the largest remainders first, with a shade a
/// part (`spec/02-cli.md`, "Cards").
#[test]
fn a_strip_splits_its_width_by_share() {
    assert_eq!(strip(&[3, 1], 8), "██████▓▓");
    assert_eq!(strip(&[1, 1, 1], 4), "██▓▒");
    assert_eq!(strip(&[0, 0], 4), "    ");
}
