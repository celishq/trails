//! The route of a step on a board: the steps it follows, the step, and the steps that follow it.

use super::super::table::{Row, Table};
use super::{head, legend, Board, NARROW};
use crate::progress::Model;
use crate::terminal::canvas::{cut, paint, visible, Styled, DIM};
use crate::terminal::card::Card;
use crate::terminal::ink::Ink;
use crate::terminal::live::Page;
use crate::terminal::look::look;
use crate::text::line::time;
use celis_trails::records::id::short;
use celis_trails::Step;
use ratatui::text::Span;

impl Board<'_> {
    /// The page of the route of `step`: the steps it follows, the step, and the steps that follow it.
    pub(super) fn route(&mut self, model: &Model, step: Step, width: usize) -> Page {
        let reuse = model.reuse;
        let up = reuse.up(step);
        let down = reuse.down(step);
        let count = up.len() + down.len();
        self.row = self.row.min(count.saturating_sub(1));
        let name = &step.record().anchor.name;
        let mut lines = head(Some(&short(step.id())), name, width);
        let mut rows = vec![];
        let mut table = vec![];
        let mut line = None;
        let mut item = |lines: &mut Vec<Styled>, kind: &str, at: Step, depth: usize| {
            let look = look(at.state());
            let tail = time(at).unwrap_or_else(|| at.state().text().to_string());
            let lead = "  ".repeat(depth.saturating_sub(1));
            let room = width.saturating_sub(visible(&tail) + lead.len() + 8);
            let name = cut(at.title(), room);
            let pad = " ".repeat(room.saturating_sub(visible(&name)));
            let picked = rows.len() == self.row;
            if picked {
                line = Some(lines.len());
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
            lines.push(Styled::from(spans));
            rows.push(Some(at.id().to_string()));
            table.push(Row {
                id: Some(at.id().to_string()),
                cells: vec![
                    kind.to_string(),
                    at.title().to_string(),
                    at.record().anchor.name.clone(),
                    time(at).unwrap_or("—".into()),
                ],
            });
        };
        if !up.is_empty() {
            lines.push(paint(DIM, "  FOLLOWS").into());
            for link in &up {
                item(&mut lines, "follows", link.source, link.depth);
            }
            lines.push(Styled::default());
        }
        let ink = Ink {
            plain: width < NARROW,
        };
        let took = time(step).unwrap_or("—".into());
        let side = [
            ink.dim("took").into(),
            ink.dim(&format!("revised {}", step.revised().len())).into(),
        ];
        let face = ink.beside(&took, &side, 3);
        let look = look(step.state());
        let card = Card {
            title: format!("{} {}", look.mark, step.title()),
            rows: face,
            picked: true,
        };
        lines.extend(card.lines(width, card.rows.len() + 2));
        if !down.is_empty() {
            let followed = paint(DIM, "  FOLLOWED BY").into();
            lines.extend([Styled::default(), followed]);
            for link in &down {
                item(&mut lines, "followed by", link.step, link.depth);
            }
        }
        if count == 0 {
            lines.extend([
                Styled::default(),
                paint(DIM, "  it follows no step, and no step follows it").into(),
            ]);
        }
        let copy = rows.get(self.row).cloned().flatten();
        let markdown = Table {
            head: vec!["Link", "Step", "Project", "Took"],
            rows: table,
            wide: 1,
        }
        .markdown(&format!("{} {}", short(step.id()), step.title()));
        self.rows = rows;
        Page {
            lines,
            picked: line,
            keys: legend("↑↓ select · enter go to · ← back", copy.as_deref()),
            copy: copy.map(|id| short(&id)),
            markdown: Some(markdown),
            ..Page::default()
        }
    }
}
