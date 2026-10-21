//! The Quality section of the stats board: how the resolved steps ended.

use super::kind::Kind;
use super::moment::Moment;
use super::panel::Panel;
use super::sheet::{Group, Sheet};
use crate::terminal::canvas::{paint, BOLD, CYAN, DIM, GREEN, RED, YELLOW};
use crate::terminal::unfold::ease;

/// The panel of Quality: how the resolved steps of the last 14 days ended.
pub(super) fn panel() -> Panel {
    Panel {
        name: "Resolved steps",
        says: "how they ended".into(),
        start: 0.5,
        draw: resolution,
    }
}

/// How the resolved steps ended: one bar split by outcome, growing; the status model that adds them up;
/// and the rework rate with its change, each wrapped to the width; the rows it takes. First-pass success
/// stands on its tile.
fn resolution(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let now = m.now();
    let model: Vec<Group> = [
        paint(BOLD, &format!("{} resolved", now.resolved())),
        paint(DIM, " = "),
        paint(
            GREEN.patch(BOLD),
            &format!("{} successful", now.successful()),
        ),
        paint(
            DIM,
            &format!(
                " ({} first pass + {} reworked)",
                now.first_pass(),
                now.reworked
            ),
        ),
        paint(DIM, " + "),
        paint(RED.patch(BOLD), &format!("{} dropped", now.dropped)),
        paint(DIM, " + "),
        paint(DIM.patch(BOLD), &format!("{} expired", now.expired)),
    ]
    .into_iter()
    .map(|s| vec![s])
    .collect();
    let k = ease((m.t - 0.7) / 0.6);
    let parts = [
        (now.first_pass() as f64 * k, CYAN),
        (now.reworked as f64 * k, YELLOW),
        (now.dropped as f64 * k, RED),
        (now.expired as f64 * k, DIM),
    ];
    s.stack(x + 2, y + 2, &parts, w - 4);
    let mut row = y + 3;
    row += s.flow(x + 2, row, w - 4, &model);
    let rework = vec![
        paint(YELLOW.patch(BOLD), &format!("{}%", now.rework_rate())),
        paint(DIM, " reworked"),
    ];
    let values = (Some(now.rework_rate()), m.before(|o| o.rework_rate()));
    row += s.flow(
        x + 2,
        row,
        w - 4,
        &m.measure(rework, values, Kind::Share, false),
    );
    row - y + 2
}
