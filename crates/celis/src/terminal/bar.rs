//! A progress bar in the manner of rich: its parts in their states' colours, a half cell at a time.

use super::canvas::{paint, Styled, DIM, GREEN};
use super::look::SPINNER;

/// A bar `cells` wide, `done` of it green and `working` of it the spinner's cyan, each a share of one, the rest dim:
/// `━` a whole cell, `╸` a half, and a half-cell gap between the green and the cyan: the cyan's first
/// cell is `╺` wherever it starts.
pub(crate) fn bar(done: f64, working: f64, cells: usize) -> Styled {
    let halves = cells * 2;
    let green = (done.clamp(0.0, 1.0) * halves as f64).round() as usize;
    let cyan = ((done + working).clamp(0.0, 1.0) * halves as f64).round() as usize;
    let part = |half: usize| match half {
        h if h < green => 0,
        h if h < cyan => 1,
        _ => 2,
    };
    let looks = [GREEN, SPINNER, DIM];
    let mut before = None;
    let spans = (0..cells).map(|cell| {
        let (left, right) = (part(cell * 2), part(cell * 2 + 1));
        let glyph = match (left == 1 && before == Some(0), left == right) {
            (true, _) => "╺",
            (false, false) => "╸",
            (false, true) => "━",
        };
        before = Some(right);
        paint(looks[left], glyph)
    });
    Styled::from(spans.collect::<Vec<_>>())
}

#[cfg(test)]
mod tests;
