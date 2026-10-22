use super::*;

/// The text of `lines`.
fn text(lines: Vec<Styled>) -> Vec<String> {
    lines.iter().map(|l| l.to_string()).collect()
}

/// A card is framed at its width, its title in its top line, each row padded or cut to fit, and empty
/// rows to its height.
#[test]
fn a_card_frames_its_rows_at_its_width() {
    let card = Frame {
        title: "01 REUSE".into(),
        rows: vec!["one".into(), "a row too long to fit".into()],
        picked: false,
    };
    assert_eq!(
        text(card.lines(16, 5)),
        [
            "┌─ 01 REUSE ───┐",
            "│ one          │",
            "│ a row too l… │",
            "│              │",
            "└──────────────┘",
        ]
    );
}

/// The picked card is framed in heavy lines.
#[test]
fn the_picked_card_has_a_heavy_frame() {
    let card = Frame {
        title: "02".into(),
        rows: vec![],
        picked: true,
    };
    assert_eq!(text(card.lines(8, 3)), ["┏━ 02 ━┓", "┃      ┃", "┗━━━━━━┛"]);
}

/// A divider row draws a line across the card, joined to its frame.
#[test]
fn a_divider_row_crosses_the_card() {
    let card = Frame {
        title: "c".into(),
        rows: vec!["a".into(), RULE.into(), "b".into()],
        picked: false,
    };
    assert_eq!(
        text(card.lines(7, 5)),
        ["┌─ c ─┐", "│ a   │", "├─────┤", "│ b   │", "└─────┘"]
    );
}
