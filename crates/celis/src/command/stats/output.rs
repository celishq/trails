//! The Output and speed section of the stats board: the steps resolved and the time a step took.

use super::moment::Moment;
use super::month::{DAYS, SPANS};
use super::panel::Panel;
use super::sheet::Sheet;
use crate::terminal::canvas::{CYAN, DIM};
use crate::terminal::unfold::ease;
use ratatui::style::Style;

/// The panels of Output and speed: resolved steps, then step time.
pub(super) fn panels() -> [Panel; 2] {
    [
        Panel {
            name: "Resolved steps",
            says: "by week".into(),
            start: 0.6,
            draw: resolved,
        },
        Panel {
            name: "Step time",
            says: "successful steps, start to done".into(),
            start: 0.7,
            draw: completion,
        },
    ]
}

/// Resolved steps by week: each of the four weeks as a bar in one colour with its label and count in the
/// text's colour; the rows it takes. The count of the 14 days stands on its tile.
fn resolved(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let row = y + 2;
    let weeks = m.month.weeks();
    let most = weeks.iter().map(|w| w.resolved()).max().unwrap_or(0).max(1);
    let bar = m.bars(x, w);
    let value = x + w - 6;
    for (k, week) in weeks.iter().enumerate() {
        let name = match w >= 44 {
            true => format!("{} – {}", m.date(7 * k), m.date((6 + 7 * k).min(DAYS - 1))),
            false => m.date(7 * k),
        };
        let r = row + k as u16;
        let look = Style::new();
        s.put(x + 2, r, &name, look);
        let share = week.resolved() as f64 / most as f64 * ease(m.t - 0.8 - k as f64 * 0.08);
        s.bar(bar, r, share, value.saturating_sub(bar + 2).max(3), CYAN);
        s.put(value, r, &format!("{:>3}", week.resolved()), look);
    }
    row + 4 - y + 2
}

/// Step time: how many successful steps took each span of time from start to done; the rows it takes.
/// The median stands on its tile.
fn completion(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let month = m.month;
    let row = y + 2;
    let most = month.spans.iter().copied().max().unwrap_or(0).max(1);
    let bar = m.bars(x, w);
    let value = x + w - 6;
    for (k, ((name, _), v)) in SPANS.iter().zip(month.spans).enumerate() {
        let r = row + k as u16;
        let look = if v > 0 { Style::new() } else { DIM };
        s.put(x + 2, r, name, look);
        let share = v as f64 / most as f64 * ease(m.t - 0.9 - k as f64 * 0.05);
        s.bar(bar, r, share, value.saturating_sub(bar + 2).max(3), CYAN);
        s.put(value, r, &format!("{v:>3}"), look);
    }
    row + SPANS.len() as u16 - y + 2
}
