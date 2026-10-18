//! The steps of a store, as they stand at one moment.

use crate::{Decision, Mark, Refusal, Step, StepRecord};
use celis_nostr::Note;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

/// The step records and marks of a store at the time `now`: which steps are open, how they nest, and how
/// each went.
pub struct Steps<'a> {
    now: Cell<u64>,
    index: Index<'a>,
}

impl<'a> Steps<'a> {
    pub fn new(records: &'a [Note], now: u64) -> Steps<'a> {
        Steps {
            now: Cell::new(now),
            index: Index::of(records),
        }
    }

    pub fn now(&self) -> u64 {
        self.now.get()
    }

    /// Moves it to the time `now`, with the same records.
    pub fn at(&self, now: u64) {
        self.now.set(now);
    }

    /// Every step, oldest first.
    pub fn all(&'a self) -> Vec<Step<'a>> {
        let all = self.index.all.iter();
        all.filter_map(|n| self.get(&n.id)).collect()
    }

    /// Whether a decision record has the id `id`.
    pub(crate) fn holds_decision(&self, id: &str) -> bool {
        self.index.decisions.contains(id)
    }

    /// The step whose id is `id`.
    pub fn get(&'a self, id: &str) -> Option<Step<'a>> {
        Some(Step::new(self, self.index.entries.get(id)?))
    }

    /// The step whose id is `id`, or the only step whose id starts with `id` when `id` is 4 or more
    /// characters.
    pub fn find(&'a self, id: &str) -> Result<Step<'a>, Refusal> {
        let note = match self.index.entries.get(id) {
            Some(entry) => Some(entry.note),
            None => crate::records::id::find(self.index.all.iter().copied(), id),
        };
        note.and_then(|n| self.get(&n.id))
            .ok_or_else(|| Refusal::NoStep(id.to_string()))
    }

    /// The step `find` gives for `id`; `closed` when a done or a drop names it.
    pub fn find_open(&'a self, id: &str) -> Result<Step<'a>, Refusal> {
        let step = self.find(id)?;
        match step.closed() {
            Some(_) => Err(Refusal::Closed(step.id().to_string())),
            None => Ok(step),
        }
    }

    /// Whether a step that is not a trail has ended: done, dropped or expired.
    pub fn closed_any(&'a self) -> bool {
        let all = self.all().into_iter();
        all.filter(|s| !s.is_trail())
            .any(|s| s.closed_at().is_some())
    }

    /// Every trail of the project `project`, oldest first.
    pub fn trails(&'a self, project: &str) -> Vec<Step<'a>> {
        let roots = self.roots().into_iter();
        roots
            .filter(|s| s.record().anchor.project == project)
            .collect()
    }

    /// Every trail of every project, oldest first.
    pub fn roots(&'a self) -> Vec<Step<'a>> {
        self.all().into_iter().filter(|s| s.is_trail()).collect()
    }

    /// The open trail of the session `session` in the project `project`: the trail of its newest step,
    /// when it is not done or dropped.
    pub fn session(&'a self, project: &str, session: &str) -> Option<Step<'a>> {
        let newest = self.all().into_iter().rev().find(|s| {
            let record = s.record();
            record.anchor.project == project && record.session.as_deref() == Some(session)
        })?;
        let trail = newest.trail();
        trail.closed().is_none().then_some(trail)
    }
}

/// The seconds a trail stays active after the newest record of its tree.
pub const LIFE: u64 = 30 * 60;

/// The step records, oldest first, and what the marks say of each step.
pub(crate) struct Index<'a> {
    all: Vec<&'a Note>,
    entries: HashMap<&'a str, Entry<'a>>,
    decisions: HashSet<&'a str>,
}

/// One step: its record, its text with its edits, its links, the steps that replace it, and how its marks
/// left it; of its tree, the newest record, whether a step works, and whether a step under it is open.
pub(crate) struct Entry<'a> {
    pub(crate) note: &'a Note,
    pub(crate) step: StepRecord,
    pub(crate) children: Vec<&'a Note>,
    pub(crate) replaced: Vec<&'a Note>,
    pub(crate) start: Option<&'a Note>,
    pub(crate) close: Option<(&'a Note, Close)>,
    pub(crate) touched: u64,
    pub(crate) newest: u64,
    pub(crate) busy: bool,
    pub(crate) loose: bool,
}

/// How a step was closed.
#[derive(Clone, Copy)]
pub(crate) enum Close {
    Done,
    Dropped,
}

impl<'a> Index<'a> {
    /// The index of `records`, read in the order they were written.
    fn of(records: &'a [Note]) -> Index<'a> {
        let mut sorted: Vec<&'a Note> = records.iter().collect();
        sorted.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        let mut all = Vec::new();
        let mut entries: HashMap<&'a str, Entry<'a>> = HashMap::new();
        for &note in &sorted {
            if let Some(step) = StepRecord::read(note) {
                all.push(note);
                entries.insert(
                    &note.id,
                    Entry {
                        note,
                        step,
                        children: Vec::new(),
                        replaced: Vec::new(),
                        start: None,
                        close: None,
                        touched: note.created_at,
                        newest: 0,
                        busy: false,
                        loose: false,
                    },
                );
            }
        }
        for &note in &all {
            let step = &entries[note.id.as_str()].step;
            let (under, replaces) = (step.under.clone(), step.replaces.clone());
            if let Some(parent) = under.and_then(|id| entries.get_mut(&*id)) {
                parent.children.push(note);
            }
            if let Some(old) = replaces.and_then(|id| entries.get_mut(&*id)) {
                old.replaced.push(note);
            }
        }
        for &note in &sorted {
            let Some((id, mark)) = Mark::read(note) else {
                continue;
            };
            let Some(entry) = entries.get_mut(&*id) else {
                continue;
            };
            entry.touched = entry.touched.max(note.created_at);
            entry.mark(note, mark);
        }
        sum(&mut entries);
        let decisions = sorted.iter().filter(|n| Decision::read(n).is_some());
        let decisions = decisions.map(|n| n.id.as_str()).collect();
        Index {
            all,
            entries,
            decisions,
        }
    }
}

/// Sets on each entry of `entries` what its tree holds, each step's tree after the trees under it.
fn sum<'a>(entries: &mut HashMap<&'a str, Entry<'a>>) {
    let mut order: Vec<&'a str> = entries
        .iter()
        .filter(|(_, e)| {
            e.step
                .under
                .as_deref()
                .is_none_or(|u| !entries.contains_key(u))
        })
        .map(|(id, _)| *id)
        .collect();
    let mut i = 0;
    while i < order.len() {
        order.extend(entries[order[i]].children.iter().map(|n| n.id.as_str()));
        i += 1;
    }
    for id in order.into_iter().rev() {
        let entry = &entries[id];
        let mut newest = entry.touched;
        let mut busy = entry.start.is_some() && entry.close.is_none();
        let mut loose = false;
        for child in &entry.children {
            let child = &entries[child.id.as_str()];
            newest = newest.max(child.newest);
            busy |= child.busy;
            loose |= child.loose || child.close.is_none();
        }
        if let Some(entry) = entries.get_mut(id) {
            (entry.newest, entry.busy, entry.loose) = (newest, busy, loose);
        }
    }
}

impl<'a> Entry<'a> {
    /// Takes in the mark `mark` that the record `note` holds, in the order the marks were written.
    fn mark(&mut self, note: &'a Note, mark: Mark) {
        match mark {
            Mark::Start => {
                self.start.get_or_insert(note);
            }
            Mark::Done { proof, .. } if !proof.trim().is_empty() => {
                self.close.get_or_insert((note, Close::Done));
            }
            Mark::Drop { reason } if !reason.trim().is_empty() => {
                self.close.get_or_insert((note, Close::Dropped));
            }
            Mark::Edit {
                text,
                places,
                check,
            } => {
                let step = &mut self.step;
                step.content = match (
                    text.trim().is_empty(),
                    text.lines().count(),
                    step.content.split_once('\n'),
                ) {
                    (true, _, _) => std::mem::take(&mut step.content),
                    (false, 1, Some((_, detail))) => format!("{}\n{detail}", text.trim_end()),
                    (false, _, _) => text,
                };
                if !places.is_empty() {
                    step.places = places;
                }
                step.check = check.or(step.check.take());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
