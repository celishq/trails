use super::Cell;

/// A cell is lower case, with each run of other characters as one `-` and none at its ends
/// (`spec/05-patterns.md`, "Cells").
#[test]
fn a_cell_is_lower_case_words_joined_by_a_dash() {
    assert_eq!(Cell::new("CLI Help").unwrap().name(), "cli-help");
    assert_eq!(Cell::new("  --cli__help!! ").unwrap().name(), "cli-help");
    assert_eq!(Cell::new("Grüße 2").unwrap().name(), "grüße-2");
    assert_eq!(Cell::new("cli-help"), Cell::new("CLI Help"));
}

/// A cell with no letter or digit is no cell (`spec/05-patterns.md`, "Cells").
#[test]
fn a_cell_with_no_letter_or_digit_is_none() {
    assert_eq!(Cell::new(" - _ "), None);
    assert_eq!(Cell::new(""), None);
}

/// `CLI Help` and `cli-help` are one cell, and a cell with no letter or digit is left out
/// (`spec/05-patterns.md`, "Cells").
#[test]
fn all_keeps_each_cell_once() {
    let names: Vec<String> = Cell::all(["CLI Help", "-", "grep", "cli-help"])
        .iter()
        .map(|c| c.name().to_string())
        .collect();
    assert_eq!(names, ["cli-help", "grep"]);
}
