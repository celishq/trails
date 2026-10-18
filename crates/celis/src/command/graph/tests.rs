use super::*;

fn columns() -> Vec<Vec<String>> {
    let column = |ids: &[&str]| ids.iter().map(|id| id.to_string()).collect();
    vec![
        column(&["a", "a1", "a2"]),
        column(&["b"]),
        column(&["c", "c1"]),
    ]
}

/// The arrows move the selection up and down a column, and to the same height in the column before or
/// after, kept inside the graph; with nothing selected, they select the first card.
#[test]
fn the_arrows_select_the_step_next_to_the_selected_one() {
    let columns = columns();
    let moved = |from: Option<&str>, key| next(&columns, from, key);
    assert_eq!(moved(None, Key::Down).as_deref(), Some("a"));
    assert_eq!(moved(Some("a"), Key::Down).as_deref(), Some("a1"));
    assert_eq!(moved(Some("a2"), Key::Down).as_deref(), Some("a2"));
    assert_eq!(moved(Some("a"), Key::Up).as_deref(), Some("a"));
    assert_eq!(moved(Some("a2"), Key::Right).as_deref(), Some("b"));
    assert_eq!(moved(Some("c1"), Key::Left).as_deref(), Some("b"));
    assert_eq!(moved(Some("c"), Key::Right).as_deref(), Some("c"));
    assert_eq!(moved(Some("a1"), Key::Left).as_deref(), Some("a1"));
}
