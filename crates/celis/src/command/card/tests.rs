use super::*;

/// A card is framed at its width, its title in its top line, each row padded or cut to fit, and empty
/// rows to its height (`spec/02-cli.md`, "Cards").
#[test]
fn a_card_frames_its_rows_at_its_width() {
    let card = Card {
        title: "01 REUSE".into(),
        rows: vec!["one".into(), "a row too long to fit".into()],
        picked: false,
    };
    assert_eq!(
        card.lines(16, 5, false),
        [
            "┌─ 01 REUSE ───┐",
            "│ one          │",
            "│ a row too l… │",
            "│              │",
            "└──────────────┘",
        ]
    );
}

/// The picked card is framed in heavy lines (`spec/02-cli.md`, "Cards").
#[test]
fn the_picked_card_has_a_heavy_frame() {
    let card = Card {
        title: "02".into(),
        rows: vec![],
        picked: true,
    };
    assert_eq!(
        card.lines(8, 3, false),
        ["┏━ 02 ━┓", "┃      ┃", "┗━━━━━━┛"]
    );
}

/// Cards print side by side, a space between two, each row of cards as tall as its tallest card
/// (`spec/02-cli.md`, "Cards").
#[test]
fn cards_print_side_by_side() {
    let card = |rows: usize| Card {
        title: "c".into(),
        rows: vec!["x".into(); rows],
        picked: false,
    };
    let lines = grid(&[card(1), card(2), card(1)], 2, 7, false);
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

/// A divider row draws a line across the card, joined to its frame (`spec/02-cli.md`, "Cards").
#[test]
fn a_divider_row_crosses_the_card() {
    let card = Card {
        title: "c".into(),
        rows: vec!["a".into(), RULE.into(), "b".into()],
        picked: false,
    };
    assert_eq!(
        card.lines(7, 5, false),
        ["┌─ c ─┐", "│ a   │", "├─────┤", "│ b   │", "└─────┘"]
    );
}
