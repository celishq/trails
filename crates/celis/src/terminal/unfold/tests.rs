use super::*;
use crate::terminal::canvas::line;

/// Rows arrive top down: at the start only the first shows, faded and cut short; a row starts a pace
/// after the one above it; every row is whole and in its own look once the arrival's length has passed.
#[test]
fn rows_fade_in_and_unfold_top_down() {
    let lines: Vec<Styled> = (0..3).map(|_| Styled::from("abcdefghij")).collect();
    let text = |at| -> Vec<String> { unfold(&lines, at).iter().map(|l| line(l, false)).collect() };
    let early = text(0.1);
    assert!(!early[0].is_empty() && early[0].len() < 10, "{early:?}");
    assert_eq!(early[2], "");
    assert_eq!(unfold(&lines, length(3)), lines);
}

/// A row on its way is a true colour between the background and its own.
#[test]
fn a_row_on_its_way_is_between_the_background_and_its_colour() {
    let lines = vec![Styled::from(Span::styled(
        "abc",
        Style::new().fg(Color::Green),
    ))];
    let half = unfold(&lines, ARRIVAL / 2.0);
    let Some(Color::Rgb(r, g, _)) = half[0].spans[0].style.fg else {
        panic!("{half:?}");
    };
    assert!(
        r > background().0 && r < 110 && g > background().1 && g < 205,
        "{half:?}"
    );
}

/// A fade starts at the background and ends in the lines' own looks.
#[test]
fn a_fade_goes_from_the_background_to_the_lines_own_looks() {
    let lines = vec![Styled::from(Span::styled(
        "abc",
        Style::new().fg(Color::Green),
    ))];
    let start = fade(&lines, 0.0, 0.5);
    let (r, g, b) = background();
    assert_eq!(start[0].spans[0].style.fg, Some(Color::Rgb(r, g, b)));
    assert_eq!(fade(&lines, 0.5, 0.5), lines);
}
