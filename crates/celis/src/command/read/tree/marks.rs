//! The legend a tree watch shows above its keys: the marks a view prints.

use crate::progress::Phase;
use crate::terminal::canvas::{Styled, DIM, GREEN, YELLOW};
use crate::terminal::look::Look;
use ratatui::text::Span;

/// The states a step prints a mark for, each with its word.
const STATES: [(Phase, &str); 5] = [
    (Phase::Open, "open"),
    (Phase::Started, "started"),
    (Phase::Done, "done"),
    (Phase::Dropped, "dropped"),
    (Phase::Stale, "stale"),
];

/// The line of the legend: each state's mark as `look` draws it, then the signs of a revision and a
/// change; each mark in its look, each word dim.
pub(crate) fn marks(look: fn(Phase) -> Look) -> Vec<Styled> {
    let states = STATES.map(|(state, word)| (look(state).mark, look(state).text, word));
    let signs = [("↻", GREEN, "revised"), ("~", YELLOW, "changed")];
    let mut spans = vec![Span::raw("  ")];
    for (n, (mark, style, word)) in states.into_iter().chain(signs).enumerate() {
        if n > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(mark, style));
        spans.push(Span::styled(format!(" {word}"), DIM));
    }
    vec![Styled::from(spans)]
}
