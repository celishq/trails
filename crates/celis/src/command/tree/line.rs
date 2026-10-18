//! A line of a tree, and how a row lays out its title and details.

use super::cells::{Cells, COLUMNS, STEPS, TIME};
use super::{visible, BOLD, DIM, NARROWEST};

/// A line of the tree: text as it is, or a row whose details line up with the other rows' of its day,
/// its title in the colour `tone`, wrapped with its next lines after `rest`; or the start of a selected
/// element's lines, or of lines no element holds.
pub(super) enum Line {
    Text(String),
    Row {
        lead: String,
        title: String,
        tone: &'static str,
        cells: Cells,
        rest: String,
    },
    Mark(bool, Box<Line>),
}

impl Line {
    /// Its lead, title and details when it is a row, also when it is selected.
    pub fn row(&self) -> Option<(&String, &String, &Cells)> {
        match self {
            Line::Row {
                lead, title, cells, ..
            } => Some((lead, title, cells)),
            Line::Mark(_, line) => line.row(),
            Line::Text(_) => None,
        }
    }

    /// Its printed lines: a row's title, bold when `picked` unless dim, wrapped at `width`, then its
    /// details in the columns of `widths` up to its last one, each dim unless it has its own colour,
    /// ending at `edge`.
    pub fn lines(
        self,
        widths: &[usize; COLUMNS],
        (width, edge): (Option<usize>, usize),
        picked: bool,
        paint: &dyn Fn(&str, &str) -> String,
    ) -> Vec<String> {
        let (lead, title, tone, cells, rest) = match self {
            Line::Row {
                lead,
                title,
                tone,
                cells,
                rest,
            } => (lead, title, tone, cells, rest),
            Line::Text(text) => return vec![text],
            Line::Mark(_, line) => return line.lines(widths, (width, edge), picked, paint),
        };
        let tone = match picked && tone != DIM {
            true => BOLD,
            false => tone,
        };
        let cells = cells.columns();
        let filled = |n: &usize| cells[*n].is_some() && widths[*n] > 0;
        let (from, to) = ((0..COLUMNS).find(filled), (0..COLUMNS).rev().find(filled));
        let mut widths = *widths;
        widths[..from.unwrap_or(COLUMNS)].fill(0);
        widths[to.map_or(0, |to| to + 1)..].fill(0);
        let room = room(&widths);
        let mut titles = match width {
            Some(width) => wrap(
                &title,
                width.saturating_sub(visible(&lead) + room).max(NARROWEST),
            ),
            None => vec![title],
        };
        if titles.is_empty() {
            titles.push(String::new());
        }
        let block: Vec<String> = cells
            .iter()
            .zip(widths)
            .enumerate()
            .filter(|(_, (_, width))| *width > 0)
            .map(|(n, (cell, width))| {
                let cell = cell.unwrap_or("");
                let pad = " ".repeat(width - visible(cell));
                let cell = match cell.contains('\x1b') {
                    true => cell.to_string(),
                    false => paint(DIM, cell),
                };
                match n {
                    STEPS..=TIME => format!("{pad}{cell}"),
                    n if Some(n) == to => format!("{pad}{cell}"),
                    _ => format!("{cell}{pad}"),
                }
            })
            .collect();
        let block = block.join("  ");
        let first = &titles[0];
        let gap = edge.saturating_sub(visible(&lead) + visible(first) + room.saturating_sub(2));
        let first = match block.trim_end() {
            "" => format!("{lead}{}", paint(tone, first)),
            block => format!("{lead}{}{}{block}", paint(tone, first), " ".repeat(gap)),
        };
        let next = titles[1..]
            .iter()
            .map(|t| format!("{rest}{}", paint(tone, t)));
        std::iter::once(first).chain(next).collect()
    }
}

/// The characters the detail columns of `widths` take with the two spaces before them; none when all are
/// empty.
pub(super) fn room(widths: &[usize; COLUMNS]) -> usize {
    let shown = widths.iter().filter(|w| **w > 0).count();
    match shown {
        0 => 0,
        n => widths.iter().sum::<usize>() + 2 * n,
    }
}

/// `text` in lines of at most `width` characters, broken between words; a longer word stands alone.
pub(super) fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if visible(line) + 1 + visible(word) <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}
