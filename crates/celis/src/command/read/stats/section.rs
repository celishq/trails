//! A section of the stats board: closed to one card of its key measures, or opened to its details.

use super::daily::{outcomes, OUTCOMES};
use super::kind::Kind;
use super::learning::most;
use super::measures::{Cell, Measures};
use super::moment::{minutes, plural, Moment};
use super::month::HALF;
use super::outcomes::Outcomes;
use super::panel::pair;
use super::project::Project;
use super::sheet::{Group, Sheet, SPARK};
use super::{allocation, daily, learning, output, quality};
use crate::terminal::canvas::{
    paint, BOLD, CYAN, DIM, EDGE, FRAME, GREEN, LIGHT, MAGENTA, RED, YELLOW,
};
use ratatui::style::Style;

/// The columns a closed section's small picture takes at most.
const PICTURE: u16 = 24;

/// A section of the board, in the order it shows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Section {
    Learning,
    Quality,
    Output,
    Resources,
}

impl Section {
    /// Every section, in order.
    pub const ALL: [Section; 4] = [
        Section::Learning,
        Section::Quality,
        Section::Output,
        Section::Resources,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Section::Learning => "Learning",
            Section::Quality => "Quality",
            Section::Output => "Output and speed",
            Section::Resources => "Resources",
        }
    }

    /// Draws it from row `y`, its place `n`: opened to its details or closed to its card; the row after.
    pub fn draw(self, s: &mut Sheet, m: &Moment, y: u16, n: usize) -> u16 {
        match m.open[n] {
            true => self.opened(s, m, y, n),
            false => self.closed(s, m, y, n),
        }
    }

    /// It opened: `▾` and its name as a heading, light when picked, then its details; the row after.
    fn opened(self, s: &mut Sheet, m: &Moment, y: u16, n: usize) -> u16 {
        let picked = m.focus == n;
        let light = |look: Style| if picked { LIGHT } else { look };
        s.clip = None;
        s.alpha = 1.0;
        let mut x = s.put(2, y, "▾ ", light(DIM));
        x = s.put(x, y, self.name(), light(Style::new()).patch(BOLD));
        let line = "─".repeat(usize::from(m.width.saturating_sub(3 + x)));
        s.put(x + 1, y, &line, light(FRAME));
        let y = y + 2;
        let full = m.grid().span(0, 12);
        match self {
            Section::Learning => {
                let y = pair(s, m, y, &learning::panels());
                match learning::top(m) {
                    Some(top) => y + top.place(s, m, (full.0, y, full.1), 0) + 1,
                    None => y,
                }
            }
            Section::Quality => y + quality::panel().place(s, m, (full.0, y, full.1), 0) + 1,
            Section::Output => {
                let y = pair(s, m, y, &output::panels());
                y + daily::panel(m).place(s, m, (full.0, y, full.1), 0) + 1
            }
            Section::Resources => y + allocation::panel().place(s, m, (full.0, y, full.1), 0),
        }
    }

    /// It closed: one card across the grid with `▸` and its name in its border, light when picked, its key
    /// measures as a table and a small picture at their right from the text column of track
    /// nine, or six when nine leaves it too little room, or under them when six does too; the row after
    /// it.
    fn closed(self, s: &mut Sheet, m: &Moment, y: u16, n: usize) -> u16 {
        let picked = m.focus == n;
        let grid = m.grid();
        let (x, w) = grid.span(0, 12);
        let end = x + w - 1;
        let column = [9, 6]
            .into_iter()
            .map(|k| grid.edge(k) + 2)
            .find(|at| end.saturating_sub(*at) >= PICTURE);
        let beside = column.is_some();
        let room = match column {
            Some(at) => at - (x + 2) - 2,
            None => w - 4,
        };
        let cells = self.summary(m);
        let right = x + 2 + room;
        let rows = Measures::of(grid, (x + 2, right), &cells, 0).rows;
        let height = 2 + rows + u16::from(!beside);
        s.clip = None;
        s.alpha = m.fade(0.3 + 0.08 * n as f64);
        let (border, mark) = match picked {
            true => (LIGHT, LIGHT),
            false => (EDGE, DIM),
        };
        let title = [
            paint(mark, " ▸ "),
            paint(BOLD, &format!("{} ", self.name().to_uppercase())),
        ];
        s.boxed((x, y, w, height), border, &title);
        s.clip = Some((x + 1, end));
        s.measures(grid, (x + 2, y + 1, right), &cells, 0);
        match column {
            Some(at) => self.mini(s, m, at, y + 1),
            None => self.mini(s, m, x + 2, y + 1 + rows),
        }
        s.clip = None;
        y + height + 1
    }

    /// Its key measures that the tiles do not show as cells of a table in one unit, steps or time, each
    /// value in the colour of its details, with its change in that unit after it.
    fn summary(self, m: &Moment) -> Vec<Cell> {
        let (now, month) = (m.now(), m.month);
        let steps = |now: usize, count: fn(Outcomes) -> usize, up: bool| -> Group {
            let now = now as i64;
            let prior = m.before(|o| count(o) as i64).filter(|prior| *prior != now);
            let change = prior.map(|prior| Kind::Count.change(Some(now), Some(prior), up));
            change.into_iter().flatten().flatten().collect()
        };
        let lead = |look: Style, value: String, name: &str| -> Group {
            vec![
                paint(look.patch(BOLD), &value),
                paint(DIM, &format!(" {name}")),
            ]
        };
        match self {
            Section::Learning => vec![
                (
                    lead(MAGENTA, now.applied.to_string(), "applied a pattern"),
                    steps(now.applied, |o| o.applied, true),
                ),
                (
                    lead(
                        CYAN,
                        now.reused.to_string(),
                        &format!("reused {}", plural(now.reused, "step")),
                    ),
                    steps(now.reused, |o| o.reused, true),
                ),
            ],
            Section::Quality => vec![
                (
                    lead(YELLOW, now.reworked.to_string(), "reworked"),
                    steps(now.reworked, |o| o.reworked, false),
                ),
                (
                    lead(RED, now.dropped.to_string(), "dropped"),
                    steps(now.dropped, |o| o.dropped, false),
                ),
                (
                    lead(DIM, now.expired.to_string(), "expired"),
                    steps(now.expired, |o| o.expired, false),
                ),
            ],
            Section::Output => vec![(
                lead(
                    Style::new(),
                    month.spans[0].to_string(),
                    &format!("of {} successful steps under 5m", now.successful()),
                ),
                vec![],
            )],
            Section::Resources => {
                let lost: u64 = month.projects.iter().map(|p| p.rework + p.dropped).sum();
                vec![
                    (
                        lead(Style::new(), minutes(month.time()), "active"),
                        allocation::spent(m),
                    ),
                    (lead(YELLOW, minutes(lost), "reworked or dropped"), vec![]),
                ]
            }
        }
    }

    /// Its small picture at `x`, `y`: the weekly reuse rate, the outcomes as one bar, the steps resolved
    /// each day, or the active time as productive, rework and dropped.
    fn mini(self, s: &mut Sheet, m: &Moment, x: u16, y: u16) {
        let (now, month) = (m.now(), m.month);
        match self {
            Section::Learning => {
                let weeks = month.weeks();
                let most = most(m) as f64;
                for (n, week) in weeks.iter().enumerate() {
                    let level = (week.reuse_rate() as f64 / most * 8.0).round_ties_even() as usize;
                    s.put(x + n as u16 * 2, y, SPARK[level.clamp(1, 8)], CYAN);
                }
                let (first, last) = (weeks[0].reuse_rate(), weeks[3].reuse_rate());
                s.put(x + 8, y, &format!("{first}% → {last}% by week"), DIM);
            }
            Section::Quality => {
                let parts = [
                    (now.first_pass() as f64, GREEN),
                    (now.reworked as f64, YELLOW),
                    (now.dropped as f64, RED),
                    (now.expired as f64, DIM),
                ];
                s.stack(x, y, &parts, 16);
            }
            Section::Output => {
                let days: Vec<usize> = month.days[HALF..]
                    .iter()
                    .map(|d| outcomes(*d).iter().sum())
                    .collect();
                let top = days.iter().copied().max().unwrap_or(0).max(1);
                let mut end = x;
                for v in days {
                    let (glyph, look) = match v {
                        0 => ("⡀", DIM),
                        v => {
                            let level = (v as f64 / top as f64 * 8.0).round_ties_even() as usize;
                            (SPARK[level.clamp(1, 8)], OUTCOMES[0].1)
                        }
                    };
                    end = s.put(end, y, glyph, look);
                }
                s.put(end + 1, y, "a day", DIM);
            }
            Section::Resources => {
                let sum = |part: fn(&Project) -> u64| month.projects.iter().map(part).sum::<u64>();
                let parts = [
                    (sum(Project::productive), GREEN),
                    (sum(|p| p.rework), YELLOW),
                    (sum(|p| p.dropped), RED),
                ];
                let parts = parts.map(|(v, look)| (v as f64 / 60.0, look));
                if parts.iter().any(|(v, _)| *v > 0.0) {
                    s.stack(x, y, &parts, 16);
                }
            }
        }
    }
}
