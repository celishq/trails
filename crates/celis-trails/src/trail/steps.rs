//! The steps of a store, as they stand at one moment.

use crate::{Failure, Grade, Mark, State, Status, Step};
use celis_wire::Note;
use std::collections::HashMap;

/// The step records and marks of a store at the time `now`: which steps are open, how they nest, and how
/// each went.
pub struct Steps<'a> {
    pub now: u64,
    index: Index<'a>,
}

impl<'a> Steps<'a> {
    pub fn new(records: &'a [Note], now: u64) -> Steps<'a> {
        Steps {
            now,
            index: Index::of(records),
        }
    }

    /// Every step record, oldest first.
    pub fn all(&self) -> Vec<&'a Note> {
        self.index.all.clone()
    }

    /// The step `note` holds, with each of its edits, oldest first; `None` when it is not a step.
    pub fn step(&self, note: &Note) -> Option<Step> {
        Some(self.entry(&note.id)?.step.clone())
    }

    /// The step whose id is `id`, or the only record whose id starts with `id` when `id` is 4 or more
    /// characters.
    pub fn find(&self, id: &str) -> Result<&'a Note, Failure> {
        let index = &self.index;
        match index.entries.get(id) {
            Some(entry) => Ok(entry.note),
            None => {
                crate::id::find(index.all.iter().copied(), id).ok_or_else(|| Failure::no_step(id))
            }
        }
    }

    /// The step `find` gives for `id`; `closed` when a done or a drop names it.
    pub fn find_open(&self, id: &str) -> Result<&'a Note, Failure> {
        let note = self.find(id)?;
        match self.closed(&note.id) {
            Some(_) => Err(Failure::closed(&note.id)),
            None => Ok(note),
        }
    }

    /// The first done record with a proof, or drop record with a reason, naming the step `id`; `None`
    /// while it is open.
    pub fn closed(&self, id: &str) -> Option<&'a Note> {
        self.entry(id)?.close.map(|(note, _)| note)
    }

    /// The done record that closed the step `id`; `None` while it is open or when it was dropped.
    pub fn done(&self, id: &str) -> Option<&'a Note> {
        match self.entry(id)?.close {
            Some((note, Status::Done)) => Some(note),
            _ => None,
        }
    }

    /// The drop record that closed the step `id`; `None` while it is open or when it was done.
    pub fn dropped(&self, id: &str) -> Option<&'a Note> {
        match self.entry(id)?.close {
            Some((note, Status::Dropped)) => Some(note),
            _ => None,
        }
    }

    /// The first start record naming the step `id`; `None` before one.
    pub fn started(&self, id: &str) -> Option<&'a Note> {
        self.entry(id)?.start
    }

    /// Whether the step `id` is done, dropped, working or open; expired when it is open and its trail
    /// ended.
    pub fn status(&self, id: &str) -> Status {
        let Some(entry) = self.entry(id) else {
            return Status::Open;
        };
        match (entry.close, entry.start) {
            (Some((_, status)), _) => status,
            _ if self.ended(self.trail(entry.note)) => Status::Expired,
            (None, Some(_)) => Status::Working,
            (None, None) => Status::Open,
        }
    }

    /// Every open step whose trail has not ended, oldest first.
    pub fn open(&self) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| self.closed(&n.id).is_none() && !self.ended(self.trail(n)))
            .collect()
    }

    /// Whether the trail `note` is open and ended: steps sit under it and none is open, or 30 minutes
    /// passed after the newest record of its tree.
    pub fn ended(&self, note: &'a Note) -> bool {
        if self.closed(&note.id).is_some() {
            return false;
        }
        let tree = self.tree(note);
        let idle = tree.len() > 1 && tree[1..].iter().all(|n| self.closed(&n.id).is_some());
        idle || self.now.saturating_sub(self.newest(note)) >= LIFE
    }

    /// The time of the newest record of the tree of `note`: its steps, and the marks that name them.
    pub fn newest(&self, note: &'a Note) -> u64 {
        self.tree(note)
            .iter()
            .filter_map(|n| self.entry(&n.id).map(|e| e.newest))
            .max()
            .unwrap_or(note.created_at)
    }

    /// Every trail of the project `project`, oldest first.
    pub fn trails(&self, project: &str) -> Vec<&'a Note> {
        self.roots()
            .into_iter()
            .filter(|n| {
                self.entry(&n.id)
                    .is_some_and(|e| e.step.anchor.project == project)
            })
            .collect()
    }

    /// Every trail of every project, oldest first.
    pub fn roots(&self) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| self.entry(&n.id).is_some_and(|e| e.step.under.is_none()))
            .collect()
    }

    /// The open trail of the session `session` in the project `project`: the trail of its newest step,
    /// when it is not done or dropped.
    pub fn session(&self, project: &str, session: &str) -> Option<&'a Note> {
        let newest = self.all().into_iter().rev().find(|n| {
            self.entry(&n.id).is_some_and(|e| {
                e.step.anchor.project == project && e.step.session.as_deref() == Some(session)
            })
        })?;
        let trail = self.trail(newest);
        self.closed(&trail.id).is_none().then_some(trail)
    }

    /// The steps directly under the step `id`, oldest first.
    pub fn under(&self, id: &str) -> Vec<&'a Note> {
        self.entry(id).map_or(Vec::new(), |e| e.children.clone())
    }

    /// The trail at the top of the `under` chain of the step `note`.
    pub fn trail(&self, note: &'a Note) -> &'a Note {
        let mut top = note;
        while let Some(up) = self
            .entry(&top.id)
            .and_then(|e| e.step.under.as_deref())
            .and_then(|id| self.find(id).ok())
        {
            top = up;
        }
        top
    }

    /// The ask of the step `note`: the step on its `under` chain right under its trail; `None` for a trail.
    pub fn ask(&self, note: &'a Note) -> Option<&'a Note> {
        let trail = self.trail(note);
        let mut ask = note;
        while let Some(up) = self
            .entry(&ask.id)
            .and_then(|e| e.step.under.as_deref())
            .filter(|up| *up != trail.id)
        {
            ask = self.find(up).ok()?;
        }
        (ask.id != trail.id).then_some(ask)
    }

    /// The past work, newest first: each done ask, and each reached trail with no twin.
    pub fn past(&self) -> Vec<&'a Note> {
        let mut past: Vec<&'a Note> = self
            .all()
            .into_iter()
            .filter(|note| match self.step(note).map(|s| s.under) {
                Some(Some(_)) => self.done(&note.id).is_some() && self.is_ask(note),
                Some(None) => self.state(note).reached() && self.twin(note).is_none(),
                None => false,
            })
            .collect();
        past.reverse();
        past
    }

    /// Whether the step `note` is an ask: a step right under its trail.
    pub fn is_ask(&self, note: &'a Note) -> bool {
        let under = self.entry(&note.id).and_then(|e| e.step.under.as_deref());
        under == Some(self.trail(note).id.as_str())
    }

    /// The first ask under the trail `trail`, when it has the trail's text.
    pub fn twin(&self, trail: &Note) -> Option<&'a Note> {
        let text = &self.entry(&trail.id)?.step.content;
        let first = self.under(&trail.id).into_iter().next()?;
        self.entry(&first.id)
            .is_some_and(|e| e.step.content == *text)
            .then_some(first)
    }

    /// The title of the step `note`; empty when it is not a step.
    pub fn title(&self, note: &Note) -> String {
        self.entry(&note.id)
            .map(|e| e.step.title().to_string())
            .unwrap_or_default()
    }

    /// The step `note` and every step under it, at any depth.
    pub fn tree(&self, note: &'a Note) -> Vec<&'a Note> {
        let mut tree = vec![note];
        let mut i = 0;
        while i < tree.len() {
            tree.extend(self.under(&tree[i].id));
            i += 1;
        }
        tree
    }

    /// The state of the trail `note`: active while open, dropped when dropped, expired when it ended with
    /// an open step, else reached alone or reached with help, as a steer names a step of its tree.
    pub fn state(&self, note: &'a Note) -> State {
        if self.closed(&note.id).is_none() && !self.ended(note) {
            return State::Active;
        }
        let tree = self.tree(note);
        if self.closed(&note.id).is_none() && tree[1..].iter().any(|n| self.closed(&n.id).is_none())
        {
            return State::Expired;
        }
        if self.dropped(&note.id).is_some() {
            return State::Dropped;
        }
        match tree.iter().any(|n| self.steers(&n.id) > 0) {
            true => State::ReachedWithHelp,
            false => State::ReachedAlone,
        }
    }

    /// How the work of the done step `note` went; `None` while it is open.
    pub fn grade(&self, note: &Note) -> Option<Grade> {
        let done = self.done(&note.id)?;
        let entry = self.entry(&note.id)?;
        let from = entry.start.unwrap_or(entry.note).created_at;
        Some(Grade {
            took: done.created_at.saturating_sub(from),
            steered: entry.steers,
        })
    }

    /// The steers with content that name the step `id`.
    pub fn steers(&self, id: &str) -> usize {
        self.entry(id).map_or(0, |e| e.steers)
    }

    fn entry(&self, id: &str) -> Option<&Entry<'a>> {
        self.index.entries.get(id)
    }
}

