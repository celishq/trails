//! A row of a tree, and how a lined row and the notes under it lay out.

use super::{cut, visible};
use crate::terminal::canvas::{paint, width as chars, Styled, BOLD, DIM};
use ratatui::style::Style;

/// A row of the tree: text as it is; a lined row, its title cut to one line after `lead` and its
/// `marks`, with `meta` at the right, or whole, wrapped after `whole`, when it has one; a note under a
/// row, after `rail` and `label`, in the look `tone`; or the start of a selected element's lines, or of
/// lines no element holds.
pub(super) enum Row {
    Text(Styled),
    Lined {
        lead: Styled,
        marks: Styled,
        title: String,
        tone: Style,
        meta: Styled,
        whole: Option<Styled>,
    },
    Note {
        rail: Styled,
        label: &'static str,
        text: String,
        tone: Style,
    },
    Mark(bool, Box<Row>),
}

impl Row {
    /// It moved right by `inset`: each lead, rail and text starts with it, a blank line kept blank.
    pub fn indent(&mut self, inset: &str) {
        let before = |line: &mut Styled| line.spans.insert(0, inset.to_string().into());
        match self {
            Row::Text(text) if text.spans.is_empty() => {}
            Row::Text(text) => before(text),
            Row::Lined { lead, whole, .. } => {
                before(lead);
                if let Some(whole) = whole {
                    before(whole);
                }
            }
            Row::Note { rail, .. } => before(rail),
            Row::Mark(_, line) => line.indent(inset),
        }
    }

    /// Its printed lines, its marks padded to `slot` and wrapped at `width`: a row's title bold when
    /// `picked` unless dim; a note's text after its label padded to 8 characters.
    pub fn lines(self, slot: usize, width: Option<usize>, picked: bool) -> Vec<Styled> {
        let under = |rail: Styled, pad: usize| {
            let mut rail = rail;
            rail.push_span(" ".repeat(slot.saturating_sub(1) + pad));
            rail
        };
        let (first, next, text, tone) = match self {
            Row::Text(text) => return vec![text],
            Row::Mark(_, line) => return line.lines(slot, width, picked),
            Row::Lined {
                lead,
                marks,
                title,
                tone,
                meta,
                whole,
            } => {
                return lined(
                    (lead, whole),
                    marks,
                    slot,
                    (&title, tone, picked),
                    meta,
                    width,
                )
            }
            Row::Note {
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

/// The fewest free cells a title keeps before a row drops its meta.
const ROOM: usize = 16;

/// The fewest free cells a title wraps in; with fewer it is cut.
const NARROW: usize = 8;

/// One line: `lead`, `marks` padded to `slot` and a space, at least one, none with no marks, the title in its look,
/// bold when `picked` unless dim, cut with … to fit `width`, then `meta` at the right edge; with less than `ROOM` cells for
/// the title, no meta. With a `whole` lead and at least `NARROW` cells, the title wrapped instead, its next lines
/// after that lead.
fn lined(
    (lead, whole): (Styled, Option<Styled>),
    marks: Styled,
    slot: usize,
    (title, tone, picked): (&str, Style, bool),
    meta: Styled,
    width: Option<usize>,
) -> Vec<Styled> {
    let tone = match picked && tone != DIM {
        true => BOLD,
        false => tone,
    };
    let mut line = lead;
    let gap = match chars(&marks) {
        0 => 0,
        wide => (slot + 1).saturating_sub(wide).max(1),
    };
    line.spans.extend(marks.spans);
    line.push_span(" ".repeat(gap));
    let Some(width) = width else {
        line.push_span(paint(tone, title));
        if chars(&meta) > 0 {
            line.push_span("  ");
            line.spans.extend(meta.spans);
        }
        return vec![line];
    };
    let used = chars(&line);
    let (meta, room) = match width.saturating_sub(used + chars(&meta) + 2) {
        room if room >= ROOM || chars(&meta) == 0 => (meta, room),
        _ => (Styled::default(), width.saturating_sub(used)),
    };
    let parts = match whole {
        Some(_) if visible(title) > room && room >= NARROW => wrap(title, room),
        _ => vec![cut(title, room)],
    };
    let first = parts.first().cloned().unwrap_or_default();
    line.push_span(paint(tone, &first));
    let gap = width.saturating_sub(used + visible(&first) + chars(&meta));
    if chars(&meta) > 0 {
        line.push_span(" ".repeat(gap.max(2)));
        line.spans.extend(meta.spans);
    }
    let mut lines = vec![line];
    for part in parts.into_iter().skip(1) {
        let mut next = whole.clone().unwrap_or_default();
        next.push_span(" ".repeat(used.saturating_sub(chars(&next))));
        next.push_span(paint(tone, &part));
        lines.push(next);
    }
    lines
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

#[cfg(test)]
mod tests;
