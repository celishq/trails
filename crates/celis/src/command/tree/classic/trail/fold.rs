//! The folded lines of a level: its rules line and its closed line, each open or closed.

use crate::command::tree::classic::level::ids;
use crate::command::tree::classic::row::Row;
use crate::command::tree::classic::{after, Item, RuleNote, Tree, CLOSED, RULES};
use crate::progress::Tally;
use crate::terminal::canvas::{paint, Styled};
use crate::terminal::classic::look;
use crate::terminal::classic::MAGENTA;
use crate::text::line::plural;
use celis_trails::{State, Step};
use ratatui::style::Style;
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds the rules line `§ <n> rules` of the trail `trail` after `rail`, the `last` of its level;
    /// open, a line under it for each of `rules`: `§ <text>`, wrapped, its id at the end.
    pub(super) fn rules_line(
        &self,
        rules: Vec<RuleNote>,
        rail: &Styled,
        last: bool,
        trail: &str,
        lines: &mut Vec<Row>,
    ) {
        let id = format!("{RULES}{trail}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, last, Some(open));
        let mark = paint(MAGENTA, "§");
        let first = lines.len();
        lines.push(Row::Titled {
            lead,
            marks: Styled::from(mark.clone()),
            title: plural(rules.len(), "rule"),
            tone: Style::new(),
            rest: after(&rail, Span::raw("  ")),
        });
        let item = Item {
            id: id.clone(),
            open,
            parent: Some(trail.to_string()),
            under: rules.iter().map(|(record, _)| record.id.clone()).collect(),
        };
        self.pick(item, lines, first);
        if !open {
            return;
        }
        let all = rules.len();
        for (n, (record, rule)) in rules.into_iter().enumerate() {
            let (lead, below) = self.branch(&rail, n + 1 == all, None);
            let first = lines.len();
            lines.push(Row::Titled {
                lead,
                marks: Styled::from(mark.clone()),
                title: rule.content.clone(),
                tone: Style::new(),
                rest: after(&below, Span::raw("  ")),
            });
            let item = Item {
                id: record.id.clone(),
                open: false,
                parent: Some(id.clone()),
                under: vec![],
            };
            self.pick(item, lines, first);
        }
    }

    /// Adds the closed line `<marks> <n> closed steps` of the ended `steps` of the line `above`, the last
    /// of its level after `rail`: `✓` when one is done, `✗` when one is dropped, `◌` when one is expired;
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
        let marks: Styled = [State::Done, State::Dropped, State::Expired]
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
