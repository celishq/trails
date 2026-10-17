//! One card of stats: the face it shows among the cards, and the table it opens to.

use super::glyph::{bar, columns, strip};
use super::ink::Ink;
use super::table::{Row, Table};
use crate::command::tree::{cut, look, visible};
use crate::progress::{Grade, Period, Reuse, Tally};
use crate::text::clock::{day, midnight, took};
use crate::text::line::plural;
use celis_trails::{State, Step, Trails};
use std::collections::BTreeMap;

/// A card of stats: its title, the rows of its face, and the table it opens to.
pub(crate) struct Sheet {
    pub title: String,
    pub face: Vec<String>,
    pub table: Table,
}

/// The days of one period a card compares.
const DAYS: u64 = 14;

/// The seconds of one period a card compares.
pub(crate) const SPAN: u64 = DAYS * DAY;

const DAY: u64 = 86_400;

/// The count of cards: effect, reuse, rules used, work, time, outcome and projects.
pub(crate) const CARDS: usize = 7;

/// The fewest records a number of the effect counts before it shows a change.
const FEW: usize = 10;

/// The cards of `trails` at its time: the effect `full` characters wide inside, the others `inside`;
/// their numbers plain text when `plain`.
pub(crate) fn sheets(
    trails: &Trails,
    full: usize,
    inside: usize,
    plain: bool,
    colour: bool,
) -> [Sheet; CARDS] {
    let steps = trails.steps();
    let reuse = Reuse::new(&steps);
    let now = trails.now;
    let last = Period::new(&steps, now.saturating_sub(SPAN), now + 1);
    let before = Period::new(
        &steps,
        now.saturating_sub(2 * SPAN),
        now.saturating_sub(SPAN),
    );
    let ink = Ink { plain, colour };
    [
        effect(&last, &before, full, ink),
        followed(&reuse, inside, ink),
        rules(trails, &reuse, inside, ink),
        work(&steps, &before, now, inside, ink),
        time(&last, inside, ink),
        outcome(&last, inside, ink),
        projects(&last, inside, ink),
    ]
}

/// `no step <what> in 14 days`, the line of a card with nothing in its period.
fn nothing(what: &str) -> String {
    format!("no step {what} in {DAYS} days")
}

/// One number of the effect: its label, its reading in each period, the word for the records it
/// counts, and whether more is better.
struct Measure {
    label: &'static str,
    last: Reading,
    before: Reading,
    unit: &'static str,
    up: bool,
}

/// A measure in one period: its value, none with no records, and the records it counts.
#[derive(Clone, Copy)]
struct Reading {
    value: Option<i64>,
    base: usize,
}

impl Reading {
    /// The share of `part` in `base`, in percent.
    fn share((part, base): (usize, usize)) -> Reading {
        Reading {
            value: (base > 0).then(|| (part * 100 / base) as i64),
            base,
        }
    }
}

impl Measure {
    /// A share of `part` in `base` for each period, in percent.
    fn share(
        label: &'static str,
        last: (usize, usize),
        before: (usize, usize),
        unit: &'static str,
        up: bool,
    ) -> Measure {
        Measure {
            label,
            last: Reading::share(last),
            before: Reading::share(before),
            unit,
            up,
        }
    }

    /// `value` as the card shows it.
    fn text(&self, value: Option<i64>) -> String {
        value.map_or("—".to_string(), |v| format!("{v}%"))
    }

    /// The change from the period before, coloured green when better and red when worse, dim when it
    /// moved little; none when the period before counts none, `· few records` when a period counts
    /// fewer than `FEW`.
    fn change(&self, ink: Ink) -> Option<String> {
        if self.before.base == 0 {
            return None;
        }
        let (Some(last), Some(before)) = (self.last.value, self.before.value) else {
            return Some(ink.dim("· few records"));
        };
        if self.last.base < FEW || self.before.base < FEW {
            return Some(ink.dim("· few records"));
        }
        let moved = last - before;
        let by = match moved.signum() {
            0 => "=".to_string(),
            1 => format!("▲ {moved}"),
            _ => format!("▼ {}", -moved),
        };
        let text = format!(
            "{} → {}  {by}",
            self.text(Some(before)),
            self.text(Some(last))
        );
        let tone = match (moved.abs() < 3, (moved > 0) == self.up) {
            (true, _) => "2",
            (false, true) => "32",
            (false, false) => "31",
        };
        Some(ink.paint(tone, &text))
    }
}

