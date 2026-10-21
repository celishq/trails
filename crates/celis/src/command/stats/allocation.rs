//! The Resources section of the stats board: where the active time of the 14 days went.

use super::moment::{minutes, Moment};
use super::outcomes::percent;
use super::panel::Panel;
use super::sheet::{Group, Sheet};
use crate::terminal::canvas::{cut, paint, BOLD, CYAN, DIM, RED, YELLOW};
use crate::terminal::unfold::ease;
use ratatui::style::Style;

/// The panel of Resources: the allocation of the active time.
pub(super) fn panel() -> Panel {
    Panel {
        name: "Allocation",
        says: "active time".into(),
        start: 1.0,
        draw: allocation,
    }
}

/// The groups that name a project's `rework` and `dropped` seconds, each kept whole.
fn losses(rework: u64, dropped: u64) -> Vec<Group> {
    let mut groups = vec![];
    if rework > 0 {
        groups.push(vec![
            paint(YELLOW.patch(BOLD), &minutes(rework)),
            paint(DIM, " rework"),
        ]);
    }
    if dropped > 0 {
        let lead = if rework > 0 { "  ·  " } else { "" };
        groups.push(vec![
            paint(DIM, lead),
            paint(RED.patch(BOLD), &minutes(dropped)),
            paint(DIM, " dropped"),
        ]);
    }
    groups
}

/// Allocation: the active time with its change, the time lost to rework and to dropped work with their
/// shares; then each project's time as one bar of productive, rework and dropped time, growing, with
/// its time and share, and under a project that lost time its rework and dropped minutes; the rows it
/// takes.
fn allocation(s: &mut Sheet, m: &Moment, _: u16, y: u16, w: u16) -> u16 {
    let month = m.month;
    let total = month.time();
    let rework: u64 = month.projects.iter().map(|p| p.rework).sum();
    let dropped: u64 = month.projects.iter().map(|p| p.dropped).sum();
    let share = |part: u64| percent(part as usize, total as usize);
    let moved = total as i64 - month.prior_active as i64;
    let change = match moved {
        0 => "= no change".to_string(),
        d if d > 0 => format!("▲ {}", minutes(d as u64)),
        d => format!("▼ {}", minutes(d.unsigned_abs())),
    };
    let head: Vec<Group> = vec![
        vec![paint(BOLD, &minutes(total))],
        vec![paint(DIM, " active  ")],
        vec![paint(DIM.patch(BOLD), &change)],
        vec![
            paint(DIM, "  ·  "),
            paint(YELLOW.patch(BOLD), &minutes(rework)),
            paint(DIM, &format!(" rework {}%", share(rework))),
        ],
        vec![
            paint(DIM, "  ·  "),
            paint(RED.patch(BOLD), &minutes(dropped)),
            paint(DIM, &format!(" dropped {}%", share(dropped))),
        ],
    ];
    let legend: Vec<Group> = [
        ("productive   ", CYAN),
        ("rework   ", YELLOW),
        ("dropped", RED),
    ]
    .into_iter()
    .map(|(name, look)| vec![paint(look, "■"), paint(DIM, &format!(" {name}"))])
    .collect();
    let tail = 16;
    let top = s.flow(4, y + 2, w - 4, &head);
    let mut row = y + 2 + top + 1;
    row += s.flow(4, row, w - 4, &legend);
    let most = month
        .projects
        .iter()
        .map(|p| p.active)
        .max()
        .unwrap_or(0)
        .max(1);
    let span = w.saturating_sub(14 + tail).max(3);
    for (k, project) in month.projects.iter().enumerate() {
        s.put(4, row, &format!("📁 {}", cut(&project.name, 9)), BOLD);
        let grow = ease(m.t - 1.2 - 0.1 * k as f64);
        let cells = ((project.active as f64 / most as f64 * f64::from(span) * grow)
            .round_ties_even() as u16)
            .max(1);
        let parts: Vec<(f64, Style)> = [
            (project.productive(), CYAN),
            (project.rework, YELLOW),
            (project.dropped, RED),
        ]
        .into_iter()
        .map(|(v, look)| (v as f64 / 60.0 + 0.0001, look))
        .collect();
        s.stack(16, row, &parts, cells);
        let text = format!("{:>7}  {}%", minutes(project.active), share(project.active));
        s.put(w + 3 - tail, row, &text, Style::new());
        row += 1;
        if project.rework > 0 || project.dropped > 0 {
            row += s.flow(16, row, w - 16, &losses(project.rework, project.dropped));
        }
    }
    row - y + 2
}
