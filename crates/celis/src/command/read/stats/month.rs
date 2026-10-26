//! The steps of the last 28 days, as every number of stats reads them.

use super::outcomes::Outcomes;
use super::project::Project;
use crate::progress::{Grade, Model};
use crate::text::clock::midnight;
use celis_trails::{State, Step};
use std::collections::{BTreeMap, HashMap};

/// The seconds of one day.
pub(super) const DAY: u64 = 86_400;

/// The days the board reads: the prior 14, then the last 14, today last.
pub(super) const DAYS: usize = 28;

/// The days of one half of the month.
pub(super) const HALF: usize = 14;

/// The top of each span of time a step can take, in seconds, and its name.
pub(super) const SPANS: [(&str, u64); 5] = [
    ("< 5m", 300),
    ("5–15m", 900),
    ("15m–1h", 3_600),
    ("1–4h", 14_400),
    ("> 4h", u64::MAX),
];

/// The steps of a store in the last 28 days: how those of each day ended, the first second of the
/// oldest day, the median time of a successful step in the last and the prior 14 days, how many fell in
/// each span, each project's time and the time of the prior 14 days, the live patterns, those learned and those applied in the last 14 days with the steps that
/// applied one in each project, the five live patterns with the most hits in them, each its id, its words
/// and its hits, and the trails active now.
#[derive(Debug, Default)]
pub(super) struct Month {
    pub days: Vec<Outcomes>,
    pub first: u64,
    pub median: Option<u64>,
    pub prior_median: Option<u64>,
    pub spans: [usize; 5],
    pub projects: Vec<Project>,
    pub prior_active: u64,
    pub patterns: usize,
    pub learned: usize,
    pub kept: usize,
    pub applied: Vec<(String, usize)>,
    pub top: Vec<(String, String, usize)>,
    pub active: usize,
}

impl Month {
    pub fn of(model: &Model) -> Month {
        let steps = model.steps;
        let first = midnight(steps.now()).saturating_sub((DAYS as u64 - 1) * DAY);
        let half = first + HALF as u64 * DAY;
        let mut month = Month {
            days: vec![Outcomes::default(); DAYS],
            first,
            ..Month::default()
        };
        let (mut took, mut earlier) = (vec![], vec![]);
        let mut projects: BTreeMap<String, Project> = BTreeMap::new();
        let mut used: HashMap<&str, usize> = HashMap::new();
        let mut applied: BTreeMap<String, usize> = BTreeMap::new();
        for step in steps.all().into_iter().filter(|s| !s.is_trail()) {
            let Some(at) = step.closed_at().filter(|at| *at >= first) else {
                continue;
            };
            let day = ((at - first) / DAY) as usize;
            if day >= DAYS {
                continue;
            }
            let last = at >= half;
            let name = &step.record().anchor.name;
            let outcome = &mut month.days[day];
            match step.state() {
                State::Done => {
                    let patterns = step.uses();
                    match (step.revised().is_empty(), patterns.is_empty()) {
                        (false, _) => outcome.reworked += 1,
                        (true, false) => outcome.reused += 1,
                        (true, true) => outcome.new += 1,
                    }
                    if !patterns.is_empty() {
                        outcome.applied += 1;
                    }
                    let grade = Grade::of(step).map_or(0, |g| g.took);
                    match last {
                        true => {
                            took.push(grade);
                            if !patterns.is_empty() {
                                *applied.entry(name.clone()).or_default() += 1;
                                for pattern in patterns {
                                    *used.entry(pattern).or_default() += 1;
                                }
                            }
                        }
                        false => earlier.push(grade),
                    }
                }
                State::Dropped => outcome.dropped += 1,
                State::Open => outcome.stale += usize::from(step.stale()),
            }
            if !step.children().is_empty() {
                continue;
            }
            let spent = spent(step);
            if !last {
                month.prior_active += spent;
                continue;
            }
            let project = projects.entry(name.clone()).or_insert_with(|| Project {
                name: name.clone(),
                active: 0,
                rework: 0,
                dropped: 0,
            });
            project.active += spent;
            match step.state() {
                State::Done if !step.revised().is_empty() => project.rework += spent,
                State::Dropped => project.dropped += spent,
                _ => {}
            }
        }
        for t in &took {
            let at = SPANS.iter().position(|(_, top)| t < top).unwrap_or(4);
            month.spans[at] += 1;
        }
        month.median = median(&mut took);
        month.prior_median = median(&mut earlier);
        let mut projects: Vec<Project> = projects.into_values().filter(|p| p.active > 0).collect();
        projects.sort_by(|a, b| b.active.cmp(&a.active).then_with(|| a.name.cmp(&b.name)));
        month.projects = projects;
        let live: Vec<_> = model
            .patterns
            .every()
            .iter()
            .filter(|e| e.is_live())
            .collect();
        month.patterns = live.len();
        month.learned = live.iter().filter(|e| e.note.created_at >= half).count();
        month.kept = live
            .iter()
            .filter(|e| used.contains_key(e.note.id.as_str()))
            .count();
        let mut top: Vec<(String, String, usize)> = live
            .iter()
            .filter_map(|e| {
                let hits = *used.get(e.note.id.as_str())?;
                Some((e.note.id.clone(), e.record.content.clone(), hits))
            })
            .collect();
        top.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        top.truncate(5);
        month.top = top;
        let mut applied: Vec<(String, usize)> = applied.into_iter().collect();
        applied.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        month.applied = applied;
        let roots = steps.roots().into_iter();
        month.active = roots.filter(|s| s.active()).count();
        month
    }

    /// How the steps of the last 14 days ended.
    pub fn now(&self) -> Outcomes {
        Outcomes::sum(&self.days[HALF..])
    }

    /// How the steps of the prior 14 days ended.
    pub fn prior(&self) -> Outcomes {
        Outcomes::sum(&self.days[..HALF])
    }

    /// How the steps of each of the four weeks ended, the oldest first.
    pub fn weeks(&self) -> Vec<Outcomes> {
        self.days.chunks(7).map(Outcomes::sum).collect()
    }

    /// The first second of the day `day`, the oldest 0.
    pub fn start(&self, day: usize) -> u64 {
        self.first + day as u64 * DAY
    }

    /// The seconds the projects took in the last 14 days.
    pub fn time(&self) -> u64 {
        self.projects.iter().map(|p| p.active).sum()
    }
}

/// The seconds `step` took: from its start, or its record, to its done; from its start to its drop.
pub(super) fn spent(step: Step) -> u64 {
    match (step.state(), step.start(), step.dropped()) {
        (State::Done, _, _) => Grade::of(step).map_or(0, |g| g.took),
        (State::Dropped, Some(start), Some(drop)) => {
            drop.created_at.saturating_sub(start.created_at)
        }
        _ => 0,
    }
}

/// The middle of `times`, the mean of the two middles when they are even; none of none.
fn median(times: &mut [u64]) -> Option<u64> {
    if times.is_empty() {
        return None;
    }
    times.sort_unstable();
    let n = times.len();
    Some((times[(n - 1) / 2] + times[n / 2]) / 2)
}

#[cfg(test)]
mod tests;
