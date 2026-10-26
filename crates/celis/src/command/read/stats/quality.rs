//! The Quality section of the stats board: how the resolved steps ended.

use super::kind::Kind;
use super::measures::Cell;
use super::moment::Moment;
use super::outcomes::Outcomes;
use super::panel::Panel;
use super::sheet::{Group, Sheet};
use crate::terminal::canvas::{paint, BOLD, DIM, GREEN, RED, YELLOW};
use crate::terminal::unfold::ease;
use ratatui::style::Style;

/// The panel of Quality: how the resolved steps of the last 14 days ended.
pub(super) fn panel() -> Panel {
    Panel {
        name: "Resolved steps",
        says: "how they ended".into(),
        start: 0.5,
        draw: resolution,
    }
}

/// An outcome of a resolved step: its count in a span, its name, its look, and whether more is better,
/// none when its change says nothing.
type Outcome = (fn(Outcomes) -> usize, &'static str, Style, Option<bool>);

/// How the resolved steps ended: one bar split by outcome, growing; under a blank row, a table of the
/// resolved count in plain white with its change in steps, then each outcome in the bar's order and
/// colour with the change of its count in steps, green when better and none when the same, every
/// change in one column; the rows it takes.
fn resolution(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let now = m.now();
    let outcomes: [Outcome; 4] = [
        (Outcomes::first_pass, "first pass", GREEN, Some(true)),
        (|o| o.reworked, "reworked", YELLOW, Some(false)),
        (|o| o.dropped, "dropped", RED, Some(false)),
        (|o| o.expired, "expired", DIM, Some(false)),
    ];
    let prior = m.before(|o| o.resolved() as i64);
    let changed = |kind: Kind, now: i64, value: &dyn Fn(Outcomes) -> i64, up: bool| -> Group {
        let before = m.before(value).filter(|before| *before != now);
        let change = before.map(|before| kind.change(Some(now), Some(before), up));
        change.into_iter().flatten().flatten().collect()
    };
    let resolved = now.resolved() as i64;
    let first = vec![paint(Style::new(), &format!("{resolved} resolved"))];
    let change = changed(Kind::Count, resolved, &|o| o.resolved() as i64, true);
    let mut cells: Vec<Cell> = vec![(first, change)];
    for (count, name, look, up) in outcomes {
        let steps = move |o: Outcomes| count(o) as i64;
        let lead = vec![
            paint(look.patch(BOLD), &count(now).to_string()),
            paint(DIM, &format!(" {name}")),
        ];
        let change = up.map_or(vec![], |up| changed(Kind::Count, steps(now), &steps, up));
        cells.push((lead, change));
    }
    if prior.is_none() {
        cells.push((vec![paint(DIM, "no prior steps")], vec![]));
    }
    let k = ease((m.t - 0.7) / 0.6);
    let parts = [
        (now.first_pass() as f64 * k, GREEN),
        (now.reworked as f64 * k, YELLOW),
        (now.dropped as f64 * k, RED),
        (now.expired as f64 * k, DIM),
    ];
    s.stack(x + 2, y + 2, &parts, w - 4);
    let mut row = y + 4;
    row += s.measures(m.grid(), (x + 2, row, x + w - 3), &cells, 0);
    row - y + 2
}
