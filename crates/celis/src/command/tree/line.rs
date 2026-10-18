//! A line of a tree, and how a row and the notes under it lay out.

use super::{visible, BOLD, DIM};

/// A line of the tree: text as it is; a row, its title in the colour `tone` after `lead` and its
/// `marks`, wrapped with its next lines after `rest`; a note under a row, after `rail` and `label`, in
/// the colour `tone`; or the start of a selected element's lines, or of lines no element holds.
pub(super) enum Line {
    Text(String),
    Row {
        lead: String,
        marks: String,
        title: String,
        tone: &'static str,
        rest: String,
    },
    Note {
        rail: String,
        label: &'static str,
        text: String,
        tone: &'static str,
    },
    Mark(bool, Box<Line>),
}

impl Line {
    /// Its marks when it is a row, also when it is selected.
    pub fn marks(&self) -> Option<&str> {
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
        paint: &dyn Fn(&str, &str) -> String,
    ) -> Vec<String> {
        let under = |rail: &str| format!("{rail}{}", " ".repeat(slot.saturating_sub(1)));
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
                let pad = " ".repeat(slot.saturating_sub(visible(&marks)));
                let tone = match picked && tone != DIM {
                    true => BOLD,
                    false => tone,
                };
                (format!("{lead}{marks}{pad} "), under(&rest), title, tone)
            }
            Line::Note {
                rail,
                label,
                text,
                tone,
            } => match label {
                "" => (under(&rail), under(&rail), text, tone),
                _ => (
                    format!("{}{}", under(&rail), paint(DIM, &format!("{label:<9}"))),
                    format!("{}{}", under(&rail), " ".repeat(9)),
                    text,
                    tone,
                ),
            },
        };
        let mut parts = match width {
            Some(width) => wrap(&text, width.saturating_sub(visible(&first))),
            None => vec![text],
        };
        if parts.is_empty() {
            parts.push(String::new());
        }
        let lead = std::iter::once(first).chain(std::iter::repeat(next));
        lead.zip(parts)
            .map(|(lead, part)| format!("{lead}{}", paint(tone, &part)))
            .collect()
    }
}

/// `text` in lines of at most `width` characters, at least one, broken between words; a longer word is
/// broken across lines.
fn wrap(text: &str, width: usize) -> Vec<String> {
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
