use super::*;
use crate::terminal::canvas::line;

/// The text of `rows`, with no codes.
fn text(rows: &[Styled]) -> Vec<String> {
    rows.iter().map(|r| line(r, false)).collect()
}

/// A remark's strong words are the ones between `**`, each with the space before it, none where a word
/// is glued to the one before.
#[test]
fn a_remark_marks_its_strong_words() {
    let remark = Remark::new(Kind::Kept, "I kept **a rule**: take colours.", 0);
    let words: Vec<(&str, bool)> = remark.words.iter().map(|(w, b)| (w.as_str(), *b)).collect();
    assert_eq!(
        words,
        [
            ("I", false),
            (" kept", false),
            (" a", true),
            (" rule", true),
            (":", false),
            (" take", false),
            (" colours.", false)
        ]
    );
}

/// A remark prints one box holding the dude, a rule, its emoji and headline with a line after them, a
/// blank row and its words, every row as wide as the screen gives it, then a blank line.
#[test]
fn a_remark_prints_one_box_with_the_dude_and_its_words() {
    let remark = Remark::new(Kind::Learned, "When you add a page: **copy the frame**.", 0);
    let drawn = remark.rows(0, (None, false, 1.0), 80);
    let rows = text(&drawn);
    assert_eq!(rows.len(), ROWS);
    assert!(
        rows[0].starts_with("  ╭─") && rows[5].starts_with("  ╰─"),
        "{rows:?}"
    );
    assert!(
        rows[1].starts_with("  │ ▄    ▙▖  ├─ 💡 Learned ─"),
        "{rows:?}"
    );
    assert!(
        !rows[2].chars().any(char::is_alphanumeric)
            && rows[3].contains("  │ When you add a page: copy the frame. "),
        "{rows:?}"
    );
    assert!(rows[..6].iter().all(|r| visible(r) == 80), "{rows:?}");
    assert_eq!(rows[6], "");
    let title = drawn[1]
        .spans
        .iter()
        .find(|s| s.content.contains("Learned"))
        .unwrap();
    assert_eq!((title.style.fg, title.style.bg), (Some(VIOLET), None));
}

/// A remark wraps its words in two lines at most, the second cut with `…`.
#[test]
fn a_remark_wraps_in_two_lines_and_cuts_the_rest() {
    let text_ = "one two three four five six seven eight nine ten eleven twelve";
    let rows = text(&Remark::new(Kind::Standing, text_, 0).rows(0, (None, false, 1.0), 42));
    assert!(rows[3].contains("│ one two three four"), "{rows:?}");
    assert!(
        rows[4].trim_end_matches(['│', ' ']).ends_with('…'),
        "{rows:?}"
    );
    assert!(rows.iter().all(|r| visible(r) <= 42), "{rows:?}");
}

/// A remark prints its words italic, and types only the first of them with a caret after them while it
/// types.
#[test]
fn a_remark_types_its_italic_words_behind_a_caret() {
    let remark = Remark::new(Kind::Learned, "plain **strong**", 100);
    let whole = remark.rows(0, (None, false, 1.0), 80);
    let words = whole[3]
        .spans
        .iter()
        .find(|s| s.content.contains("strong"))
        .unwrap();
    assert_eq!(words.style, Style::new().add_modifier(Modifier::ITALIC));
    let typing = text(&remark.rows(0, (Some(4), true, 1.0), 80));
    assert!(typing[3].contains("│ plai▍ "), "{typing:?}");
    assert_eq!(remark.length(), "plain strong".len());
}
