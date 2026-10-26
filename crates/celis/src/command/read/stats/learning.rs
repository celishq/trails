//! The Learning section of the stats board: the reuse of past work and the patterns applied.

use super::kind::Kind;
use super::moment::{counted, plural, Moment};
use super::panel::Panel;
use super::sheet::{Group, Sheet};
use crate::terminal::canvas::{cut, paint, BOLD, CYAN, DIM, MAGENTA};
use crate::terminal::unfold::ease;
use celis_store::records::id::short;
use ratatui::style::Style;

/// The panels of Learning: reuse, then patterns.
pub(super) fn panels() -> [Panel; 2] {
    [
        Panel {
            name: "Reuse",
            says: "past steps reapplied".into(),
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

/// The patterns with the most hits, most first: each its place, its id, its words cut to the width, a bar
/// when the card is wide, and its hits, the words on a row of their own when the card is narrow; the
/// rows it takes.
fn patterns_hit(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let top = &m.month.top;
    let (wide, narrow) = (w >= 70, w < 60);
    let per = if narrow { 2 } else { 1 };
    let most = top.iter().map(|r| r.2).max().unwrap_or(1).max(1);
    let count = x + w - 6;
    let bar = if wide { 16 } else { 0 };
    let (from, to) = match narrow {
        true => (x + 5, x + w - 2),
        false => (x + 15, count - 2 - if wide { bar + 2 } else { 0 }),
    };
    for (n, (id, words, hits)) in top.iter().enumerate() {
        let row = y + 2 + n as u16 * per;
        s.put(x + 2, row, &(n + 1).to_string(), DIM);
        s.put(x + 5, row, &short(id), DIM);
        let room = usize::from(to.saturating_sub(from));
        let shown = cut(words, room);
        let shown = match shown.strip_suffix('…') {
            Some(kept) => format!("{}…", kept.trim_end()),
            None => shown,
        };
        s.put(from, row + per - 1, &shown, Style::new());
        if wide {
            let grow = ease((m.t - 0.7 - 0.08 * n as f64) / 0.5);
            let share = *hits as f64 / most as f64 * grow;
            s.bar(to + 2, row, share, bar, MAGENTA);
        }
        s.put(count, row, &format!("{hits:>3}"), BOLD);
    }
    2 + per * top.len() as u16 + 2
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
    row += s.flow(
        x + 2,
        row,
        w - 4,
        &m.measure(lead, reused, Kind::Count, true),
    );
    s.put(
        x + 2,
        row,
        &format!("from {}", counted(m.month.sources, "earlier trail")),
        DIM,
    );
    row += 2;
    s.put(x + 2, row, "weekly reuse rate · last 4 weeks", DIM);
    let label = if w >= 40 { 15 } else { 10 };
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
        s.put(x + 4, r, &name, look);
        let share = week.reuse_rate() as f64 / most * k;
        s.bar(
            x + 5 + label,
            r,
            share,
            w.saturating_sub(label + 14).max(3),
            CYAN,
        );
        s.put(x + w - 7, r, &format!("{:>3}%", week.reuse_rate()), look);
    }
    row + 5 - y + 2
}

/// Patterns, in violet: the pattern usage with its change and the steps it counts, the patterns kept with how
/// many were applied and how many are new, the pattern hits in each project, and the usage of each of the
/// last four weeks as a run of numbers; the rows it takes.
fn patterns(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let (now, month) = (m.now(), m.month);
    let violet = MAGENTA.patch(BOLD);
    let mut row = y + 2;
    let usage = now.pattern_usage();
    let lead = vec![
        paint(violet, &format!("{usage}%")),
        paint(DIM, " pattern usage"),
    ];
    let before = m.before(|o| o.pattern_usage());
    row += s.flow(
        x + 2,
        row,
        w - 4,
        &m.measure(lead, (Some(usage), before), Kind::Share, true),
    );
    let applied = format!(
        "{} of {} successful steps applied a pattern",
        now.applied,
        now.successful()
    );
    s.put(x + 2, row, &applied, DIM);
    row += 1;
    let counts: Vec<Group> = vec![
        vec![
            paint(violet, &format!("§ {}", month.patterns)),
            paint(DIM, &format!(" {} kept", plural(month.patterns, "pattern"))),
        ],
        vec![
            paint(DIM, "  ·  "),
            paint(BOLD, &month.kept.to_string()),
            paint(DIM, " applied"),
        ],
        vec![
            paint(DIM, "  ·  "),
            paint(violet, &format!("+{}", month.learned)),
            paint(DIM, " new"),
        ],
    ];
    row += s.flow(x + 2, row, w - 4, &counts) + 1;
    s.put(x + 2, row, "pattern hits by project", DIM);
    let most = month
        .applied
        .iter()
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(0)
        .max(1);
    for (k, (name, v)) in month.applied.iter().enumerate() {
        let r = row + 1 + k as u16;
        s.put(x + 4, r, &format!("📁 {}", cut(name, 7)), Style::new());
        let grow = ease(m.t - 0.7 - k as f64 * 0.1);
        let share = *v as f64 / most as f64 * grow;
        s.bar(x + 14, r, share, w.saturating_sub(24).max(3), MAGENTA);
        s.put(x + w - 6, r, &format!("{v:>3}"), Style::new());
    }
    row += 1 + month.applied.len() as u16;
    s.put(x + 2, row, "weekly pattern usage · last 4 weeks", DIM);
    row += 1;
    let mut end = x + 4;
    let alpha = s.alpha;
    for (n, week) in month.weeks().iter().enumerate() {
        s.alpha = alpha * ease((m.t - 0.7 - n as f64 * 0.15) / 0.4);
        if n > 0 {
            end = s.put(end, row, " → ", MAGENTA);
        }
        let look = if n == 3 { BOLD } else { DIM };
        end = s.put(end, row, &format!("{}%", week.pattern_usage()), look);
    }
    s.alpha = alpha;
    row - y + 3
}