/// The seconds a trail stays active after the newest record of its tree.
pub const LIFE: u64 = 30 * 60;

/// The step records, oldest first, and what the marks say of each step.
struct Index<'a> {
    all: Vec<&'a Note>,
    entries: HashMap<&'a str, Entry<'a>>,
}

/// One step: its record, its text with its edits, its links and how its marks left it.
struct Entry<'a> {
    note: &'a Note,
    step: Step,
    children: Vec<&'a Note>,
    start: Option<&'a Note>,
    close: Option<(&'a Note, Status)>,
    steers: usize,
    newest: u64,
}

impl<'a> Index<'a> {
    /// The index of `records`, read in the order they were written.
    fn of(records: &'a [Note]) -> Index<'a> {
        let mut sorted: Vec<&'a Note> = records.iter().collect();
        sorted.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        let mut all = Vec::new();
        let mut entries: HashMap<&'a str, Entry<'a>> = HashMap::new();
        for &note in &sorted {
            if let Some(step) = Step::read(note) {
                all.push(note);
                entries.insert(
                    &note.id,
                    Entry {
                        note,
                        step,
                        children: Vec::new(),
                        start: None,
                        close: None,
                        steers: 0,
                        newest: note.created_at,
                    },
                );
            }
        }
        for &note in &all {
            let step = &entries[note.id.as_str()].step;
            let under = step.under.clone();
            if let Some(parent) = under.and_then(|id| entries.get_mut(id.as_str())) {
                parent.children.push(note);
            }
        }
        for &note in &sorted {
            let Some((id, mark)) = Mark::read(note) else {
                continue;
            };
            let Some(entry) = entries.get_mut(id.as_str()) else {
                continue;
            };
            entry.newest = entry.newest.max(note.created_at);
            entry.mark(note, mark);
        }
        Index { all, entries }
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
                self.close.get_or_insert((note, Status::Done));
            }
            Mark::Drop { reason } if !reason.trim().is_empty() => {
                self.close.get_or_insert((note, Status::Dropped));
            }
            Mark::Steer { text } if !text.trim().is_empty() => self.steers += 1,
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