/// The effect: as planned, rework and rules reused in the last 14 days, against the 14 days before.
fn effect(last: &Period, before: &Period, inside: usize, ink: Ink) -> Sheet {
    let done = |p: &Period| p.done().len();
    let measures = [
        Measure::share(
            "AS PLANNED",
            (last.planned(), done(last)),
            (before.planned(), done(before)),
            "step",
            true,
        ),
        Measure::share(
            "REWORK",
            (last.reworked(), done(last)),
            (before.reworked(), done(before)),
            "step",
            false,
        ),
        Measure::share(
            "RULES REUSED",
            (last.ruled(), last.added().len()),
            (before.ruled(), before.added().len()),
            "step",
            true,
        ),
    ];
    let column = (inside / measures.len()).max(1);
    let cell = |text: String| {
        let text = cut(&text, column - 1);
        let pad = column.saturating_sub(visible(&text));
        format!("{text}{}", " ".repeat(pad))
    };
    let mut face = vec![measures
        .iter()
        .map(|m| cell(ink.dim(m.label)))
        .collect::<String>()];
    let bigs: Vec<Vec<String>> = measures
        .iter()
        .map(|m| ink.big(&m.text(m.last.value)))
        .collect();
    for r in 0..bigs[0].len() {
        face.push(bigs.iter().map(|b| cell(b[r].clone())).collect());
    }
    let changes: Vec<Option<String>> = measures.iter().map(|m| m.change(ink)).collect();
    let compared = changes.iter().any(Option::is_some);
    if compared {
        face.push(
            changes
                .into_iter()
                .map(|c| cell(c.unwrap_or_default()))
                .collect(),
        );
    }
    face.push(
        measures
            .iter()
            .map(|m| cell(ink.dim(&plural(m.last.base, m.unit))))
            .collect(),
    );
    let rows = measures
        .iter()
        .map(|m| Row {
            id: None,
            cells: vec![
                title_case(m.label),
                m.text(m.last.value),
                m.text(m.before.value),
                plural(m.last.base, m.unit),
            ],
        })
        .collect();
    Sheet {
        title: match compared {
            true => format!("00 EFFECT · LAST {DAYS} DAYS vs {DAYS} BEFORE"),
            false => format!("00 EFFECT · LAST {DAYS} DAYS"),
        },
        face,
        table: Table {
            head: vec!["Measure", "Last 14 days", "14 days before", "Base"],
            rows,
            wide: 0,
        },
    }
}

/// The reuse: the follows, and the steps followed most.
fn followed(reuse: &Reuse, inside: usize, ink: Ink) -> Sheet {
    let side = [ink.dim("follows")];
    let mut face = ink.beside(&reuse.total().to_string(), &side, 2);
    face.push(String::new());
    let steps = reuse.followed();
    if steps.is_empty() {
        face.push(ink.dim("no step followed another yet"));
    }
    for &step in steps.iter().take(3) {
        let tail = format!(" ──▶ {:>3}", reuse.down(step).len());
        let title = cut(step.title(), inside.saturating_sub(visible(&tail) + 1));
        let pad = " ".repeat(inside.saturating_sub(visible(&title) + visible(&tail)));
        face.push(format!("{title}{pad}{tail}"));
    }
    let rows = steps
        .iter()
        .map(|&s| Row {
            id: Some(s.id().to_string()),
            cells: vec![
                s.title().to_string(),
                s.record().anchor.name.clone(),
                reuse.down(s).len().to_string(),
            ],
        })
        .collect();
    Sheet {
        title: "01 REUSE".into(),
        face,
        table: Table {
            head: vec!["Step", "Project", "Followed"],
            rows,
            wide: 0,
        },
    }
}

