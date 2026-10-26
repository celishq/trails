//! The top of the stats board: its head, the heading of a group, and the four headline tiles.

use super::kind::Kind;
use super::moment::{minutes, Moment};
use super::month::HALF;
use super::sheet::Sheet;
use crate::terminal::canvas::{paint, BOLD, CYAN, DIM, FRAME, MAGENTA};
use crate::terminal::unfold::{ease, mix, toward};
use crate::text::clock::day;
use ratatui::style::{Color, Style};

/// The rows a headline tile takes.
const TILE: u16 = 10;

/// A headline tile: its name, its value and the prior one, whether more is better, what it counts,
/// and the base it is counted from.
type Tile = (&'static str, Option<i64>, Option<i64>, bool, Kind, String);

/// The head on row 0: `CELIS STATS` and the dates of the last 14 days, the trails active now at the
/// right after a dot that pulses while one is.
pub(super) fn head(s: &mut Sheet, m: &Moment) {
    s.clip = None;
    s.alpha = 1.0;
    let w = m.width;
    s.put(2, 0, "CELIS STATS", BOLD);
    let right = format!("{} active", m.month.active);
    let right_w = right.chars().count() as u16;
    let cyan = CYAN.fg.unwrap_or(Color::Reset);
    let dot = match m.month.active {
        0 => DIM,
        _ => {
            let pulse = 0.5 + 0.5 * (m.t * 4.0).sin();
            Style::new().fg(mix(toward(cyan, 0.7), cyan, pulse))
        }
    };
    s.put(w.saturating_sub(4 + right_w), 0, "●", dot);
    s.put(w.saturating_sub(2 + right_w), 0, &right, DIM);
}

/// The Total block from row `y`: `Total` and the day the records start, then each sum of all time as its
/// name over its bold number, counting up, each on two, four or six tracks of the grid with a thin line
/// in the gutter before it, as many a row as hold 15 columns each, or a list of numbers beside their
/// names when fewer than two do; the row after it.
pub(super) fn total(s: &mut Sheet, m: &Moment, y: u16) -> u16 {
    s.clip = None;
    s.alpha = m.fade(0.04);
    let total = m.total;
    let k = ease((m.t - 0.1) / 0.9);
    let up = |v: u64| (v as f64 * k).round_ties_even() as u64;
    let patterns = total.patterns;
    let items: [(String, &str, Style); 6] = [
        (
            grouped(up(total.resolved as u64) as usize),
            "resolved steps",
            Style::new(),
        ),
        (
            grouped(up(total.reused as u64) as usize),
            "reused steps",
            CYAN,
        ),
        (
            grouped(up(total.hits as u64) as usize),
            "pattern hits",
            MAGENTA,
        ),
        (
            format!("{} of {patterns}", up(total.used as u64)),
            "patterns used",
            MAGENTA,
        ),
        (
            grouped(up(total.trails as u64) as usize),
            "trails",
            Style::new(),
        ),
        (minutes(up(total.time)), "active time", Style::new()),
    ];
    let x = s.put(2, y, "Total", BOLD);
    s.put(x, y, &format!("  since {}", day(total.since)), DIM);
    let grid = m.grid();
    let per = [6u16, 3, 2]
        .into_iter()
        .find(|per| grid.span(0, 12 / per).1 >= 15)
        .unwrap_or(1);
    let row = y + 2;
    if per < 2 {
        let full = [
            grouped(total.resolved),
            grouped(total.reused),
            grouped(total.hits),
            format!("{} of {patterns}", total.used),
            grouped(total.trails),
            minutes(total.time),
        ];
        let wide = full.iter().map(|t| t.chars().count()).max().unwrap_or(0) as u16;
        for (n, (value, name, look)) in items.iter().enumerate() {
            let r = row + n as u16;
            let at = (2 + wide).saturating_sub(value.chars().count() as u16);
            s.put(at, r, value, look.patch(BOLD));
            s.put(4 + wide, r, name, DIM);
        }
        s.alpha = 1.0;
        return row + items.len() as u16 + 1;
    }
    for (n, (value, name, look)) in items.iter().enumerate() {
        let col = n as u16 % per;
        let x = grid.edge(col * (12 / per));
        let r = row + (n as u16 / per) * 3;
        if col > 0 {
            s.put(x - 2, r, "│", FRAME);
            s.put(x - 2, r + 1, "│", FRAME);
        }
        s.put(x, r, name, DIM);
        s.put(x, r + 1, value, look.patch(BOLD));
    }
    s.alpha = 1.0;
    row + (items.len() as u16).div_ceil(per) * 3
}

/// `n` with a comma between each three digits: `1,240`.
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The heading of a group on row `y`: its name, what period it covers, then a line to the right; the
/// row after it and a blank.
pub(super) fn group(s: &mut Sheet, m: &Moment, y: u16, name: &str, says: &str) -> u16 {
    s.clip = None;
    s.alpha = 1.0;
    let mut x = s.put(2, y, name, BOLD);
    if !says.is_empty() && x + says.chars().count() as u16 + 6 < m.width {
        x = s.put(x + 2, y, says, DIM);
    }
    let line = "─".repeat(usize::from(m.width.saturating_sub(3 + x)));
    s.put(x + 1, y, &line, FRAME);
    y + 2
}

/// The four headline measures from row `y` as tiles of three, six or twelve tracks of the grid, four, two
/// or one a row as the tracks hold 24 columns: each named in its top border, a blank row, its value as big digits counting up beside a cyan bar, a
/// blank row, its change, the base it is counted from, and a blank row; the row after them.
pub(super) fn headline(s: &mut Sheet, m: &Moment, y: u16) -> u16 {
    let grid = m.grid();
    let per = [4u16, 2]
        .into_iter()
        .find(|per| grid.span(0, 12 / per).1 >= 24)
        .unwrap_or(1);
    let tracks = 12 / per;
    let (now, prior) = (m.now(), m.prior());
    let month = m.month;
    let k = ease((m.t - 0.1) / 0.9);
    let counted = |v: i64| (v as f64 * k).round_ties_even() as i64;
    let tiles: [Tile; 4] = [
        (
            "Reuse rate",
            Some(now.reuse_rate()),
            m.before(|o| o.reuse_rate()),
            true,
            Kind::Share,
            format!("{} of {} successful", now.reused, now.successful()),
        ),
        (
            "First-pass success",
            Some(now.first_pass_rate()),
            m.before(|o| o.first_pass_rate()),
            true,
            Kind::Share,
            format!("{} of {} resolved", now.first_pass(), now.resolved()),
        ),
        (
            "Resolved steps",
            Some(now.resolved() as i64),
            m.before(|o| o.resolved() as i64),
            true,
            Kind::Count,
            format!("{:.1} a day", now.resolved() as f64 / HALF as f64),
        ),
        (
            "Median step time",
            month.median.map(|t| t as i64),
            month
                .prior_median
                .filter(|_| prior.resolved() > 0)
                .map(|t| t as i64),
            false,
            Kind::Time,
            format!("of {} successful", now.successful()),
        ),
    ];
    for (n, (name, value, before, up, kind, base)) in tiles.into_iter().enumerate() {
        let n = n as u16;
        let (x, w) = grid.span((n % per) * tracks, (n % per + 1) * tracks);
        let top = y + (n / per) * (TILE + 1);
        s.alpha = ease((m.t - 0.08 * f64::from(n)) / 0.45);
        s.card((x, top, w, TILE), name, "");
        for r in 0..3 {
            s.put(x + 2, top + 2 + r, "▌", CYAN);
        }
        let text = match (value, kind) {
            (None, _) => "--".to_string(),
            (Some(v), Kind::Time) => {
                let shown = counted(v);
                format!("{}:{:02}", shown / 60, shown % 60)
            }
            (Some(v), Kind::Share) => format!("{}%", counted(v)),
            (Some(v), _) => counted(v).to_string(),
        };
        s.big(x + 4, top + 2, &text, Style::new());
        let mut change = kind.change(value, before, up);
        if let Some(head) = change.first_mut().and_then(|g| g.first_mut()) {
            *head = paint(head.style, head.content.trim_start());
        }
        let used = s.flow(x + 2, top + 6, w.saturating_sub(4), &change);
        s.put(x + 2, top + 6 + used, &base, DIM);
    }
    s.clip = None;
    let rows = 4u16.div_ceil(per);
    y + rows * (TILE + 1)
}
