use super::*;

/// A table of rules, with one rule too long for a line.
fn rules() -> Table {
    let row = |rule: &str, used: &str| Row {
        id: None,
        cells: vec![rule.to_string(), used.to_string()],
    };
    Table {
        head: vec!["Rule", "Used"],
        rows: vec![
            row(
                "When a rule is long: wrap it whole, because cut words hide it",
                "12",
            ),
            row("short", "3"),
        ],
        wide: 0,
    }
}

/// The text of `lines`, one string a line.
fn text(lines: &[Styled]) -> Vec<String> {
    lines
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.to_string()).collect())
        .collect()
}

/// An opened table wraps a long cell whole on more lines, keeps the other cells on its first line,
/// and marks each line of the selected row.
#[test]
fn a_table_wraps_a_long_cell_whole() {
    let (lines, first) = rules().lines(30, Some(0));
    let shown = text(&lines);
    assert_eq!(first, vec![1, 4]);
    assert_eq!(
        shown,
        vec![
            "  Rule                    Used",
            "▌ When a rule is long:    12",
            "▌ wrap it whole, because",
            "▌ cut words hide it",
            "  short                   3",
        ]
    );
}
