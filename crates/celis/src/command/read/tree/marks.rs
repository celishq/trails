//! The legend a tree watch shows above its keys: what celis keeps, and the marks a view prints.

use crate::progress::Phase;
use crate::terminal::canvas::{Styled, DIM, GREEN, MAGENTA, YELLOW};
use crate::terminal::look::Look;
use ratatui::style::Style;
use ratatui::text::Span;

/// What celis keeps: each one's mark, the look of its mark, and its word.
const WORDS: [(&str, Style, &str); 3] = [
    ("§", MAGENTA, "pattern"),
    ("◆", MAGENTA, "trail"),
    ("○", Style::new(), "step"),
];

/// The states a step prints a mark for, each with its word.
const STATES: [(Phase, &str); 5] = [
    (Phase::Open, "open"),
    (Phase::Started, "started"),
    (Phase::Done, "done"),
    (Phase::Dropped, "dropped"),
    (Phase::Stale, "stale"),
];

/// The two lines of the legend: what celis keeps, then each state's mark as `look` draws it and the
/// signs of a revision and a change; each mark in its look, each word dim.
pub(crate) fn marks(look: fn(Phase) -> Look) -> Vec<Styled> {
    let states = STATES.map(|(state, word)| (look(state).mark, look(state).text, word));
    let signs = [("↻", GREEN, "revised"), ("~", YELLOW, "changed")];
    let line = |marks: Vec<(&'static str, Style, &str)>| {
        let mut spans = vec![Span::raw("  ")];
        for (n, (mark, style, word)) in marks.into_iter().enumerate() {
            if n > 0 {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(mark, style));
            spans.push(Span::styled(format!(" {word}"), DIM));
        }
        Styled::from(spans)
    };
    vec![
        line(WORDS.to_vec()),
        line(states.into_iter().chain(signs).collect()),
    ]
}
