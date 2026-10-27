//! The daily activity card of the stats board: each day's resolved steps by outcome, and the source of
//! its successful steps.

use super::measures::Cell;
use super::moment::Moment;
use super::month::{Month, DAYS};
use super::outcomes::Outcomes;
use super::panel::Panel;
use super::sheet::{Sheet, SPARK};
use crate::terminal::canvas::{paint, BOLD, DIM, GREEN, RED, SLATE, YELLOW};
use crate::terminal::grid::Grid;
use crate::terminal::unfold::{ease, mix};
use crate::text::clock::{day, weekday};
use ratatui::style::{Color, Style};

/// The rows of the chart.
const TALL: u16 = 8;

/// The panel of the daily activity.
pub(super) fn panel(m: &Moment) -> Panel {
    Panel {
        name: "Daily activity",
        says: format!("last {} days", Columns::of(m.grid(), top(m.month)).shown),
        start: 0.8,
        draw: daily,
    }
}

/// How a resolved step ended, each step one: its name and look.
pub(super) const OUTCOMES: [(&str, Style); 4] = [
    ("first pass", GREEN),
    ("reworked", YELLOW),
    ("dropped", RED),
    ("expired", DIM),
];

/// Where a successful step's approach came from: its name and look.
const SOURCES: [(&str, Style); 2] = [("reused", GREEN), ("new", SLATE)];

/// The counts of `day` by outcome: first pass, reworked, dropped, expired.
pub(super) fn outcomes(day: Outcomes) -> [usize; 4] {
    [day.first_pass(), day.reworked, day.dropped, day.expired]
}

/// The most steps one day of `month` resolved, at least one: the top of the chart.
fn top(month: &Month) -> usize {
    let days = month.days.iter().map(|d| d.resolved());
    days.max().unwrap_or(0).max(1)
}

/// Where the days of the chart stand in a card across a grid: how many days it shows and the first of
/// them, one width for every column and one for every gap, the pair that fills the room most, the first
/// column of the first day, on a track text column past its counts, and the columns the days take.
struct Columns {
    shown: usize,
    first: usize,
    wide: u16,
    gap: u16,
    x0: u16,
    used: u16,
}

impl Columns {
    /// The columns of a card across `grid` whose busiest day resolved `top` steps.
    fn of(grid: Grid, top: usize) -> Columns {
        let (x, w) = grid.span(0, 12);
        let x0 = grid.after(x + 2 + top.to_string().len() as u16 + 2);
        let room = (x + w - 3).saturating_sub(x0);
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
            x0,
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
/// outcome, first pass at the bottom, growing, every column and every gap one width, from a track
/// column with the counts before it, the day picked marked, the dates, and the outcome legend under
/// them from the chart's column. Source of successful steps: its title with the picked day's reused and
/// new steps beside it, a square a day from slate to green by the share reused, and its legend under
/// the squares. The rows it takes.
fn daily(s: &mut Sheet, m: &Moment, x: u16, y: u16, w: u16) -> u16 {
    let month = m.month;
    let top = top(month);
    let grid = m.grid();
    let columns = Columns::of(grid, top);
    let right = x + w - 3;
    let picked = m.day.max(columns.first);
    let days: Vec<[usize; 4]> = month.days.iter().map(|d| outcomes(*d)).collect();
    let day_of = month.days[picked];
    let cell = |look: Style, text: String| -> Cell { (vec![paint(look, &text)], vec![]) };
    let mut outcome = vec![cell(DIM, format!("{} resolved", day_of.resolved()))];
    for ((name, look), v) in OUTCOMES.iter().zip(days[picked]) {
        if v > 0 {
            outcome.push(cell(look.patch(BOLD), format!("{v} {name}")));
        }
    }
    let source = match day_of.successful() {
        0 => vec![cell(DIM, "no successful steps".into())],
        done => vec![
            cell(DIM, format!("{done} successful")),
            cell(GREEN.patch(BOLD), format!("{} reused", day_of.reused)),
            cell(DIM, format!("{} new", done - day_of.reused)),
        ],
    };
    let titles = ["Outcome", "Source of successful steps"];
    let least = outcome
        .iter()
        .chain(&source)
        .map(|(lead, _)| {
            lead.iter()
                .map(|s| s.content.chars().count())
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0) as u16;
    let after = grid.after(x + 2 + titles[1].chars().count() as u16 + 2);
    let beside = after + least <= right;
    let titled = |s: &mut Sheet, row: u16, title: &str, cells: &[Cell]| -> u16 {
        s.put(x + 2, row, title, BOLD);
        match beside {
            true => s.measures(grid, (after, row, right), cells, least),
            false => 1 + s.measures(grid, (columns.x0, row + 1, right), cells, least),
        }
    };
    s.put(x + 2, y + 2, &weekday(month.start(picked)), DIM);
    let mut row = y + 4;
    row += titled(s, row, titles[0], &outcome) + 1;
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
                .map_or(GREEN, |(_, look)| *look);
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
    row += s.legend(grid, (x0, row, right), &OUTCOMES) + 1;
    row += titled(s, row, titles[1], &source) + 1;
    let (slate, green) = (colour(SLATE), colour(GREEN));
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
        s.put(at, row, mark, Style::new().fg(mix(slate, green, share)));
    }
    row += 2;
    row += s.legend(grid, (x0, row, right), &SOURCES);
    row - y + 2
}

/// The colour of `look`.
fn colour(look: Style) -> Color {
    look.fg.unwrap_or(Color::Reset)
}