/// The rules used: the decisions later steps used, most used first, as bars.
fn rules(trails: &Trails, reuse: &Reuse, inside: usize, ink: Ink) -> Sheet {
    let used = reuse.used(&trails.decisions());
    let most = used.first().map_or(0, |(_, _, n)| *n);
    let label = 18.min(inside / 2);
    let width = inside.saturating_sub(label + 5);
    let mut face: Vec<String> = used
        .iter()
        .take(5)
        .map(|(_, d, n)| {
            let name = cut(d.brief().trim_start_matches("When "), label);
            let pad = " ".repeat(label - visible(&name));
            format!("{name}{pad} {} {n:>2}", bar(*n, most, width))
        })
        .collect();
    let total: usize = used.iter().map(|(_, _, n)| n).sum();
    match used.len() {
        0 => face.push(ink.dim("no rule used yet")),
        n => {
            let more = match n.saturating_sub(5) {
                0 => String::new(),
                k => format!("+ {k} more rules"),
            };
            let uses = format!("{total} uses");
            let pad = " ".repeat(inside.saturating_sub(visible(&more) + uses.len()));
            face.extend([String::new(), ink.dim(&format!("{more}{pad}{uses}"))]);
        }
    }
    let rows = used
        .iter()
        .map(|(note, d, n)| Row {
            id: Some(note.id.clone()),
            cells: vec![d.content.clone(), n.to_string()],
        })
        .collect();
    Sheet {
        title: "02 RULES USED".into(),
        face,
        table: Table {
            head: vec!["Rule", "Used"],
            rows,
            wide: 0,
        },
    }
}

/// The work: the steps done each day of the last 14, as columns, and their total against the 14 days
/// before when those hold a done step.
fn work(steps: &celis_trails::Steps, before: &Period, now: u64, inside: usize, ink: Ink) -> Sheet {
    let today = midnight(now);
    let starts: Vec<u64> = (0..DAYS)
        .rev()
        .map(|i| today.saturating_sub(i * DAY))
        .collect();
    let counts: Vec<usize> = starts
        .iter()
        .map(|&s| Period::new(steps, s, s + DAY).done().len())
        .collect();
    let total: usize = counts.iter().sum();
    let earlier = before.done().len();
    let moved = match (earlier, total >= earlier) {
        (0, _) => None,
        (_, true) => Some(format!("▲ {} vs {DAYS} days before", total - earlier)),
        (_, false) => Some(format!("▼ {} vs {DAYS} days before", earlier - total)),
    };
    let moved = moved.map(|m| ink.dim(&m));
    let side: Vec<String> = std::iter::once(ink.dim("steps done"))
        .chain(moved.clone())
        .collect();
    let mut face = ink.beside(&total.to_string(), &side, 2);
    if ink.plain {
        face.extend(moved);
    }
    face.push(String::new());
    if total == 0 {
        face.push(ink.dim(&nothing("done")));
    } else {
        face.extend(columns(&counts, 2, &ink.dim("·")));
        let first = day(starts[0]);
        let pad = " ".repeat(27usize.min(inside).saturating_sub(first.len() + 5));
        face.push(ink.dim(&format!("{first}{pad}today")));
    }
    let rows = starts
        .iter()
        .zip(&counts)
        .rev()
        .map(|(&s, n)| Row {
            id: None,
            cells: vec![day(s), n.to_string()],
        })
        .collect();
    Sheet {
        title: format!("03 WORK · {DAYS} DAYS"),
        face,
        table: Table {
            head: vec!["Day", "Steps done"],
            rows,
            wide: 0,
        },
    }
}

/// The steps done in `period` with no step under them, which the time a step took counts.
fn leaves<'a>(period: &Period<'a>) -> Vec<Step<'a>> {
    period
        .done()
        .into_iter()
        .filter(|s| s.children().is_empty())
        .collect()
}

/// The time a step took: its median and its spread over five spans, as bars.
fn time(last: &Period, inside: usize, ink: Ink) -> Sheet {
    let mut done: Vec<(u64, Step)> = leaves(last)
        .into_iter()
        .filter_map(|s| Some((Grade::of(s)?.took, s)))
        .collect();
    done.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    let spans: [(&str, u64); 5] = [
        ("< 5m", 300),
        ("5-15m", 900),
        ("15m-1h", 3_600),
        ("1-4h", 14_400),
        ("> 4h", u64::MAX),
    ];
    let mut counts = [0usize; 5];
    for (t, _) in &done {
        let at = spans.iter().position(|(_, top)| t < top).unwrap_or(4);
        counts[at] += 1;
    }
    let most = counts.iter().copied().max().unwrap_or(0);
    let width = inside.saturating_sub(13);
    let face = match done.get(done.len() / 2) {
        None => vec![ink.dim(&nothing("done"))],
        Some((median, _)) => {
            let mut face = vec![
                format!("{} {}", ink.dim("median"), ink.paint("1", &took(*median))),
                String::new(),
            ];
            for ((label, _), n) in spans.iter().zip(counts) {
                face.push(format!("{label:<7} {} {n:>3}", bar(n, most, width)));
            }
            face
        }
    };
    let rows = done
        .iter()
        .map(|(t, s)| Row {
            id: Some(s.id().to_string()),
            cells: vec![s.title().to_string(), took(*t)],
        })
        .collect();
    Sheet {
        title: "04 TIME A STEP TOOK".into(),
        face,
        table: Table {
            head: vec!["Step", "Took"],
            rows,
            wide: 0,
        },
    }
}

