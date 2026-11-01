//! The folded lines of a level: its patterns line, its notes line and its closed line, each open or
//! closed.

use crate::command::trail::tree::classic::level::ids;
use crate::command::trail::tree::classic::row::Row;
use crate::command::trail::tree::classic::{after, Item, Tree, CLOSED};
use crate::command::trail::tree::details;
use crate::progress::{Phase, Tally};
use crate::terminal::canvas::{paint, Styled};
use crate::terminal::classic::look;
use crate::text::line::plural;
use celis_nostr::Note;
use celis_trails::Step;
use ratatui::style::Style;
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds the folded line `<mark> <n> <noun>s` of `records` under the row `trail` after `rail`, the
    /// `last` of its level; open, a line under it for each record: `<mark> <first line>`, wrapped, which
    /// opens to the `details` of its other lines.
    pub(super) fn records_line(
        &self,
        records: Vec<&Note>,
        (mark, noun): (Span<'static>, &str),
        rail: &Styled,
        last: bool,
        trail: &str,
        lines: &mut Vec<Row>,
    ) {
        let id = format!("{}{trail}", mark.content);
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, last, Some(open));
        let first = lines.len();
        lines.push(Row::Titled {
            lead,
            marks: Styled::from(mark.clone()),
            title: plural(records.len(), noun),
            tone: Style::new(),
            rest: after(&rail, Span::raw("  ")),
        });
        let item = Item {
            id: id.clone(),
            open,
            parent: Some(trail.to_string()),
            under: records.iter().map(|r| r.id.clone()).collect(),
        };
        self.pick(item, lines, first);
        if !open {
            return;
        }
        let all = records.len();
        for (n, record) in records.into_iter().enumerate() {
            let more = details(record.content.lines().skip(1));
            let toggle = (!more.is_empty()).then(|| self.open(&record.id));
            let (lead, below) = self.branch(&rail, n + 1 == all, toggle);
            let first = lines.len();
            lines.push(Row::Titled {
                lead,
                marks: Styled::from(mark.clone()),
                title: record
                    .content
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                tone: Style::new(),
                rest: after(&below, Span::raw("  ")),
            });
            let item = Item {
                id: record.id.clone(),
                open: toggle == Some(true),
                parent: Some(id.clone()),
                under: vec![],
            };
            self.pick(item, lines, first);
            if toggle == Some(true) {
                let rail = after(&below, Span::raw("  "));
                for (label, text) in more {
                    self.labelled(lines, &rail, &label, text);
                }
            }
        }
    }

    /// Adds the closed line `<marks> <n> closed steps` of the ended `steps` of the line `above`, the last
    /// of its level after `rail`: `✓` when one is done, `✗` when one is dropped, `◌` when one is stale;
    /// open, the steps under it.
    pub(super) fn closed_line(
        &self,
        steps: Vec<Step>,
        rail: &Styled,
        above: &str,
        lines: &mut Vec<Row>,
    ) {
        let id = format!("{CLOSED}{above}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, true, Some(open));
        let tally = Tally::of(&steps);
        let marks: Styled = [Phase::Done, Phase::Dropped, Phase::Stale]
            .into_iter()
            .filter(|state| tally.count(*state) > 0)
            .map(|state| paint(look(state).text, look(state).mark))
            .collect();
        let first = lines.len();
        lines.push(Row::Titled {
            lead,
            marks,
            title: plural(steps.len(), "closed step"),
            tone: Style::new(),
            rest: after(&rail, Span::raw("  ")),
        });
        let item = Item {
            id: id.clone(),
            open,
            parent: Some(above.to_string()),
            under: ids(&steps),
        };
        self.pick(item, lines, first);
        if open {
            self.steps(steps, &rail, &id, false, lines);
        }
    }
}
