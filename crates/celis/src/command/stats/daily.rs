//! The daily activity card of the stats board: each day's resolved steps by outcome, and the source of
//! its successful steps.

use super::moment::Moment;
use super::month::{Month, DAYS};
use super::outcomes::Outcomes;
use super::panel::Panel;
use super::sheet::{Group, Sheet, SPARK};
use crate::terminal::canvas::{paint, BOLD, CYAN, DIM, LINK, RED, SLATE, YELLOW};
use crate::terminal::unfold::{ease, mix};
use crate::text::clock::{day, weekday};
use ratatui::style::{Color, Style};

/// The rows of the chart.
const TALL: u16 = 8;

/// The panel of the daily activity.
pub(super) fn panel(m: &Moment) -> Panel {
    Panel {
        name: "Daily activity",
        says: format!("last {} days", Columns::of(m.width, top(m.month)).shown),
        start: 0.8,
        draw: daily,
    }
}

/// How a resolved step ended, each step one: its name and look.
pub(super) const OUTCOMES: [(&str, Style); 4] = [
    ("first pass", LINK),
    ("reworked", YELLOW),
    ("dropped", RED),
    ("expired", DIM),
];

/// Where a successful step's approach came from: its name and look.
const SOURCES: [(&str, Style); 2] = [("reused", CYAN), ("new", SLATE)];

/// The counts of `day` by outcome: first pass, reworked, dropped, expired.
pub(super) fn outcomes(day: Outcomes) -> [usize; 4] {
    [day.first_pass(), day.reworked, day.dropped, day.expired]
}

/// The most steps one day of `month` resolved, at least one: the top of the chart.
fn top(month: &Month) -> usize {
    let days = month.days.iter().map(|d| d.resolved());
    days.max().unwrap_or(0).max(1)
}

/// Where the days of the chart stand on a board `width` wide: how many days it shows and the first of
/// them, one width for every column and one for every gap, the pair that fills the room most, the first
/// column of the first day, the whole chart with its counts centred, and the columns the days take.
struct Columns {
    shown: usize,
    first: usize,
    wide: u16,
    gap: u16,
    x0: u16,
    used: u16,
}

impl Columns {
    /// The columns of a board `width` wide whose busiest day resolved `top` steps.
    fn of(width: u16, top: usize) -> Columns {
        let labels = top.to_string().len() as u16 + 2;
        let room = width.saturating_sub(8 + labels);
        let shown = DAYS.min(usize::from(room).div_ceil(2)).max(1);
        let n = shown as u16;
        let fits = (1..8u16)
            .flat_map(|wide| [1u16, 2].map(|gap| (wide, gap)))
            .map(|(wide, gap)| (n * wide + (n - 1) * gap, wide, gap))
            .filter(|(used, _, _)| *used <= room);
        let (used, wide, gap) = fits
            .max_by_key(|(used, wide, gap)| (*used, std::cmp::Reverse(*gap), *wide))
            .unwrap_or((n, 1, 0));
        Columns {
            shown,
            first: DAYS - shown,
            wide,
            gap,
            x0: 4 + (room - used) / 2 + labels,
            used,
        }
    }

    /// The first column of the day `day`.
    fn column(&self, day: usize) -> u16 {
        self.x0 + (day - self.first) as u16 * (self.wide + self.gap)
    }
}

