//! A section of the stats board: closed to one card of its key measures, or opened to its details.

use super::daily::{outcomes, OUTCOMES};
use super::kind::Kind;
use super::learning::most;
use super::moment::{minutes, plural, Moment};
use super::month::HALF;
use super::panel::pair;
use super::sheet::{place, Group, Sheet, SPARK};
use super::{allocation, daily, learning, output, quality};
use crate::terminal::canvas::{
    paint, BOLD, CYAN, DIM, EDGE, FRAME, LIGHT, MAGENTA, RED, SLATE, YELLOW,
};
use ratatui::style::Style;

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
        let full = (2, m.width - 4);
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

    /// It closed: one card with `▸` and its name in its border, light when picked, its key measures
    /// wrapped to the width and a small picture at their right, or under them when the card is narrow;
    /// the row after it.
    fn closed(self, s: &mut Sheet, m: &Moment, y: u16, n: usize) -> u16 {
        let picked = m.focus == n;
        let w = m.width - 4;
        let beside = w >= 80;
        let room = if beside { w - 4 - 28 } else { w - 4 };
        let groups = self.summary(m);
        let (_, rows) = place(&groups, room);
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
        s.boxed((2, y, w, height), border, &title);
        s.clip = Some((3, 2 + w - 1));
        s.flow(4, y + 1, room, &groups);
        match beside {
            true => self.mini(s, m, 2 + w - 2 - 26, y + 1),
            false => self.mini(s, m, 4, y + 1 + rows),
        }
        s.clip = None;
        y + height + 1
    }

    /// Its key measures that the tiles do not show, each value in the colour of its details, with its
    /// change beside it.
    fn summary(self, m: &Moment) -> Vec<Group> {
        let (now, month) = (m.now(), m.month);
        let dot = || paint(DIM, "  ·  ");
        let arrow = |kind: Kind, now: i64, prior: Option<i64>, up: bool| {
            kind.change(Some(now), prior, up).remove(0)
        };
        match self {
            Section::Learning => vec![
                vec![
                    paint(MAGENTA.patch(BOLD), &format!("{}%", now.rule_usage())),
                    paint(DIM, " rule usage"),
                ],
                arrow(
                    Kind::Share,
                    now.rule_usage(),
                    m.before(|o| o.rule_usage()),
                    true,
                ),
                vec![
                    dot(),
                    paint(CYAN.patch(BOLD), &now.reused.to_string()),
                    paint(DIM, &format!(" reused {}", plural(now.reused, "step"))),
                ],
            ],
            Section::Quality => vec![
                vec![
                    paint(YELLOW.patch(BOLD), &format!("{}%", now.rework_rate())),
                    paint(DIM, " reworked"),
                ],
                arrow(
                    Kind::Share,
                    now.rework_rate(),
                    m.before(|o| o.rework_rate()),
                    false,
                ),
                vec![
                    dot(),
                    paint(RED.patch(BOLD), &now.dropped.to_string()),
                    paint(DIM, " dropped "),
                    paint(DIM.patch(BOLD), &now.expired.to_string()),
                    paint(DIM, " expired"),
                ],
            ],
            Section::Output => vec![vec![
                paint(BOLD, &month.spans[0].to_string()),
                paint(
                    DIM,
                    &format!(" of {} successful steps under 5m", now.successful()),
                ),
            ]],
            Section::Resources => {
                let lost: u64 = month.projects.iter().map(|p| p.rework + p.dropped).sum();
                vec![
                    vec![paint(BOLD, &minutes(month.time())), paint(DIM, " active")],
                    vec![
                        dot(),
                        paint(YELLOW.patch(BOLD), &minutes(lost)),
                        paint(DIM, " lost to rework and drops"),
                    ],
                    vec![
                        dot(),
                        paint(BOLD, &month.projects.len().to_string()),
                        paint(
                            DIM,
                            &format!(" {}", plural(month.projects.len(), "project")),
                        ),
                    ],
                ]
            }
        }
    }

    /// Its small picture at `x`, `y`: the weekly reuse rate, the outcomes as one bar, the steps resolved
    /// each day, or each project's share of the time.
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
                    (now.first_pass() as f64, CYAN),
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
                let shades = [CYAN, SLATE, EDGE];
                let parts: Vec<(f64, Style)> = month
                    .projects
                    .iter()
                    .enumerate()
                    .map(|(n, p)| (p.active as f64 / 60.0, shades[n.min(2)]))
                    .collect();
                if !parts.is_empty() {
                    s.stack(x, y, &parts, 16);
                }
            }
        }
    }
}