/// The outcome: the steps closed in the last 14 days by how they ended, as a strip of shares, with the
/// count of revised done steps beside the done share.
fn outcome(last: &Period, inside: usize, ink: Ink) -> Sheet {
    let closed = last.closed();
    let tally = Tally::of(&closed);
    let states = [State::Done, State::Dropped, State::Expired];
    let counts: Vec<usize> = states.iter().map(|&st| tally.count(st)).collect();
    let total: usize = counts.iter().sum();
    let face = match total {
        0 => vec![ink.dim(&nothing("closed"))],
        _ => {
            let mut face = vec![strip(&counts, inside.saturating_sub(4)), String::new()];
            for (&state, &n) in states.iter().zip(&counts) {
                let look = look(state);
                let share = n * 100 / total;
                let mut line = format!(
                    "{} {:<10}{n:>4} {share:>5}%",
                    ink.paint(look.text, look.mark),
                    state.text()
                );
                if state == State::Done && tally.revised > 0 {
                    line.push_str(&format!(" · {} revised", tally.revised));
                }
                face.push(line);
            }
            face
        }
    };
    let rows = closed
        .iter()
        .map(|s| Row {
            id: Some(s.id().to_string()),
            cells: vec![
                format!("{} {}", look(s.state()).mark, s.state().text()),
                s.title().to_string(),
            ],
        })
        .collect();
    Sheet {
        title: "05 OUTCOME".into(),
        face,
        table: Table {
            head: vec!["State", "Step"],
            rows,
            wide: 1,
        },
    }
}

/// The projects: the time the steps done in the last 14 days took in each, most first, as bars.
fn projects(last: &Period, inside: usize, ink: Ink) -> Sheet {
    let mut spent: BTreeMap<String, (u64, usize)> = BTreeMap::new();
    for step in leaves(last) {
        let entry = spent.entry(step.record().anchor.name.clone()).or_default();
        entry.0 += Grade::of(step).map_or(0, |g| g.took);
        entry.1 += 1;
    }
    let mut spent: Vec<(String, (u64, usize))> = spent.into_iter().collect();
    spent.sort_by_key(|(name, (t, _))| (std::cmp::Reverse(*t), name.clone()));
    let most = spent.first().map_or(0, |(_, (t, _))| *t as usize);
    let label = 14.min(inside / 2);
    let width = inside.saturating_sub(label + 9);
    let mut face: Vec<String> = spent
        .iter()
        .take(4)
        .map(|(name, (t, _))| {
            let name = cut(name, label);
            let pad = " ".repeat(label - visible(&name));
            format!(
                "{}{pad} {} {:>6}",
                ink.paint("1", &name),
                bar(*t as usize, most, width),
                took(*t)
            )
        })
        .collect();
    if face.is_empty() {
        face.push(ink.dim(&nothing("done")));
    }
    let rows = spent
        .iter()
        .map(|(name, (t, n))| Row {
            id: None,
            cells: vec![name.clone(), took(*t), n.to_string()],
        })
        .collect();
    Sheet {
        title: "06 PROJECTS · TIME".into(),
        face,
        table: Table {
            head: vec!["Project", "Time", "Steps"],
            rows,
            wide: 0,
        },
    }
}

/// `label` in sentence case, as `As planned`.
fn title_case(label: &str) -> String {
    let lower = label.to_lowercase();
    let mut chars = lower.chars();
    chars.next().map_or(String::new(), |c| {
        c.to_uppercase().collect::<String>() + chars.as_str()
    })
}

#[cfg(test)]
mod tests;
