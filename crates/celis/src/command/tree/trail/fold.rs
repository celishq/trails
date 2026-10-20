//! The folded lines of a level: its rules line and its closed line, each open or closed.

use crate::command::tree::level::ids;
use crate::command::tree::row::Row;
use crate::command::tree::{Item, RuleNote, Tree, CLOSED, RULES};
use crate::progress::Tally;
use crate::terminal::canvas::{paint, Styled, DIM, MAGENTA};
use crate::terminal::look::look;
use crate::text::line::plural;
use celis_trails::records::id::short;
use celis_trails::{State, Step};
use ratatui::style::Style;
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds the rules line `§ <n> rules` of the trail `trail` after `rail`, the `last` of its level;
    /// open, a line under it for each of `rules`: `§ <text>`, cut to fit, its scope and id at the right.
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
        let (lead, rail) = self.branch(rail, last, Some(open), self.lane(trail));
        let mark = paint(MAGENTA, "§");
        let first = lines.len();
        lines.push(Row::Lined {
            lead,
            marks: Styled::from(mark.clone()),
            title: plural(rules.len(), "rule"),
            tone: Style::new(),
            meta: Styled::default(),
            whole: None,
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
            let (lead, _) = self.branch(&rail, n + 1 == all, None, MAGENTA);
            let first = lines.len();
            let meta = format!("{} · {}", rule.scope.short(), short(&record.id));
            lines.push(Row::Lined {
                lead,
                marks: Styled::from(mark.clone()),
                title: rule.content.clone(),
                tone: Style::new(),
                meta: Styled::from(paint(DIM, &meta)),
                whole: None,
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

    /// Adds the closed line `✓ <n> closed steps` of the ended `steps` of the line `above`, the last of its
    /// level after `rail`, their count by state at the right (`✓ 7 · ✗ 1 · ◌ 2`); open, the steps under it.
    pub(super) fn closed_line(
        &self,
        steps: Vec<Step>,
        rail: &Styled,
        above: &str,
        lines: &mut Vec<Row>,
    ) {
        let id = format!("{CLOSED}{above}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, true, Some(open), self.lane(above));
        let counts = counts(Tally::of(&steps));
        let first = lines.len();
        lines.push(Row::Lined {
            lead,
            marks: Styled::from(paint(look(State::Done).text, look(State::Done).mark)),
            title: plural(steps.len(), "closed step"),
            tone: Style::new(),
            meta: Styled::from(counts),
            whole: None,
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

/// The closed steps of `tally` counted by state, each count after its mark (`✓ 7 · ✗ 1 · ◌ 2`).
pub(super) fn counts(tally: Tally) -> Vec<Span<'static>> {
    let mut counts: Vec<Span> = vec![];
    for state in [State::Done, State::Dropped, State::Expired] {
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
