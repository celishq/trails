//! The outcome card: the steps closed by how they ended.

use super::super::table::{Row, Table};
use super::{nothing, Sheet};
use crate::progress::{Period, Tally};
use crate::terminal::canvas::Styled;
use crate::terminal::glyph::strip;
use crate::terminal::ink::Ink;
use crate::terminal::look::look;
use celis_trails::State;

/// The outcome: the steps closed in the last 14 days by how they ended, as a strip of shares, with the
/// count of revised done steps beside the done share.
pub(super) fn outcome(last: &Period, inside: usize, ink: Ink) -> Sheet {
    let closed = last.closed();
    let tally = Tally::of(&closed);
    let states = [State::Done, State::Dropped, State::Expired];
    let counts: Vec<usize> = states.iter().map(|&st| tally.count(st)).collect();
    let total: usize = counts.iter().sum();
    let face = match total {
        0 => vec![ink.dim(&nothing("closed")).into()],
        _ => {
            let strip = strip(&counts, inside.saturating_sub(4));
            let mut face = vec![Styled::from(strip), Styled::default()];
            for (&state, &n) in states.iter().zip(&counts) {
                let look = look(state);
                let share = n * 100 / total;
                let mut line = Styled::from(vec![
                    ink.paint(look.text, look.mark),
                    format!(" {:<10}{n:>4} {share:>5}%", state.text()).into(),
                ]);
                if state == State::Done && tally.revised > 0 {
                    line.push_span(format!(" · {} revised", tally.revised));
                }
                face.push(line);
            }
            face
        }
    };
    let rows = closed
        .iter()
        .map(|s| Row {
            id: Some(s.id().to_string()),
            cells: vec![
                format!("{} {}", look(s.state()).mark, s.state().text()),
                s.title().to_string(),
            ],
        })
        .collect();
    Sheet {
        title: "05 OUTCOME".into(),
        face,
        table: Table {
            head: vec!["State", "Step"],
            rows,
            wide: 1,
        },
    }
}
