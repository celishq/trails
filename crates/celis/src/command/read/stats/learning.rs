//! The Learning section of the stats board: the reuse of past work and the patterns applied.

use super::kind::Kind;
use super::moment::{plural, Moment};
use super::panel::Panel;
use super::sheet::Sheet;
use crate::command::read::tree::wrap;
use crate::terminal::canvas::{cut, paint, BOLD, CYAN, DIM, MAGENTA};
use crate::terminal::unfold::ease;
use ratatui::style::Style;

/// The panels of Learning: reuse, then patterns.
pub(super) fn panels() -> [Panel; 2] {
    [
        Panel {
            name: "Reuse",
            says: "steps done with a pattern".into(),
            start: 0.3,
            draw: reuse,
        },
        Panel {
            name: "Patterns",
            says: "learned patterns applied".into(),
            start: 0.4,
            draw: patterns,
        },
    ]
}

/// The panel of the top patterns of the last 14 days, when a pattern has a hit.
pub(super) fn top(m: &Moment) -> Option<Panel> {
    (!m.month.top.is_empty()).then(|| Panel {
        name: "Top patterns",
        says: "most hits".into(),
        start: 0.5,
        draw: patterns_hit,
    })
}

/// The patterns with the most hits, most first: each its words whole, wrapped before its bar, a bar
/// from the text column of track nine when the card is wide, and its hits at the right of its first row,
/// a blank row between two when one wraps; the rows it takes.
fn patterns_hit(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let top = &m.month.top;
    let wide = w >= 70;
    let most = top.iter().map(|r| r.2).max().unwrap_or(1).max(1);
    let count = x + w - 6;
    let start = m.grid().edge(9) + 2;
    let to = if wide { start - 2 } else { count - 2 };
    let room = usize::from(to.saturating_sub(x + 2));
    let texts: Vec<Vec<String>> = top.iter().map(|(_, words, _)| wrap(words, room)).collect();
    let gap = u16::from(texts.iter().any(|t| t.len() > 1));
    let mut row = y + 2;
    for (n, ((_, _, hits), lines)) in top.iter().zip(texts).enumerate() {
        if n > 0 {
            row += gap;
        }
        for (k, line) in lines.iter().enumerate() {
            s.put(x + 2, row + k as u16, line, Style::new());
        }
        if wide {
            let grow = ease((m.t - 0.7 - 0.08 * n as f64) / 0.5);
            let share = *hits as f64 / most as f64 * grow;
            s.bar(start, row, share, count.saturating_sub(start + 2), MAGENTA);
        }
        s.put(count, row, &format!("{hits:>3}"), BOLD);
        row += lines.len().max(1) as u16;
    }
    row - y + 2
}

/// The full length of a week's reuse bar: the highest weekly reuse rate of the four weeks, rounded up
/// to ten, at least ten.
pub(super) fn most(m: &Moment) -> i64 {
    let weeks = m.month.weeks();
    let most = weeks.iter().map(|w| w.reuse_rate()).max().unwrap_or(0);
    (most.max(1) + 9) / 10 * 10
}

/// Reuse, in cyan: the reused steps with their change, the earlier trails they came from, and the reuse
/// rate of each of the last four weeks as a bar; the rows it takes. The rate of the 14 days stands on
/// its tile.
fn reuse(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let now = m.now();
    let mut row = y + 2;
    let lead = vec![
        paint(CYAN.patch(BOLD), &now.reused.to_string()),
        paint(DIM, &format!(" reused {}", plural(now.reused, "step"))),
    ];
    let before = m.before(|o| o.reused as i64);
    let reused = (Some(now.reused as i64), before);
    let cell = m.measure(lead, reused, Kind::Count, true);
    row += s.measures(m.grid(), (x + 2, row, x + w - 3), &[cell], 0) + 1;
    s.put(x + 2, row, "weekly reuse rate · last 4 weeks", DIM);
    let bar = m.bars(x, w);
    let value = x + w - 7;
    let most = most(m) as f64;
    for (n, week) in m.month.weeks().iter().enumerate() {
        let k = ease((m.t - 0.6 - n as f64 * 0.12) / 0.5);
        let lit = n == 3;
        let first = m.date(n * 7);
        let name = match (w >= 40, lit) {
            (true, true) => "this week".to_string(),
            (true, false) => format!("week of {first}"),
            (false, true) => "this wk".to_string(),
            (false, false) => first,
        };
        let r = row + 1 + n as u16;
        let look = if lit { BOLD } else { DIM };
        s.put(x + 2, r, &name, look);
        let share = week.reuse_rate() as f64 / most * k;
        s.bar(bar, r, share, value.saturating_sub(bar + 2).max(3), CYAN);
        s.put(value, r, &format!("{:>3}%", week.reuse_rate()), look);
    }
    row + 5 - y + 2
}

/// Patterns, in violet: the steps that applied a pattern of the successful ones with their change in
/// steps, the patterns kept with how many were applied and how many are new as one plain line, a blank
/// row, then the pattern hits in each project as bars; the rows it takes.
fn patterns(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let (now, month) = (m.now(), m.month);
    let mut row = y + 2;
    let applied = now.applied as i64;
    let lead = vec![
        paint(MAGENTA.patch(BOLD), &applied.to_string()),
        paint(
            DIM,
            &format!(" of {} steps applied a pattern", now.successful()),
        ),
    ];
    let before = m.before(|o| o.applied as i64);
    let cell = m.measure(lead, (Some(applied), before), Kind::Count, true);
    row += s.measures(m.grid(), (x + 2, row, x + w - 3), &[cell], 0);
    let kept = [
        paint(Style::new(), &month.patterns.to_string()),
        paint(
            DIM,
            &format!(" {} kept: ", plural(month.patterns, "pattern")),
        ),
        paint(Style::new(), &month.kept.to_string()),
        paint(DIM, " applied, "),
        paint(Style::new(), &month.learned.to_string()),
        paint(DIM, " new"),
    ];
    s.spans(x + 2, row, &kept);
    row += 2;
    s.put(x + 2, row, "hits by project", DIM);
    let most = month
        .applied
        .iter()
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(0)
        .max(1);
    let bar = m.bars(x, w);
    let value = x + w - 6;
    for (k, (name, v)) in month.applied.iter().enumerate() {
        let r = row + 1 + k as u16;
        s.put(x + 2, r, &format!("📁 {}", cut(name, 7)), Style::new());
        let grow = ease(m.t - 0.7 - k as f64 * 0.1);
        let share = *v as f64 / most as f64 * grow;
        s.bar(bar, r, share, value.saturating_sub(bar + 2).max(3), MAGENTA);
        s.put(value, r, &format!("{v:>3}"), Style::new());
    }
    row += 1 + month.applied.len() as u16;
    row - y + 2
}
