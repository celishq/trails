//! The Resources section of the stats board: where the active time of the 14 days went.

use super::measures::Cell;
use super::moment::{minutes, Moment};
use super::panel::Panel;
use super::project::Project;
use super::sheet::{Group, Sheet, LEGEND};
use crate::terminal::canvas::{cut, paint, visible, BOLD, DIM, GREEN, RED, YELLOW};
use crate::terminal::unfold::ease;
use ratatui::style::Style;

/// The change of the active time from the prior 14 days in whole minutes, as the time itself reads:
/// `▼ 38m`; `no prior steps` with none before.
pub(super) fn spent(m: &Moment) -> Group {
    let moved = m.month.time() as i64 - m.month.prior_active as i64;
    let text = match moved {
        _ if m.prior().resolved() == 0 => return vec![paint(DIM, "no prior steps")],
        0 => return vec![paint(DIM, "= no change")],
        d if d > 0 => format!("▲ {}", minutes(d as u64)),
        d => format!("▼ {}", minutes(d.unsigned_abs())),
    };
    vec![paint(DIM.patch(BOLD), &text)]
}

/// The panel of Resources: the allocation of the active time.
pub(super) fn panel() -> Panel {
    Panel {
        name: "Allocation",
        says: "active time".into(),
        start: 1.0,
        draw: allocation,
    }
}

/// Allocation: the active time with its change; then each project's time as one bar of productive,
/// rework and dropped time from a track column, growing, with its time at the right; under the bars the
/// legend of the three, each with its time, on the legend's columns; the rows it takes.
fn allocation(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let month = m.month;
    let total = month.time();
    let sum = |part: fn(&Project) -> u64| month.projects.iter().map(part).sum::<u64>();
    let head: Vec<Cell> = vec![(
        vec![paint(BOLD, &minutes(total)), paint(DIM, " active")],
        spent(m),
    )];
    let legend: Vec<Cell> = [
        ("productive", GREEN, sum(Project::productive)),
        ("rework", YELLOW, sum(|p| p.rework)),
        ("dropped", RED, sum(|p| p.dropped)),
    ]
    .into_iter()
    .map(|(name, look, time)| {
        let lead = vec![paint(look, "■"), paint(DIM, &format!(" {name}"))];
        (lead, vec![paint(Style::new(), &minutes(time))])
    })
    .collect();
    let tail = 8;
    let grid = m.grid();
    let bar = m.bars(x, w);
    let end = x + w - 3;
    let top = s.measures(grid, (x + 2, y + 2, end), &head, 0);
    let mut row = y + 2 + top + 1;
    let most = month
        .projects
        .iter()
        .map(|p| p.active)
        .max()
        .unwrap_or(0)
        .max(1);
    let span = end.saturating_sub(bar + tail + 1).max(3);
    for (k, project) in month.projects.iter().enumerate() {
        s.put(x + 2, row, &format!("📁 {}", cut(&project.name, 8)), BOLD);
        let grow = ease(m.t - 1.2 - 0.1 * k as f64);
        let cells = ((project.active as f64 / most as f64 * f64::from(span) * grow)
            .round_ties_even() as u16)
            .max(1);
        let parts: Vec<(f64, Style)> = [
            (project.productive(), GREEN),
            (project.rework, YELLOW),
            (project.dropped, RED),
        ]
        .into_iter()
        .map(|(v, look)| (v as f64 / 60.0 + 0.0001, look))
        .collect();
        s.stack(bar, row, &parts, cells);
        let text = minutes(project.active);
        s.put(end - visible(&text) as u16, row, &text, Style::new());
        row += 1;
    }
    row += 1;
    row += s.measures(grid, (bar, row, end), &legend, LEGEND);
    row - y + 2
}
