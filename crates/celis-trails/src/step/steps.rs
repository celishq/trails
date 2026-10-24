//! The steps of a store, as they stand at one moment.

use crate::{Mark, Name, NamedTrail, Refusal, Rule, Step, StepRecord};
use celis_nostr::Note;
use std::cell::Cell;
use std::collections::hash_map::Entry::Vacant;
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

    /// Whether a rule record has the id `id`.
    pub(crate) fn holds_rule(&self, id: &str) -> bool {
        self.index.rules.contains(id)
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
        let roots = self.index.roots.iter();
        roots.filter_map(|n| self.get(&n.id)).collect()
    }

    /// The trail that holds the name `name`.
    pub fn named(&'a self, name: &Name) -> Option<NamedTrail<'a>> {
        let (name, record) = self.index.names.get_key_value(name)?;
        let trail = self.get(record.tag("e")?)?;
        Some(NamedTrail {
            name,
            trail,
            record,
        })
    }

    /// `step` with the name it holds first; `None` when it holds none.
    pub(crate) fn named_of(&'a self, step: Step<'a>) -> Option<NamedTrail<'a>> {
        let record = self.index.entries.get(step.id())?.name?;
        let (name, _) = self
            .index
            .names
            .get_key_value(&Name::parse(&record.content)?)?;
        Some(NamedTrail {
            name,
            trail: step,
            record,
        })
    }

    /// Each named trail, by name.
    pub fn names(&'a self) -> Vec<NamedTrail<'a>> {
        let mut names: Vec<&Name> = self.index.names.keys().collect();
        names.sort_by_key(|name| name.as_str());
        names
            .into_iter()
            .filter_map(|name| self.named(name))
            .collect()
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
    roots: Vec<&'a Note>,
    entries: HashMap<&'a str, Entry<'a>>,
    rules: HashSet<&'a str>,
    names: HashMap<Name, &'a Note>,
}

/// One step: its record, its text with its edits, its links, the steps that replace it, how its marks
/// left it, the drop that counts on it, the record of the first name it holds, and its newest prompt;
/// of its tree, the newest record, whether a step works, whether a step under it is open, and when the
/// last of its steps closed.
pub(crate) struct Entry<'a> {
    pub(crate) note: &'a Note,
    pub(crate) step: StepRecord,
    pub(crate) children: Vec<&'a Note>,
    pub(crate) replaced: Vec<&'a Note>,
    pub(crate) start: Option<&'a Note>,
    pub(crate) close: Option<(&'a Note, Close)>,
    pub(crate) drop: Option<&'a Note>,
    pub(crate) name: Option<&'a Note>,
    pub(crate) touched: u64,
    pub(crate) newest: u64,
    pub(crate) busy: bool,
    pub(crate) loose: bool,
    pub(crate) asked: u64,
    pub(crate) shut: u64,
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
                        drop: None,
                        name: None,
                        touched: note.created_at,
                        newest: 0,
                        busy: false,
                        loose: false,
                        asked: 0,
                        shut: 0,
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
        let mut named = Vec::new();
        for &note in &sorted {
            let Some((id, mark)) = Mark::read(note) else {
                continue;
            };
            let Some(entry) = entries.get_mut(&*id) else {
                continue;
            };
            if let Mark::Name { name } = &mark {
                named.extend(Name::parse(name).map(|name| (name, id.clone(), note)));
            }
            entry.touched = entry.touched.max(note.created_at);
            entry.mark(note, mark);
        }
        let mut names = HashMap::new();
        for (name, id, note) in named {
            let Some(entry) = entries.get_mut(&*id) else {
                continue;
            };
            if let (None, Vacant(free)) = (entry.drop, names.entry(name)) {
                free.insert(note);
                entry.name.get_or_insert(note);
            }
        }
        sum(&mut entries);
        let roots = all.iter().copied();
        let roots = roots.filter(|n| entries[n.id.as_str()].step.under.is_none());
        let rules = sorted.iter().filter(|n| Rule::read(n).is_some());
        let rules = rules.map(|n| n.id.as_str()).collect();
        Index {
            roots: roots.collect(),
            all,
            entries,
            rules,
            names,
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
        let mut shut = entry.close.map_or(0, |(note, _)| note.created_at);
        for child in &entry.children {
            let child = &entries[child.id.as_str()];
            newest = newest.max(child.newest);
            busy |= child.busy;
            loose |= child.loose || child.close.is_none();
            shut = shut.max(child.shut);
        }
        if let Some(entry) = entries.get_mut(id) {
            (entry.newest, entry.busy, entry.loose, entry.shut) = (newest, busy, loose, shut);
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
            Mark::Prompt { .. } => self.asked = self.asked.max(note.created_at),
            Mark::Done(_) => {
                self.close.get_or_insert((note, Close::Done));
            }
            Mark::Drop { .. } if self.close.is_none() => {
                self.close = Some((note, Close::Dropped));
                self.drop = Some(note);
            }
            Mark::Drop { .. } if self.step.under.is_none() => {
                self.drop.get_or_insert(note);
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
