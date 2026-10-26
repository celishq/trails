//! The Quality section of the stats board: how the resolved steps ended.

use super::kind::Kind;
use super::moment::Moment;
use super::outcomes::{percent, Outcomes};
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

/// How the resolved steps ended: one bar split by outcome, growing; under a blank row, the resolved
/// count in plain white with its change in steps, then each outcome in the bar's order and colour with
/// the change of its share in pp, green when better and none when the same, each wrapped to the width;
/// the rows it takes.
fn resolution(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let now = m.now();
    let outcomes: [Outcome; 4] = [
        (Outcomes::first_pass, "first pass", GREEN, Some(true)),
        (|o| o.reworked, "reworked", YELLOW, Some(false)),
        (|o| o.dropped, "dropped", RED, Some(false)),
        (|o| o.stale, "stale", DIM, Some(false)),
    ];
    let prior = m.before(|o| o.resolved() as i64);
    let changed = |kind: Kind, now: i64, value: &dyn Fn(Outcomes) -> i64, up: bool| {
        let before = m.before(value).filter(|before| *before != now);
        let change = before.map(|before| kind.change(Some(now), Some(before), up));
        let change: Vec<_> = change.into_iter().flatten().flatten().collect();
        let gap = (!change.is_empty()).then(|| paint(DIM, " "));
        gap.into_iter().chain(change)
    };
    let resolved = now.resolved() as i64;
    let mut first = vec![paint(Style::new(), &format!("{resolved} resolved"))];
    first.extend(changed(
        Kind::Count,
        resolved,
        &|o| o.resolved() as i64,
        true,
    ));
    let mut model: Vec<Group> = vec![first];
    for (count, name, look, up) in outcomes {
        let share = move |o: Outcomes| percent(count(o), o.resolved());
        let mut group = vec![
            paint(DIM, "  ·  "),
            paint(look.patch(BOLD), &count(now).to_string()),
            paint(DIM, &format!(" {name}")),
        ];
        if let Some(up) = up {
            group.extend(changed(Kind::Share, share(now), &share, up));
        }
        model.push(group);
    }
    if prior.is_none() {
        model.push(vec![paint(DIM, "  ·  no prior steps")]);
    }
    let k = ease((m.t - 0.7) / 0.6);
    let parts = [
        (now.first_pass() as f64 * k, GREEN),
        (now.reworked as f64 * k, YELLOW),
        (now.dropped as f64 * k, RED),
        (now.stale as f64 * k, DIM),
    ];
    s.stack(x + 2, y + 2, &parts, w - 4);
    let mut row = y + 4;
    row += s.flow(x + 2, row, w - 4, &model);
    row - y + 2
}
