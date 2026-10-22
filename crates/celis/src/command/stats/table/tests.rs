use super::*;

/// A table prints as Markdown under its heading, a `|` in a cell escaped.
#[test]
fn a_table_prints_as_markdown() {
    let table = Table {
        head: vec!["Step", "Took"],
        rows: vec![vec!["a | b".into(), "3m".into()]],
    };
    assert_eq!(
        table.markdown("Route"),
        "### Route\n\n| Step | Took |\n| --- | --- |\n| a \\| b | 3m |\n"
    );
}
