//! One card of stats: the face it shows among the cards, and the table it opens to.

use super::card::paint;
use super::clock::{day, midnight, saved, took};
use super::glyph::{bar, columns, digits, strip};
use super::table::{Row, Table};
use super::tree::{cut, look, visible};
use celis_trails::{Period, Reuse, State, Step, Tally, Trails};
use std::collections::BTreeMap;

/// A card of stats: its title, the rows of its face, and the table it opens to.
pub(super) struct Sheet {
    pub title: String,
    pub face: Vec<String>,
    pub table: Table,
}

/// The seconds of one period a card compares.
pub(super) const SPAN: u64 = 14 * DAY;

const DAY: u64 = 86_400;

/// The fewest records a number of the effect counts before it shows a change.
const FEW: usize = 10;

/// The cards of `trails` at its time: the effect `full` characters wide inside, the others `inside`;
/// their numbers plain text when `plain`.
pub(super) fn sheets(
    trails: &Trails,
    full: usize,
    inside: usize,
    plain: bool,
    colour: bool,
) -> Vec<Sheet> {
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
    vec![
        effect(&last, &before, &reuse, full, ink),
        followed(&reuse, inside, ink),
        rules(trails, &reuse, inside, ink),
        work(&steps, &before, now, inside, ink),
        time(&last, inside, ink),
        outcome(&last, inside, ink),
        projects(&last, inside, ink),
    ]
}

/// How a card prints: its numbers as plain text, and in colour.
#[derive(Clone, Copy)]
struct Ink {
    plain: bool,
    colour: bool,
}

impl Ink {
    fn paint(self, code: &str, text: &str) -> String {
        paint(self.colour, code, text)
    }

    /// `text` big and bold: three rows of digits, or one row when plain.
    fn big(self, text: &str) -> Vec<String> {
        match self.plain {
            true => vec![self.paint("1", text)],
            false => digits(text).iter().map(|r| self.paint("1", r)).collect(),
        }
    }
}

/// One number of the effect: its label, its value in each period, the records it counts in each, the
/// word for them, and whether more is better.
struct Measure {
    label: &'static str,
    last: Option<i64>,
    before: Option<i64>,
    bases: (usize, usize),
    unit: &'static str,
    up: bool,
    time: bool,
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
        let percent = |(part, base): (usize, usize)| (base > 0).then(|| (part * 100 / base) as i64);
        Measure {
            label,
            last: percent(last),
            before: percent(before),
            bases: (last.1, before.1),
            unit,
            up,
            time: false,
        }
    }

    /// `value` as the card shows it.
    fn text(&self, value: Option<i64>) -> String {
        match (value, self.time) {
            (None, _) => "—".to_string(),
            (Some(v), true) => saved(v).replace(' ', ""),
            (Some(v), false) => format!("{v}%"),
        }
    }

    /// The change from the period before, coloured green when better and red when worse, dim when it
    /// moved little; `· few records` when a period counts fewer than `FEW`.
    fn change(&self, ink: Ink) -> String {
        let (Some(last), Some(before)) = (self.last, self.before) else {
            return ink.paint("2", "· few records");
        };
        if self.bases.0 < FEW || self.bases.1 < FEW {
            return ink.paint("2", "· few records");
        }
        let moved = last - before;
        let small = match self.time {
            true => moved.abs() < 60,
            false => moved.abs() < 3,
        };
        let step = |v: i64| match self.time {
            true => unit(v),
            false => self.text(Some(v)),
        };
        let by = match (moved.signum(), self.time) {
            (0, _) => "=".to_string(),
            (1, true) => format!("▲ {}", unit(moved)),
            (_, true) => format!("▼ {}", unit(-moved)),
            (1, false) => format!("▲ {moved}"),
            (_, false) => format!("▼ {}", -moved),
        };
        let text = format!("{} → {}  {by}", step(before), step(last));
        let tone = match (small, (moved > 0) == self.up) {
            (true, _) => "2",
            (false, true) => "32",
            (false, false) => "31",
        };
        ink.paint(tone, &text)
    }
}

/// The effect: as planned, rework, rules reused and time saved in the last 14 days, against the 14
/// days before.
fn effect(last: &Period, before: &Period, reuse: &Reuse, inside: usize, ink: Ink) -> Sheet {
    let done = |p: &Period| p.done().len();
    let (follows, gain) = last.follows(reuse);
    let (follows_before, gain_before) = before.follows(reuse);
    let measures = [
        Measure::share(
            "AS PLANNED",
            (last.planned(), done(last)),
            (before.planned(), done(before)),
            "steps",
            true,
        ),
        Measure::share(
            "REWORK",
            (last.reworked(), done(last)),
            (before.reworked(), done(before)),
            "steps",
            false,
        ),
        Measure::share(
            "RULES REUSED",
            (last.ruled(), last.added().len()),
            (before.ruled(), before.added().len()),
            "steps",
            true,
        ),
        Measure {
            label: "TIME SAVED",
            last: gain,
            before: gain_before,
            bases: (follows, follows_before),
            unit: "follows",
            up: true,
            time: true,
        },
    ];
    let column = (inside / measures.len()).max(1);
    let cell = |text: String| {
        let text = cut(&text, column - 1);
        let pad = column.saturating_sub(visible(&text));
        format!("{text}{}", " ".repeat(pad))
    };
    let mut face = vec![measures
        .iter()
        .map(|m| cell(ink.paint("2", m.label)))
        .collect::<String>()];
    let bigs: Vec<Vec<String>> = measures.iter().map(|m| ink.big(&m.text(m.last))).collect();
    for r in 0..bigs[0].len() {
        face.push(bigs.iter().map(|b| cell(b[r].clone())).collect());
    }
    face.push(measures.iter().map(|m| cell(m.change(ink))).collect());
    face.push(
        measures
            .iter()
            .map(|m| cell(ink.paint("2", &format!("{} {}", m.bases.0, m.unit))))
            .collect(),
    );
    let rows = measures
        .iter()
        .map(|m| Row {
            id: None,
            cells: vec![
                title_case(m.label),
                m.text(m.last),
                m.text(m.before),
                format!("{} {}", m.bases.0, m.unit),
            ],
        })
        .collect();
    Sheet {
        title: "00 EFFECT · LAST 14 DAYS vs 14 BEFORE".into(),
        face,
        table: Table {
            head: vec!["Measure", "Last 14 days", "14 days before", "Base"],
            rows,
            wide: 0,
        },
    }
}

