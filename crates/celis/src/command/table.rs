//! The rows behind a card of stats, as an opened card shows them and as Markdown.

use super::card::paint;
use super::tree::{cut, visible};

/// A table: the name of each column, each row with the id of the record it shows when it shows one, and
/// the column that takes the width the others leave.
pub(super) struct Table {
    pub head: Vec<&'static str>,
    pub rows: Vec<Row>,
    pub wide: usize,
}

/// One row of a table: the id of the record it shows, when it shows one, and its cells.
pub(super) struct Row {
    pub id: Option<String>,
    pub cells: Vec<String>,
}

impl Table {
    /// Its lines `width` wide: the names dim, then a row a line, each cell cut to its column, the
    /// `selected` row marked `▌` and bold.
    pub fn lines(&self, width: usize, selected: Option<usize>, colour: bool) -> Vec<String> {
        let mut widths: Vec<usize> = self.head.iter().map(|h| visible(h)).collect();
        for row in &self.rows {
            for (w, cell) in widths.iter_mut().zip(&row.cells) {
                *w = (*w).max(visible(cell));
            }
        }
        let others: usize = widths
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.wide)
            .map(|(_, w)| w + 2)
            .sum();
        widths[self.wide] = width.saturating_sub(others + 2).max(8);
        let line = |cells: &[String]| {
            let cells = cells.iter().zip(&widths).map(|(c, &w)| {
                let c = cut(c, w);
                format!("{c}{}", " ".repeat(w - visible(&c)))
            });
            cells.collect::<Vec<_>>().join("  ").trim_end().to_string()
        };
        let head: Vec<String> = self.head.iter().map(|h| h.to_string()).collect();
        let mut lines = vec![format!("  {}", paint(colour, "2", &line(&head)))];
        for (i, row) in self.rows.iter().enumerate() {
            lines.push(match Some(i) == selected {
                true => format!("▌ {}", paint(colour, "1", &line(&row.cells))),
                false => format!("  {}", line(&row.cells)),
            });
        }
        lines
    }

    /// It as a Markdown table under the heading `title`.
    pub fn markdown(&self, title: &str) -> String {
        let row = |cells: Vec<String>| format!("| {} |", cells.join(" | "));
        let mut lines = vec![
            format!("### {title}"),
            String::new(),
            row(self.head.iter().map(|h| h.to_string()).collect()),
            row(self.head.iter().map(|_| "---".to_string()).collect()),
        ];
        for r in &self.rows {
            lines.push(row(r.cells.iter().map(|c| c.replace('|', "\\|")).collect()));
        }
        lines.join("\n") + "\n"
    }
}
