//! The steps of a store, as they stand at one moment.

use crate::{Failure, Grade, Mark, Step};
use celis_wire::Note;

/// The step records and marks of a store at the time `now`: which steps are open, how they nest, and how
/// each went.
pub struct Steps<'a> {
    pub records: &'a [Note],
    pub now: u64,
}

impl<'a> Steps<'a> {
    /// Every step record, oldest first.
    pub fn all(&self) -> Vec<&'a Note> {
        let mut all: Vec<&'a Note> = self
            .records
            .iter()
            .filter(|n| Step::read(n).is_some())
            .collect();
        all.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        all
    }

    /// The step `note` holds, its text from its newest edit; `None` when it is not a step.
    pub fn step(&self, note: &Note) -> Option<Step> {
        let mut step = Step::read(note)?;
        let edit = self
            .records
            .iter()
            .filter_map(|n| match Mark::read(n) {
                Some((id, Mark::Edit { text })) if id == note.id && !text.trim().is_empty() => {
                    Some((n.created_at, &n.id, text))
                }
                _ => None,
            })
            .max_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        if let Some((_, _, text)) = edit {
            step.content = text;
        }
        Some(step)
    }

    /// The step whose id is `id`, or the only record whose id starts with `id` when `id` is 4 or more
    /// characters.
    pub fn find(&self, id: &str) -> Result<&'a Note, Failure> {
        crate::id::find(self.all(), id).ok_or_else(|| Failure::no_step(id))
    }

    /// The first done record with a proof, or drop record with a reason, naming the step `id`; `None`
    /// while it is open.
    pub fn closed(&self, id: &str) -> Option<&'a Note> {
        self.records
            .iter()
            .filter(|n| match Mark::read(n) {
                Some((p, Mark::Done { proof: text, .. }))
                | Some((p, Mark::Drop { reason: text })) => p == id && !text.trim().is_empty(),
                _ => false,
            })
            .min_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)))
    }

    /// The done record that closed the step `id`; `None` while it is open or when it was dropped.
    pub fn done(&self, id: &str) -> Option<&'a Note> {
        self.closed(id)
            .filter(|n| matches!(Mark::read(n), Some((_, Mark::Done { .. }))))
    }

    /// The drop record that closed the step `id`; `None` while it is open or when it was done.
    pub fn dropped(&self, id: &str) -> Option<&'a Note> {
        self.closed(id)
            .filter(|n| matches!(Mark::read(n), Some((_, Mark::Drop { .. }))))
    }

    /// The first start record naming the step `id`; `None` before one.
    pub fn started(&self, id: &str) -> Option<&'a Note> {
        self.records
            .iter()
            .filter(|n| matches!(Mark::read(n), Some((s, Mark::Start)) if s == id))
            .min_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)))
    }

    /// Whether the step `id` is `done`, `dropped`, `working` or `open`.
    pub fn status(&self, id: &str) -> &'static str {
        match (self.done(id), self.dropped(id), self.started(id)) {
            (Some(_), _, _) => "done",
            (None, Some(_), _) => DROPPED,
            (None, None, Some(_)) => "working",
            (None, None, None) => "open",
        }
    }

    /// Every open step whose trail has not ended, oldest first.
    pub fn open(&self) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| self.closed(&n.id).is_none() && !self.ended(self.trail(n)))
            .collect()
    }

    /// Whether the trail `note` is open and ended: 12 hours after the newest record of its tree while a
    /// session's join is live; else when an end is newer than that record, or 30 minutes after it.
    pub fn ended(&self, note: &'a Note) -> bool {
        if self.closed(&note.id).is_some() {
            return false;
        }
        let newest = self.newest(note);
        let quiet = self.now.saturating_sub(newest);
        let live = self.sessions(note).iter().any(|s| self.live(note, s));
        let end = self.ends(note, None).into_iter().any(|at| at >= newest);
        match live {
            true => quiet >= SESSION_LIFE,
            false => end || quiet >= LIFE,
        }
    }

    /// The sessions that joined the trail `note`: the one that started it, and each one with a join.
    pub fn sessions(&self, note: &Note) -> Vec<String> {
        let mut sessions: Vec<String> = Step::read(note)
            .and_then(|s| s.session)
            .into_iter()
            .collect();
        for n in self.records {
            if let Some((id, Mark::Join { session })) = Mark::read(n) {
                if id == note.id && !sessions.contains(&session) {
                    sessions.push(session);
                }
            }
        }
        sessions
    }

    /// The time the session `session` last joined the trail `note`; `None` when it never did.
    pub fn joined(&self, note: &Note, session: &str) -> Option<u64> {
        let started = Step::read(note)
            .filter(|s| s.session.as_deref() == Some(session))
            .map(|_| note.created_at);
        self.records
            .iter()
            .filter(|n| {
                matches!(Mark::read(n), Some((id, Mark::Join { session: s })) if id == note.id && s == session)
            })
            .map(|n| n.created_at)
            .chain(started)
            .max()
    }

    /// Whether the session `session` joined the trail `note` and wrote no end on it since.
    pub fn live(&self, note: &Note, session: &str) -> bool {
        self.joined(note, session).is_some_and(|at| {
            self.ends(note, Some(session))
                .into_iter()
                .all(|end| end < at)
        })
    }

    /// The times of the ends on the trail `note` that end the session `session`, or every end with none.
    /// An end with no session ends them all.
    fn ends(&self, note: &Note, session: Option<&str>) -> Vec<u64> {
        self.records
            .iter()
            .filter(|n| match Mark::read(n) {
                Some((id, Mark::End { session: s })) => {
                    id == note.id && (s.is_none() || session.is_none() || s.as_deref() == session)
                }
                _ => false,
            })
            .map(|n| n.created_at)
            .collect()
    }

    /// The time of the newest record of the tree of `note`: its steps, and the marks that name them, but
    /// an end.
    pub fn newest(&self, note: &'a Note) -> u64 {
        let tree: Vec<&str> = self.tree(note).iter().map(|n| n.id.as_str()).collect();
        self.records
            .iter()
            .filter(|n| {
                tree.contains(&n.id.as_str())
                    || Mark::read(n).is_some_and(|(id, m)| {
                        !matches!(m, Mark::End { .. }) && tree.contains(&id.as_str())
                    })
            })
            .map(|n| n.created_at)
            .max()
            .unwrap_or(note.created_at)
    }

    /// Every trail of the project `project`, oldest first.
    pub fn trails(&self, project: &str) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| {
                Step::read(n).is_some_and(|s| s.under.is_none() && s.anchor.project == project)
            })
            .collect()
    }

    /// The open trail of the session `session` in the project `project`: the trail it joined last that
    /// is not done or dropped.
    pub fn session(&self, project: &str, session: &str) -> Option<&'a Note> {
        self.trails(project)
            .into_iter()
            .filter(|n| self.closed(&n.id).is_none())
            .filter_map(|n| self.joined(n, session).map(|at| (at, n)))
            .max_by_key(|(at, n)| (*at, n.created_at))
            .map(|(_, n)| n)
    }

    /// The steps directly under the step `id`, oldest first.
    pub fn under(&self, id: &str) -> Vec<&'a Note> {
        self.all()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.under.as_deref() == Some(id)))
            .collect()
    }

    /// The trail at the top of the `under` chain of the step `note`.
    pub fn trail(&self, note: &'a Note) -> &'a Note {
        let mut top = note;
        while let Some(up) = Step::read(top)
            .and_then(|s| s.under)
            .and_then(|id| self.find(&id).ok())
        {
            top = up;
        }
        top
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
    pub fn state(&self, note: &'a Note) -> &'static str {
        if self.closed(&note.id).is_none() && !self.ended(note) {
            return ACTIVE;
        }
        let tree = self.tree(note);
        if self.closed(&note.id).is_none() && tree[1..].iter().any(|n| self.closed(&n.id).is_none())
        {
            return EXPIRED;
        }
        if self.dropped(&note.id).is_some() {
            return DROPPED;
        }
        match tree.iter().any(|n| self.steers(&n.id) > 0) {
            true => "reached with help",
            false => "reached alone",
        }
    }

    /// How the work of the done step `note` went; `None` while it is open.
    pub fn grade(&self, note: &Note) -> Option<Grade> {
        let done = self.done(&note.id)?;
        let from = self.started(&note.id).unwrap_or(note).created_at;
        let rework = self
            .all()
            .into_iter()
            .filter(|n| Step::read(n).is_some_and(|s| s.fixes.as_deref() == Some(&note.id)))
            .count();
        Some(Grade {
            took: done.created_at.saturating_sub(from),
            rework,
            steered: self.steers(&note.id),
        })
    }

    /// The steers with content that name the step `id`.
    pub fn steers(&self, id: &str) -> usize {
        self.records
            .iter()
            .filter(|n| matches!(Mark::read(n), Some((s, Mark::Steer { text })) if s == id && !text.trim().is_empty()))
            .count()
    }
}

/// The state of a trail that is still open.
pub const ACTIVE: &str = "active";

/// The state of a dropped step or trail.
pub const DROPPED: &str = "dropped";

/// The state of a trail that ended with an open step under it.
pub const EXPIRED: &str = "expired";

/// The seconds a trail with no live join stays active after the newest record of its tree.
pub const LIFE: u64 = 30 * 60;

/// The seconds a trail stays active after the newest record of its tree while a session's join is live.
pub const SESSION_LIFE: u64 = 12 * 60 * 60;

#[cfg(test)]
mod tests;