/// Daily activity: the picked day's date once at the top, then two small tables. Outcome: its title with
/// the picked day's resolved steps by outcome beside it, each day as a column of its resolved steps by
/// outcome, first pass at the bottom, growing, every column and every gap one width, the whole chart
/// centred, the day picked marked, the dates, and the outcome legend under them. Source of successful
/// steps: its title with the picked day's reused and new steps beside it, a square a day from slate to
/// cyan by the share reused, and its legend under the squares. The rows it takes.
fn daily(s: &mut Sheet, m: &Moment, _: u16, y: u16, w: u16) -> u16 {
    let month = m.month;
    let top = top(month);
    let columns = Columns::of(m.width, top);
    let picked = m.day.max(columns.first);
    let days: Vec<[usize; 4]> = month.days.iter().map(|d| outcomes(*d)).collect();
    let day_of = month.days[picked];
    let dot = || paint(DIM, "  ·  ");
    let mut outcome: Vec<Group> = vec![
        vec![paint(BOLD, "Outcome")],
        vec![
            dot(),
            paint(DIM, &format!("{} resolved", day_of.resolved())),
        ],
    ];
    for ((name, look), v) in OUTCOMES.iter().zip(days[picked]) {
        if v > 0 {
            outcome.push(vec![dot(), paint(look.patch(BOLD), &format!("{v} {name}"))]);
        }
    }
    let mut source: Vec<Group> = vec![vec![paint(BOLD, "Source of successful steps")]];
    match day_of.successful() {
        0 => source.push(vec![dot(), paint(DIM, "no successful steps")]),
        done => source.extend([
            vec![dot(), paint(DIM, &format!("{done} successful"))],
            vec![
                dot(),
                paint(CYAN.patch(BOLD), &format!("{} reused", day_of.reused)),
            ],
            vec![dot(), paint(DIM, &format!("{} new", done - day_of.reused))],
        ]),
    }
    let legend = |kinds: &[(&str, Style)]| -> Vec<Group> {
        let key = |(name, look): &(&str, Style)| {
            vec![paint(*look, "■"), paint(DIM, &format!(" {name}   "))]
        };
        kinds.iter().map(key).collect()
    };
    s.put(4, y + 2, &weekday(month.start(picked)), DIM);
    let mut row = y + 4;
    row += s.flow(4, row, w - 4, &outcome) + 1;
    let x0 = columns.x0;
    let marks = [
        (top.to_string(), 0),
        ((top / 2).to_string(), TALL / 2),
        ("0".to_string(), TALL - 1),
    ];
    for (label, r) in marks {
        let x = (x0 - 2).saturating_sub(label.len() as u16);
        s.put(x, row + r, &label, DIM);
    }
    let eighths = f64::from(TALL) * 8.0;
    for (d, counts) in days.iter().enumerate().skip(columns.first) {
        let grow = ease((m.t - 1.0 - (d - columns.first) as f64 * 0.03) / 0.5);
        let total: usize = counts.iter().sum();
        let filled = (total as f64 / top as f64 * eighths * grow).round_ties_even() as usize;
        let mut edges = vec![];
        let mut at = 0.0;
        for (n, v) in counts.iter().enumerate() {
            at += *v as f64 / top as f64 * eighths * grow;
            edges.push((at, OUTCOMES[n].1));
        }
        let x = columns.column(d);
        for r in 0..usize::from(TALL) {
            let low = r * 8;
            if low >= filled {
                break;
            }
            let fill = (filled - low).min(8);
            let middle = low as f64 + fill as f64 / 2.0;
            let look = edges
                .iter()
                .find(|(edge, _)| middle <= *edge)
                .or(edges.last())
                .map_or(LINK, |(_, look)| *look);
            let glyph = SPARK[fill].repeat(usize::from(columns.wide));
            s.put(x, row + TALL - 1 - r as u16, &glyph, look);
        }
    }
    let axis = row + TALL;
    let line = format!("└{}", "─".repeat(usize::from(columns.used)));
    s.put(x0 - 1, axis, &line, DIM);
    let wide = usize::from(columns.wide);
    s.put(columns.column(picked), axis + 1, &"▔".repeat(wide), BOLD);
    let mut dates: Vec<usize> = [0, 7, 14, 21]
        .into_iter()
        .filter(|d| *d >= columns.first)
        .collect();
    if columns.wide + columns.gap < 3 {
        dates.truncate(1);
    }
    for d in dates {
        s.put(columns.column(d), axis + 2, &day(month.start(d)), DIM);
    }
    let last = day(month.start(DAYS - 1));
    let end = columns.column(DAYS - 1) + columns.wide;
    s.put(end.saturating_sub(last.len() as u16), axis + 2, &last, DIM);
    row = axis + 4;
    row += s.flow(4, row, w - 4, &legend(&OUTCOMES)) + 1;
    row += s.flow(4, row, w - 4, &source) + 1;
    let (slate, cyan) = (colour(SLATE), colour(CYAN));
    for d in columns.first..DAYS {
        let x = columns.column(d);
        let (mark, at) = match wide >= 2 {
            true => ("██", x + (columns.wide - 2) / 2),
            false => ("■", x),
        };
        let day = month.days[d];
        if day.successful() == 0 {
            s.put(at, row, if wide >= 2 { " ·" } else { "·" }, DIM);
            continue;
        }
        let share = day.reused as f64 / day.successful() as f64;
        s.put(at, row, mark, Style::new().fg(mix(slate, cyan, share)));
    }
    row += 2;
    row += s.flow(4, row, w - 4, &legend(&SOURCES));
    row - y + 2
}

/// The colour of `look`.
fn colour(look: Style) -> Color {
    look.fg.unwrap_or(Color::Reset)
}
