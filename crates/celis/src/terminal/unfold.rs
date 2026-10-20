//! Rows that arrive on a live screen as the demo draws them: each fades in from the background and
//! unfolds from its left edge, a little after the one above it.

use super::background::background;
use super::canvas::{width, Styled};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use unicode_width::UnicodeWidthChar;

/// The seconds between the starts of two rows.
const PACE: f64 = 0.08;

/// The seconds a row takes to arrive whole.
const ARRIVAL: f64 = 0.42;

/// The seconds the rows `count` take to arrive, the last one whole.
pub(crate) fn length(count: usize) -> f64 {
    count as f64 * PACE + ARRIVAL
}

/// `lines` `at` seconds into their arrival: each row starts `PACE` after the one above it, fades in
/// from the background over `ARRIVAL` and shows a share of its cells that eases to the whole.
pub(crate) fn unfold(lines: &[Styled], at: f64) -> Vec<Styled> {
    lines
        .iter()
        .enumerate()
        .map(|(n, line)| {
            let alpha = smooth((at - n as f64 * PACE) / ARRIVAL);
            if alpha >= 1.0 {
                return line.clone();
            }
            let shown = (width(line) as f64 * ease(alpha)).ceil() as usize;
            faded(&cells(line, shown), alpha)
        })
        .collect()
}

/// `lines` faded in `at` seconds into a fade of `length` seconds: each colour on its way from the
/// background to its own, eased in and out; whole once the fade has passed.
pub(crate) fn fade(lines: &[Styled], at: f64, length: f64) -> Vec<Styled> {
    faded_by(lines, smooth(at / length))
}

/// `lines` with each colour `alpha` of the way from the background to its own; whole at 1.
pub(crate) fn faded_by(lines: &[Styled], alpha: f64) -> Vec<Styled> {
    match alpha >= 1.0 {
        true => lines.to_vec(),
        false => lines.iter().map(|line| faded(line, alpha)).collect(),
    }
}

/// The true colour `alpha` of the way from the background to `colour`; `colour` itself at 1.
pub(crate) fn toward(colour: Color, alpha: f64) -> Color {
    match alpha >= 1.0 {
        true => colour,
        false => blend(rgb(Some(colour), false, true), alpha.max(0.0)),
    }
}

/// `x` held between 0 and 1, eased in and out.
pub(crate) fn smooth(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// `x` held between 0 and 1, eased out.
fn ease(x: f64) -> f64 {
    1.0 - (1.0 - x.clamp(0.0, 1.0)).powi(3)
}

/// The first `count` cells of `line`, each span in its look.
fn cells(line: &Styled, count: usize) -> Styled {
    let mut left = count;
    let mut spans = vec![];
    for span in &line.spans {
        let mut text = String::new();
        for c in span.content.chars() {
            let size = c.width().unwrap_or(0);
            if size > left {
                left = 0;
                break;
            }
            left -= size;
            text.push(c);
        }
        if !text.is_empty() {
            spans.push(Span::styled(text, span.style));
        }
        if left == 0 {
            break;
        }
    }
    Styled::from(spans)
}

/// `line` with each colour `alpha` of the way from the background to its own, in true colour.
fn faded(line: &Styled, alpha: f64) -> Styled {
    let spans = line.spans.iter().map(|span| {
        let style = span.style;
        let dim = style.add_modifier.contains(Modifier::DIM);
        let fg = blend(rgb(style.fg, dim, true), alpha);
        let mut look = Style::new()
            .fg(fg)
            .add_modifier(style.add_modifier - Modifier::DIM);
        if style.bg.is_some_and(|c| c != Color::Reset) {
            look = look.bg(blend(rgb(style.bg, false, false), alpha));
        }
        Span::styled(span.content.clone(), look)
    });
    Styled::from(spans.collect::<Vec<_>>())
}

/// The colour `alpha` of the way from the background to `to`.
fn blend((r, g, b): (u8, u8, u8), alpha: f64) -> Color {
    let mix = |from: u8, to: u8| (from as f64 + (to as f64 - from as f64) * alpha).round() as u8;
    let (br, bg, bb) = background();
    Color::Rgb(mix(br, r), mix(bg, g), mix(bb, b))
}

/// The true colour of `colour` as the demo paints it, toward the background when `dim`; the text's
/// own colour when none and `text`.
fn rgb(colour: Option<Color>, dim: bool, text: bool) -> (u8, u8, u8) {
    let full = match colour {
        Some(Color::Rgb(r, g, b)) => (r, g, b),
        Some(Color::Red | Color::LightRed) => (230, 100, 100),
        Some(Color::Green | Color::LightGreen) => (110, 205, 125),
        Some(Color::Yellow | Color::LightYellow) => (225, 190, 100),
        Some(Color::Magenta | Color::LightMagenta) => (200, 120, 220),
        Some(Color::Cyan | Color::LightCyan | Color::Blue | Color::LightBlue) => (90, 200, 220),
        Some(Color::White) => (244, 245, 247),
        Some(Color::Gray | Color::DarkGray) => (120, 126, 142),
        Some(Color::Black) => background(),
        _ if text => (220, 223, 230),
        _ => background(),
    };
    match dim {
        true => {
            let (r, g, b) = background();
            let half = |a: u8, b: u8| ((a as u16 + b as u16) / 2) as u8;
            (half(full.0, r), half(full.1, g), half(full.2, b))
        }
        false => full,
    }
}

#[cfg(test)]
mod tests;