/// The reuse: the follows and the time they saved, and the steps followed most.
fn followed(reuse: &Reuse, inside: usize, ink: Ink) -> Sheet {
    let (follows, gain) = reuse.total();
    let big = ink.big(&follows.to_string());
    let room = big.iter().map(|r| visible(r)).max().unwrap_or(0) + 2;
    let side = [
        format!(
            "{}   {}",
            ink.paint("2", "follows"),
            ink.paint("2", "saved")
        ),
        format!("{:<10}{}", "", gain.map_or("—".to_string(), saved)),
    ];
    let mut face: Vec<String> = big
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let pad = " ".repeat(room - visible(r));
            format!("{r}{pad}{}", side.get(i).map_or("", String::as_str))
        })
        .collect();
    face.push(String::new());
    let steps = reuse.followed();
    if steps.is_empty() {
        face.push(ink.paint("2", "no step followed another yet"));
    }
    for &step in steps.iter().take(3) {
        let tail = format!(
            " ──▶ {:<3}{:>8}",
            reuse.down(step).len(),
            reuse.saved(step).map_or(String::new(), saved)
        );
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
                reuse.saved(s).map_or("—".to_string(), saved),
            ],
        })
        .collect();
    Sheet {
        title: "01 REUSE".into(),
        face,
        table: Table {
            head: vec!["Step", "Project", "Followed", "Saved"],
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
        0 => face.push(ink.paint("2", "no rule used yet")),
        n => {
            let more = match n.saturating_sub(5) {
                0 => String::new(),
                k => format!("+ {k} more rules"),
            };
            let uses = format!("{total} uses");
            let pad = " ".repeat(inside.saturating_sub(visible(&more) + uses.len()));
            face.extend([String::new(), ink.paint("2", &format!("{more}{pad}{uses}"))]);
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
/// before.
fn work(steps: &celis_trails::Steps, before: &Period, now: u64, inside: usize, ink: Ink) -> Sheet {
    let today = midnight(now);
    let starts: Vec<u64> = (0..14)
        .rev()
        .map(|i| today.saturating_sub(i * DAY))
        .collect();
    let counts: Vec<usize> = starts
        .iter()
        .map(|&s| Period::new(steps, s, s + DAY).done().len())
        .collect();
    let total: usize = counts.iter().sum();
    let earlier = before.done().len();
    let moved = match total >= earlier {
        true => format!("▲ {} vs 14 days before", total - earlier),
        false => format!("▼ {} vs 14 days before", earlier - total),
    };
    let big = ink.big(&total.to_string());
    let room = big.iter().map(|r| visible(r)).max().unwrap_or(0) + 2;
    let side = [ink.paint("2", "steps done"), ink.paint("2", &moved)];
    let mut face: Vec<String> = big
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let pad = " ".repeat(room - visible(r));
            format!("{r}{pad}{}", side.get(i).map_or("", String::as_str))
        })
        .collect();
    if big.len() == 1 {
        face.push(ink.paint("2", &moved));
    }
    face.push(String::new());
    face.extend(columns(&counts, 2));
    let first = day(starts[0]);
    let pad = " ".repeat(27usize.min(inside).saturating_sub(first.len() + 5));
    face.push(ink.paint("2", &format!("{first}{pad}today")));
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
        title: "03 WORK · 14 DAYS".into(),
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
        .filter_map(|s| Some((s.grade()?.took, s)))
        .collect();
    done.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    let median = match done.len() {
        0 => "—".to_string(),
        n => took(done[n / 2].0),
    };
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
    let mut face = vec![
        format!("{} {}", ink.paint("2", "median"), ink.paint("1", &median)),
        String::new(),
    ];
    for ((label, _), n) in spans.iter().zip(counts) {
        face.push(format!("{label:<7} {} {n:>3}", bar(n, most, width)));
    }
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
    let mut face = vec![strip(&counts, inside.saturating_sub(4)), String::new()];
    for (&state, &n) in states.iter().zip(&counts) {
        let look = look(state);
        let share = (n * 100).checked_div(total).unwrap_or(0);
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
        entry.0 += step.grade().map_or(0, |g| g.took);
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
        face.push(ink.paint("2", "no step done in 14 days"));
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

/// `seconds` in its largest unit only, as `46m`, with a `-` when below zero.
fn unit(seconds: i64) -> String {
    let sign = if seconds < 0 { "-" } else { "" };
    let s = seconds.unsigned_abs();
    match s {
        s if s >= 86_400 => format!("{sign}{}d", s / 86_400),
        s if s >= 3_600 => format!("{sign}{}h", s / 3_600),
        s if s >= 60 => format!("{sign}{}m", s / 60),
        s => format!("{sign}{s}s"),
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
