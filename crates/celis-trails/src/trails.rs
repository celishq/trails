//! The trails and decisions of a store at one moment, and what the account writes on them.

pub mod draft;
pub mod notice;
pub mod review;
pub mod written;

use crate::id::short;
use crate::{
    Check, Decision, Decisions, Draft, Failure, Mark, Notice, Review, Scope, Status, Step, Steps,
    Store, Written,
};
use celis_wire::{Account, Note};
use std::path::Path;

/// The records of a store at the time `now`, and the account's writes on them, each signed now. A
/// step's check runs for `limit` seconds at most.
pub struct Trails {
    pub now: u64,
    store: Store,
    records: Vec<Note>,
    limit: u64,
}

impl Trails {
    /// The records of `store` at `now`.
    pub fn open(store: Store, now: u64, limit: u64) -> Trails {
        Trails {
            records: store.records(),
            store,
            now,
            limit,
        }
    }

    pub fn steps(&self) -> Steps<'_> {
        Steps::new(&self.records, self.now)
    }

    pub fn decisions(&self) -> Decisions<'_> {
        Decisions::new(&self.records)
    }

    /// Writes the step `draft` states. With no `under`, it goes in the replaced step's place, or under
    /// the session's trail, else starts a trail; a session's new trail holds its first
    /// ask. A replaced step's open tree drops. An open step with the same place and words is given back.
    pub fn add(&self, draft: Draft) -> Result<Written, Failure> {
        let steps = self.steps();
        let replaced = self.resolve(&steps, draft.replaces.as_deref(), true)?;
        let place =
            match replaced.and_then(|n| steps.step(n)) {
                Some(old) if old.under.is_none() => return Err(Failure::usage(
                    "step add --replaces takes a step under a trail; edit a trail's title instead",
                )),
                Some(old) => old.under,
                None => None,
            };
        let named = match (&draft.under, &place) {
            (None, Some(place)) => Some(place),
            (under, _) => under.as_ref(),
        };
        let under = self.resolve(&steps, named.map(String::as_str), true)?;
        let under = match (under, &draft.session) {
            (None, Some(session)) if !draft.new => steps.session(&draft.anchor.project, session),
            (under, _) => under,
        };
        let under = under.map(|n| n.id.clone());
        let joined = match (&draft.under, &under, replaced) {
            (None, Some(trail), None) => Some(trail.clone()),
            _ => None,
        };
        let open = steps.open().into_iter().find(|n| {
            Step::read(n)
                .is_some_and(|p| p.anchor.project == draft.anchor.project && p.under == under)
                && n.content == draft.content
        });
        let mut written = Written::default();
        if let Some(note) = open {
            written.records.push(note.clone());
            return Ok(written);
        }
        let step = Step {
            anchor: draft.anchor,
            under,
            replaces: replaced.map(|n| n.id.clone()),
            places: draft.places,
            check: draft.check,
            session: draft.session,
            content: draft.content,
        };
        if step.under.is_none() && step.session.is_some() {
            let head = Step {
                places: Vec::new(),
                check: None,
                ..step.clone()
            };
            let trail = self.sign(|account, now| head.record(account, now))?;
            let ask = Step {
                under: Some(trail.id.clone()),
                ..step
            };
            written
                .records
                .push(self.sign(|account, now| ask.record(account, now))?);
            written.records.push(trail);
            return Ok(written);
        }
        let parent = step.under.as_deref().and_then(|u| steps.find(u).ok());
        let ahead = parent
            .filter(|p| steps.ask(p).is_some_and(|a| a.id != p.id))
            .and_then(|p| {
                steps
                    .under(&p.id)
                    .into_iter()
                    .find(|n| steps.status(&n.id) == Status::Open)
            });
        written
            .notices
            .extend(ahead.map(|n| Notice::NotStarted(n.id.clone())));
        let note = self.sign(|account, now| step.record(account, now))?;
        let reason = format!("replaced by {}", short(&note.id));
        written.records.push(note);
        if let Some(old) = replaced {
            written
                .records
                .extend(self.drop_open(&steps, old, &reason)?);
        }
        written.notices.extend(joined.map(Notice::Joined));
        Ok(written)
    }

    /// Starts the open step `id`, or gives back the first start of a working step; then the live
    /// decisions at the step's places.
    pub fn start(&self, id: &str) -> Result<Written, Failure> {
        let steps = self.steps();
        let note = steps.find_open(id)?;
        let step = steps.step(note).ok_or_else(|| Failure::no_step(id))?;
        let mut written = Written::default();
        match steps.started(&note.id) {
            Some(start) => written.records.push(start.clone()),
            None => written
                .records
                .push(self.sign(|account, now| Mark::Start.record(account, &note.id, now))?),
        }
        let decisions = self.decisions();
        for place in &step.places {
            for (said, decision) in decisions.at(&step.anchor.project, place) {
                if !written.decisions.iter().any(|(n, _)| n.id == said.id) {
                    written.decisions.push((said.clone(), decision));
                }
            }
        }
        Ok(written)
    }

    /// Closes the open step `id` done, with its proof and what changed from its plan, once nothing under
    /// it is open and its check passes in `folder`; a pitfall becomes a decision at its place, else on its
    /// trail. It gives the step's review.
    pub fn done(
        &self,
        id: &str,
        proof: String,
        changed: Option<String>,
        pitfall: Option<String>,
        folder: &Path,
    ) -> Result<Written, Failure> {
        let steps = self.steps();
        let note = steps.find_open(id)?;
        if steps
            .under(&note.id)
            .iter()
            .any(|n| steps.closed(&n.id).is_none())
        {
            return Err(Failure::open_steps(&note.id));
        }
        let step = steps.step(note).ok_or_else(|| Failure::no_step(id))?;
        if let Some(check) = &step.check {
            self.check(check, folder)?;
        }
        let mark = Mark::Done {
            proof,
            changed: changed.clone(),
        };
        let mut written = Written::default();
        written
            .records
            .push(self.sign(|account, now| mark.record(account, &note.id, now))?);
        let trail = steps.trail(note);
        if let Some(pitfall) = pitfall {
            let scope = match step.at() {
                Some(at) => Scope::At(at.to_string()),
                None => Scope::Trail(trail.id.clone()),
            };
            let decision = Decision {
                anchor: step.anchor.clone(),
                scope,
                replaces: None,
                content: pitfall,
            };
            written
                .records
                .push(self.sign(|account, now| decision.record(account, now))?);
        }
        let review = self.review(&steps, note, changed.as_deref());
        written.notices.push(Notice::Review(review));
        Ok(written)
    }

    /// Drops the open step `id`, and each open step under it, with the reason its work is no longer
    /// wanted; the step's own drop first.
    pub fn drop(&self, id: &str, reason: &str) -> Result<Written, Failure> {
        let steps = self.steps();
        let note = steps.find_open(id)?;
        Ok(Written {
            records: self.drop_open(&steps, note, reason)?,
            ..Written::default()
        })
    }

    /// Gives the step `id` a new text, places or check.
    pub fn edit(
        &self,
        id: &str,
        text: String,
        places: Vec<String>,
        check: Option<String>,
    ) -> Result<Written, Failure> {
        let steps = self.steps();
        let note = steps.find(id)?;
        let mark = Mark::Edit {
            text,
            places,
            check,
        };
        let edit = self.sign(|account, now| mark.record(account, &note.id, now))?;
        Ok(Written {
            records: vec![edit],
            ..Written::default()
        })
    }

    /// Writes `decision`, its trail scope taken whole from a trail of its project that is not done or
    /// dropped, and its
    /// `replaces` from a live decision there.
    pub fn decide(&self, mut decision: Decision) -> Result<Written, Failure> {
        let project = decision.anchor.project.clone();
        if let Scope::Trail(id) = &decision.scope {
            let steps = self.steps();
            let note = steps.find(id).map_err(|_| Failure::no_trail(id))?;
            let open = steps.closed(&note.id).is_none()
                && Step::read(note)
                    .is_some_and(|s| s.under.is_none() && s.anchor.project == project);
            if !open {
                return Err(Failure::no_trail(id));
            }
            decision.scope = Scope::Trail(note.id.clone());
        }
        if let Some(id) = &decision.replaces {
            decision.replaces = Some(self.decisions().find(&project, id)?.id.clone());
        }
        let written = self.sign(|account, now| decision.record(account, now))?;
        Ok(Written {
            records: vec![written],
            ..Written::default()
        })
    }

    /// Writes a hook's prompt `text` as a steer on the oldest working step of `trail`, else as a prompt
    /// mark on the trail; gives back the record and the working step.
    pub fn prompt<'a>(
        &self,
        steps: &Steps<'a>,
        trail: &'a Note,
        text: String,
    ) -> Result<(Note, Option<&'a Note>), Failure> {
        let working = steps
            .tree(trail)
            .into_iter()
            .skip(1)
            .find(|n| steps.status(&n.id) == Status::Working);
        let (mark, on) = match working {
            Some(step) => (Mark::Steer { text }, step),
            None => (Mark::Prompt { text }, trail),
        };
        let record = self.sign(|account, now| mark.record(account, &on.id, now))?;
        Ok((record, working))
    }

    /// The note whose id is `id` or starts with it, open when `open`; none when `id` is none.
    fn resolve<'a>(
        &self,
        steps: &Steps<'a>,
        id: Option<&str>,
        open: bool,
    ) -> Result<Option<&'a Note>, Failure> {
        id.map(|id| match open {
            true => steps.find_open(id),
            false => steps.find(id),
        })
        .transpose()
    }

    /// Drops each open step of the tree of `note` with `reason`.
    fn drop_open(&self, steps: &Steps, note: &Note, reason: &str) -> Result<Vec<Note>, Failure> {
        let open = steps.tree(note);
        let open = open.into_iter().filter(|n| steps.closed(&n.id).is_none());
        open.map(|n| {
            let mark = Mark::Drop {
                reason: reason.to_string(),
            };
            self.sign(|account, now| mark.record(account, &n.id, now))
        })
        .collect()
    }

    /// How `note`, just done with `changed`, went against its plan.
    fn review(&self, steps: &Steps, note: &Note, changed: Option<&str>) -> Review {
        let tree = steps.tree(note);
        let parts = &tree[1..];
        let began = parts
            .iter()
            .filter_map(|n| steps.started(&n.id))
            .map(|s| s.created_at)
            .min()
            .unwrap_or(u64::MAX);
        let line = |n: &Note| (n.id.clone(), steps.title(n));
        let done = parts
            .iter()
            .filter_map(|n| Some((n.id.clone(), steps.done(&n.id)?.tag("changed")?.to_string())));
        let own = changed.map(|c| (note.id.clone(), c.to_string()));
        let ask = steps.is_ask(note);
        let trail = steps.trail(note);
        let asked = self.records.iter().filter_map(|record| {
            let (on, Mark::Prompt { text } | Mark::Steer { text }) = Mark::read(record)? else {
                return None;
            };
            let during = ask && on == trail.id && record.created_at >= note.created_at;
            (during || tree.iter().any(|n| n.id == on)).then_some(text)
        });
        let step = steps.step(note);
        let under = step.as_ref().and_then(|s| s.under.clone());
        let expect = step.as_ref().and_then(|s| {
            s.detail()
                .find_map(|l| l.trim().strip_prefix("expect:"))
                .map(|e| e.trim().to_string())
        });
        let open = under.as_deref().map_or(Vec::new(), |u| steps.under(u));
        Review {
            step: note.id.clone(),
            ask,
            expect,
            as_planned: changed.is_none(),
            planned: parts.iter().filter(|n| n.created_at <= began).count(),
            added: parts
                .iter()
                .filter(|n| n.created_at > began)
                .map(|n| line(n))
                .collect(),
            dropped: parts
                .iter()
                .filter_map(|n| {
                    let drop = steps.dropped(&n.id)?;
                    Some((n.id.clone(), steps.title(n), drop.content.clone()))
                })
                .collect(),
            changed: done.chain(own).collect(),
            asked: asked.collect(),
            open: open
                .into_iter()
                .filter(|n| n.id != note.id && steps.closed(&n.id).is_none())
                .map(line)
                .collect(),
            under,
        }
    }

    fn check(&self, check: &str, folder: &Path) -> Result<(), Failure> {
        Check::new(check, folder, self.limit).run()
    }

    /// Writes the record `record` gives for the account and now, and returns it.
    fn sign(
        &self,
        record: impl FnOnce(&Account, u64) -> anyhow::Result<Note>,
    ) -> Result<Note, Failure> {
        let note = record(self.store.account()?, self.now)?;
        self.store.write(&note)?;
        Ok(note)
    }
}
