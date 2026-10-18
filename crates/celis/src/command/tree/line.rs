//! A line of a tree, and how a row and the notes under it lay out.

use super::visible;
use crate::terminal::canvas::{width as chars, Styled, BOLD, DIM};
use ratatui::style::Style;
use ratatui::text::Span;

/// A line of the tree: text as it is; a row, its title in the look `tone` after `lead` and its
/// `marks`, wrapped with its next lines after `rest`; a note under a row, after `rail` and `label`, in
/// the look `tone`; or the start of a selected element's lines, or of lines no element holds.
pub(super) enum Line {
    Text(Styled),
    Row {
        lead: Styled,
        marks: Styled,
        title: String,
        tone: Style,
        rest: Styled,
    },
    Note {
        rail: Styled,
        label: &'static str,
        text: String,
        tone: Style,
    },
    Mark(bool, Box<Line>),
}

impl Line {
    /// Its marks when it is a row, also when it is selected.
    pub fn marks(&self) -> Option<&Styled> {
        match self {
            Line::Row { marks, .. } => Some(marks),
            Line::Mark(_, line) => line.marks(),
            _ => None,
        }
    }

    /// Its printed lines, its marks padded to `slot` and wrapped at `width`: a row's title bold when
    /// `picked` unless dim; a note's text after its label padded to 8 characters.
    pub fn lines(
        self,
        slot: usize,
        width: Option<usize>,
        picked: bool,
        paint: &dyn Fn(Style, &str) -> Span<'static>,
    ) -> Vec<Styled> {
        let under = |rail: Styled, pad: usize| {
            let mut rail = rail;
            rail.push_span(" ".repeat(slot.saturating_sub(1) + pad));
            rail
        };
        let (first, next, text, tone) = match self {
            Line::Text(text) => return vec![text],
            Line::Mark(_, line) => return line.lines(slot, width, picked, paint),
            Line::Row {
                lead,
                marks,
                title,
                tone,
                rest,
            } => {
                let pad = " ".repeat(slot.saturating_sub(chars(&marks)));
                let tone = match picked && tone != DIM {
                    true => BOLD,
                    false => tone,
                };
                let mut first = lead;
                first.spans.extend(marks.spans);
                first.push_span(format!("{pad} "));
                (first, under(rest, 0), title, tone)
            }
            Line::Note {
                rail,
                label,
                text,
                tone,
            } => match label {
                "" => (under(rail.clone(), 0), under(rail, 0), text, tone),
                _ => {
                    let mut first = under(rail.clone(), 0);
                    first.push_span(paint(DIM, &format!("{label:<9}")));
                    (first, under(rail, 9), text, tone)
                }
            },
        };
        let mut parts = match width {
            Some(width) => wrap(&text, width.saturating_sub(chars(&first))),
            None => vec![text],
        };
        if parts.is_empty() {
            parts.push(String::new());
        }
        let lead = std::iter::once(first).chain(std::iter::repeat(next));
        lead.zip(parts)
            .map(|(mut lead, part)| {
                lead.push_span(paint(tone, &part));
                lead
            })
            .collect()
    }
}

/// `text` in lines of at most `width` characters, at least one, broken between words; a longer word is
/// broken across lines.
pub(crate) fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines: Vec<String> = vec![];
    for word in text.split_whitespace() {
        let chars: Vec<char> = word.chars().collect();
        for part in chars.chunks(width).map(String::from_iter) {
            match lines.last_mut() {
                Some(line) if visible(line) + 1 + visible(&part) <= width => {
                    line.push(' ');
                    line.push_str(&part);
                }
                _ => lines.push(part),
            }
        }
    }
    lines
}
