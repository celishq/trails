use super::*;
use crate::terminal::canvas::DIM;

fn lined_row(title: &str, meta: &str) -> Row {
    Row::Lined {
        lead: Styled::from("├─ "),
        marks: Styled::from("✓"),
        title: title.to_string(),
        tone: Style::new(),
        meta: Styled::from(paint(DIM, meta)),
        whole: None,
    }
}

/// A lined row is one line as wide as the width: its title, then its meta at the right edge.
#[test]
fn a_lined_row_puts_its_meta_at_the_right_edge() {
    let lines = lined_row("the step", "3fa2c1d0 · 4m").lines(1, Some(40), false);
    assert_eq!(lines.len(), 1);
    let text = lines[0].to_string();
    assert_eq!(chars(&lines[0]), 40, "{text:?}");
    assert!(text.starts_with("├─ ✓  the step"), "{text:?}");
    assert!(text.ends_with("3fa2c1d0 · 4m"), "{text:?}");
}

/// A title too long for the width is cut with …, and the row keeps to one line.
#[test]
fn a_long_title_is_cut_to_fit() {
    let title = "a title much longer than the width leaves room for";
    let lines = lined_row(title, "3fa2c1d0").lines(1, Some(40), false);
    assert_eq!(lines.len(), 1);
    assert_eq!(chars(&lines[0]), 40, "{:?}", lines[0].to_string());
    assert!(lines[0].to_string().contains("…"));
}

/// With less than 16 cells left for the title, the row drops its meta.
#[test]
fn a_narrow_row_drops_its_meta() {
    let lines =
        lined_row("the step's title", "3fa2c1d0 · revised by 9b0e11aa").lines(1, Some(30), false);
    let text = lines[0].to_string();
    assert!(!text.contains("3fa2c1d0"), "{text:?}");
    assert!(chars(&lines[0]) <= 30, "{text:?}");
}
