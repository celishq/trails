//! The folded line of a level: its patterns line, open or closed.

use crate::command::read::tree::row::Row;
use crate::command::read::tree::{after, details, Item, Tree, PATTERNS};
use crate::progress::Phase;
use crate::progress::Tally;
use crate::terminal::canvas::{paint, Styled, DIM, MAGENTA};
use crate::terminal::look::look;
use crate::text::line::plural;
use celis_nostr::Note;
use ratatui::style::Style;
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds the patterns line `§ <n> patterns` of the row `trail` after `rail`, the `last` of its level;
    /// open, a line under it for each of `patterns`: `§ <When>`, wrapped to fit, which opens to the
    /// `details` of its other lines.
    pub(super) fn patterns_line(
        &self,
        patterns: Vec<&Note>,
        rail: &Styled,
        last: bool,
        trail: &str,
        lines: &mut Vec<Row>,
    ) {
        let id = format!("{PATTERNS}{trail}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, last, Some(open), self.lane());
        let mark = paint(MAGENTA, "§");
        let first = lines.len();
        lines.push(Row::Lined {
            lead,
            marks: Styled::from(mark.clone()),
            title: plural(patterns.len(), "pattern"),
            tone: Style::new(),
            meta: Styled::default(),
            whole: None,
        });
        let item = Item {
            id: id.clone(),
            open,
            parent: Some(trail.to_string()),
            under: patterns.iter().map(|p| p.id.clone()).collect(),
        };
        self.pick(item, lines, first);
        if !open {
            return;
        }
        let all = patterns.len();
        for (n, record) in patterns.into_iter().enumerate() {
            let more = details(record.content.lines().skip(1));
            let toggle = (!more.is_empty()).then(|| self.open(&record.id));
            let (lead, under) = self.branch(&rail, n + 1 == all, toggle, self.lane());
            let first = lines.len();
            lines.push(Row::Lined {
                lead,
                marks: Styled::from(mark.clone()),
                title: record
                    .content
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                tone: Style::new(),
                meta: Styled::default(),
                whole: Some(under.clone()),
            });
            let item = Item {
                id: record.id.clone(),
                open: toggle == Some(true),
                parent: Some(id.clone()),
                under: vec![],
            };
            self.pick(item, lines, first);
            if toggle == Some(true) {
                let below = after(&under, Span::raw("  "));
                for (label, text) in more {
                    self.labelled(lines, &below, label, text);
                }
            }
        }
    }
}

/// The closed steps of `tally` counted by state, each count after its mark (`✓ 7 · ✗ 1 · ◌ 2`).
pub(in crate::command::read::tree) fn counts(tally: Tally) -> Vec<Span<'static>> {
    let mut counts: Vec<Span> = vec![];
    for state in [Phase::Done, Phase::Dropped, Phase::Stale] {
        let n = tally.count(state);
        if n > 0 {
            if !counts.is_empty() {
                counts.push(paint(DIM, " · "));
            }
            counts.push(paint(look(state).text, look(state).mark));
            counts.push(paint(DIM, &format!(" {n}")));
        }
    }
    counts
}
