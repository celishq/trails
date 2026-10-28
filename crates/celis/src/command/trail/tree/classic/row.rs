//! A row of a tree, and how a titled row and the notes under it lay out.

use super::visible;
use crate::terminal::canvas::{paint, width as chars, Styled, BOLD, DIM};
use ratatui::style::Style;

/// A row of the tree: text as it is; a titled row, its title in the look `tone` after `lead` and its
/// `marks`, wrapped with its next lines after `rest`; a note under a row, after `rail` and `label`, in
/// the look `tone`; or the start of a selected element's lines, or of lines no element holds.
pub(super) enum Row {
    Text(Styled),
    Titled {
        lead: Styled,
        marks: Styled,
        title: String,
        tone: Style,
        rest: Styled,
    },
    Note {
        rail: Styled,
        label: String,
        text: String,
        tone: Style,
    },
    Mark(bool, Box<Row>),
}

impl Row {
    /// Pads the labels of each run of notes to the longest of them.
    pub fn aligned(rows: &mut [Row]) {
        let notes = |a: &Row, b: &Row| matches!((a, b), (Row::Note { .. }, Row::Note { .. }));
        for run in rows.chunk_by_mut(notes) {
            let label = |row: &Row| match row {
                Row::Note { label, .. } => label.chars().count(),
                _ => 0,
            };
            let width = run.iter().map(label).max().unwrap_or(0);
            for row in run {
                if let Row::Note { label, .. } = row {
                    if !label.is_empty() {
                        *label = format!("{label:<width$}");
                    }
                }
            }
        }
    }

    /// Its printed lines, its marks padded to `slot` and wrapped at `width`: a row's title bold when
    /// `picked` unless dim; a note's text after its label padded to 8 characters or more.
    pub fn lines(self, slot: usize, width: Option<usize>, picked: bool) -> Vec<Styled> {
        let under = |rail: Styled, pad: usize| {
            let mut rail = rail;
            rail.push_span(" ".repeat(slot.saturating_sub(1) + pad));
            rail
        };
        let (first, next, text, tone) = match self {
            Row::Text(text) => return vec![text],
            Row::Mark(_, line) => return line.lines(slot, width, picked),
            Row::Titled {
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
            Row::Note {
                rail,
                label,
                text,
                tone,
            } => match label.as_str() {
                "" => (under(rail.clone(), 0), under(rail, 0), text, tone),
                _ => {
                    let pad = label.chars().count().max(8) + 1;
                    let mut first = under(rail.clone(), 0);
                    first.push_span(paint(DIM, &format!("{label:<pad$}")));
                    (first, under(rail, pad), text, tone)
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
