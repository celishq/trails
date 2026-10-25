//! The steps linked to the step of a route: a line each, as the route lists them and as its Markdown.

use crate::terminal::canvas::{cut, paint, visible, Styled, DIM};
use crate::terminal::look::look;
use crate::text::line::time;
use celis_trails::Step;
use ratatui::text::Span;

/// The lines of a route, `width` wide: each linked step's line, the id it selects and its row of the
/// Markdown table; the line of the selected `row`, once it is added.
pub(super) struct Links {
    pub lines: Vec<Styled>,
    pub rows: Vec<Option<String>>,
    pub table: Vec<Vec<String>>,
    pub picked: Option<usize>,
    row: usize,
    width: usize,
}

impl Links {
    /// Lines after `lines`, the selected one the `row`th linked step.
    pub fn new(lines: Vec<Styled>, row: usize, width: usize) -> Links {
        Links {
            lines,
            rows: vec![],
            table: vec![],
            picked: None,
            row,
            width,
        }
    }

    /// Adds `at`, linked as `kind`, `depth` links away: its mark, title and time, `▌` when selected.
    pub fn add(&mut self, kind: &str, at: Step, depth: usize) {
        let look = look(at.state());
        let tail = time(at).unwrap_or_else(|| at.state().text().to_string());
        let lead = "  ".repeat(depth.saturating_sub(1));
        let room = self.width.saturating_sub(visible(&tail) + lead.len() + 8);
        let name = cut(at.title(), room);
        let pad = " ".repeat(room.saturating_sub(visible(&name)));
        let picked = self.rows.len() == self.row;
        if picked {
            self.picked = Some(self.lines.len());
        }
        let spans = vec![
            Span::raw(match picked {
                true => format!("▌ {lead}"),
                false => format!("  {lead}"),
            }),
            paint(look.text, look.mark),
            Span::raw(format!(" {name}{pad}  ")),
            paint(DIM, &tail),
        ];
        self.lines.push(Styled::from(spans));
        self.rows.push(Some(at.id().to_string()));
        self.table.push(vec![
            kind.to_string(),
            at.title().to_string(),
            at.record().anchor.name.clone(),
            time(at).unwrap_or("—".into()),
        ]);
    }
}
