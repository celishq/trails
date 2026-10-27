use super::*;

/// The text of `lines`.
fn text(lines: Vec<Styled>) -> Vec<String> {
    lines.iter().map(|l| l.to_string()).collect()
}

/// Cards print side by side, a space between two, each row of cards as tall as its tallest card.
#[test]
fn cards_print_side_by_side() {
    let card = |rows: usize| Frame {
        title: "c".into(),
        rows: vec!["x".into(); rows],
        picked: false,
    };
    let lines = text(grid(&[card(1), card(2), card(1)], 2, 7));
    assert_eq!(
        lines,
        [
            "┌─ c ─┐ ┌─ c ─┐",
            "│ x   │ │ x   │",
            "│     │ │ x   │",
            "└─────┘ └─────┘",
            "┌─ c ─┐",
            "│ x   │",
            "└─────┘",
        ]
    );
}
