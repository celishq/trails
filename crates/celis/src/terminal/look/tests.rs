use super::*;
use crate::terminal::canvas::line;

/// A check being drawn shows a stroke a frame, then `✓`.
#[test]
fn a_check_is_drawn_stroke_by_stroke() {
    let lines = vec![Styled::from(Span::styled(DRAWN, GREEN))];
    let at = |drawn: usize| line(&spinner(&lines, 0, drawn)[0], false);
    assert_eq!(at(0), "⠄");
    assert_eq!(at(4), "⢼");
    assert_eq!(at(5), "✓");
}

/// A shimmering title keeps its text, a bold band running across it, and loses its mark.
#[test]
fn a_shimmering_title_runs_a_bold_band_across_it() {
    let lines = vec![Styled::from(Span::styled(
        "the step",
        Style::new().add_modifier(SHIMMER),
    ))];
    let drawn = spinner(&lines, 2, 0);
    assert_eq!(line(&drawn[0], false), "the step");
    let bold: String = drawn[0]
        .spans
        .iter()
        .filter(|s| s.style.add_modifier.contains(Modifier::BOLD))
        .map(|s| s.content.as_ref())
        .collect();
    assert_eq!(bold, "the ");
    assert!(drawn[0]
        .spans
        .iter()
        .all(|s| !s.style.add_modifier.contains(SHIMMER)));
}
