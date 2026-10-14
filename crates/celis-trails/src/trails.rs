//! The trails and decisions of a store at one moment, and what the account writes on them.

pub mod draft;
pub mod notice;
pub mod review;
pub mod written;

use crate::id::short;
use crate::{
    Check, Decision, Decisions, Draft, Failure, Mark, Notice, Review, Scope, State, Step,
    StepRecord, Steps, Store, Written,
};
use celis_nostr::{Account, Note};
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

    /// Writes the step `draft` states, where `place` puts it; a session's new trail holds its first ask,
    /// which takes the uses. A replaced open step's open tree drops; a replaced done step stays done. An
    /// open step with the same place and words is given back.
    pub fn add(&self, draft: Draft) -> Result<Written, Failure> {
        let steps = self.steps();
        let replaced = draft.replaces.as_deref().map(|id| replaced(&steps, id));
        let replaced = replaced.transpose()?;
        let moved = replaced.filter(|s| s.done().is_none());
        let under = place(&steps, &draft, moved)?;
        let trail = under.map(|u| u.trail().id());
        let uses = draft
            .uses
            .iter()
            .map(|id| self.used(&steps, &draft.anchor.project, trail, id));
        let uses = uses.collect::<Result<Vec<_>, _>>()?;
        let mut written = Written::default();
        if let Some(same) = same_open(&steps, &draft, under) {
            written.records.push(same.note().clone());
            return Ok(written);
        }
        let joined = match (&draft.under, under, moved) {
            (None, Some(trail), None) => Some(trail.id().to_string()),
            _ => None,
        };
        let record = StepRecord {
            anchor: draft.anchor,
            under: under.map(|u| u.id().to_string()),
            replaces: replaced.map(|s| s.id().to_string()),
            uses,
            places: draft.places,
            check: draft.check,
            session: draft.session,
            content: draft.content,
        };
        if record.under.is_none() && record.session.is_some() {
            return self.start_trail(record);
        }
        written.notices.extend(ahead(under).map(Notice::NotStarted));
        let note = self.sign(|account, now| record.record(account, now))?;
        let reason = format!("replaced by {}", short(&note.id));
        written.records.push(note);
        if let Some(old) = moved {
            written.records.extend(self.drop_open(old, &reason)?);
        }
        written.notices.extend(joined.map(Notice::Joined));
        Ok(written)
    }

    /// Writes `ask` as a new trail with no places, check or uses, then `ask` under it; the ask first.
    fn start_trail(&self, ask: StepRecord) -> Result<Written, Failure> {
        let head = StepRecord {
            uses: Vec::new(),
            places: Vec::new(),
            check: None,
            ..ask.clone()
        };
        let trail = self.sign(|account, now| head.record(account, now))?;
        let ask = StepRecord {
            under: Some(trail.id.clone()),
            ..ask
        };
        let ask = self.sign(|account, now| ask.record(account, now))?;
        Ok(Written {
            records: vec![ask, trail],
            ..Written::default()
        })
    }

    /// Starts the open step `id`, or gives back the first start of a working step; then the live
    /// decisions at the step's places.
    pub fn start(&self, id: &str) -> Result<Written, Failure> {
        let steps = self.steps();
        let step = steps.find_open(id)?;
        let mut written = Written::default();
        match step.start() {
            Some(start) => written.records.push(start.clone()),
            None => written
                .records
                .push(self.sign(|account, now| Mark::Start.record(account, step.id(), now))?),
        }
        let step = step.record();
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
        if note.children().iter().any(|s| s.closed().is_none()) {
            return Err(Failure::open_steps(note.id()));
        }
        let step = note.record();
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
            .push(self.sign(|account, now| mark.record(account, note.id(), now))?);
        if let Some(pitfall) = pitfall {
            let scope = match step.at() {
                Some(at) => Scope::At(at.to_string()),
                None => Scope::Trail(note.trail().id().to_string()),
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
        let review = self.review(note, changed.as_deref());
        written.notices.push(Notice::Review(review));
        Ok(written)
    }

    /// Drops the open step `id`, and each open step under it, with the reason its work is no longer
    /// wanted; the step's own drop first.
    pub fn drop(&self, id: &str, reason: &str) -> Result<Written, Failure> {
        let steps = self.steps();
        let step = steps.find_open(id)?;
        Ok(Written {
            records: self.drop_open(step, reason)?,
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
        let step = steps.find(id)?;
        let mark = Mark::Edit {
            text,
            places,
            check,
        };
        let edit = self.sign(|account, now| mark.record(account, step.id(), now))?;
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
            let trail = steps.find(id).map_err(|_| Failure::no_trail(id))?;
            let open = trail.closed().is_none()
                && trail.is_trail()
                && trail.record().anchor.project == project;
            if !open {
                return Err(Failure::no_trail(id));
            }
            decision.scope = Scope::Trail(trail.id().to_string());
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

    /// Writes a hook's prompt `text` as a prompt mark on `trail`; gives back the record.
    pub fn prompt(&self, trail: Step, text: String) -> Result<Note, Failure> {
        let mark = Mark::Prompt { text };
        self.sign(|account, now| mark.record(account, trail.id(), now))
    }

    /// The whole id of the live decision of `project`, or of the step of a trail other than `trail`, that
    /// `id` names; a trail gives its first ask.
    fn used(
        &self,
        steps: &Steps,
        project: &str,
        trail: Option<&str>,
        id: &str,
    ) -> Result<String, Failure> {
        if let Ok(decision) = self.decisions().find(project, id) {
            return Ok(decision.id.clone());
        }
        let step = steps.find(id)?;
        let step = match step.is_trail() {
            true => step.children().first().copied().unwrap_or(step),
            false => step,
        };
        if Some(step.trail().id()) == trail {
            return Err(Failure::usage(
                "step add --uses takes a step of another trail",
            ));
        }
        Ok(step.id().to_string())
    }

    /// Drops each open step of the tree of `step` with `reason`.
    fn drop_open(&self, step: Step, reason: &str) -> Result<Vec<Note>, Failure> {
        let open = step.tree().into_iter().filter(|s| s.closed().is_none());
        open.map(|s| {
            let mark = Mark::Drop {
                reason: reason.to_string(),
            };
            self.sign(|account, now| mark.record(account, s.id(), now))
        })
        .collect()
    }

    /// How `step`, just done with `changed`, went against its plan.
    fn review(&self, step: Step, changed: Option<&str>) -> Review {
        let tree = step.tree();
        let parts = &tree[1..];
        let began = parts
            .iter()
            .filter_map(|s| s.start())
            .map(|s| s.created_at)
            .min()
            .unwrap_or(u64::MAX);
        let line = |s: &Step| (s.id().to_string(), s.title().to_string());
        let done = parts.iter().filter_map(|s| {
            let changed = s.done()?.tag("changed")?;
            Some((s.id().to_string(), changed.to_string()))
        });
        let own = changed.map(|c| (step.id().to_string(), c.to_string()));
        let trail = step.trail();
        let since = step.note().created_at;
        let asked = self.records.iter().filter_map(|record| {
            let (on, Mark::Prompt { text }) = Mark::read(record)? else {
                return None;
            };
            (on == trail.id() && record.created_at >= since).then_some(text)
        });
        let expect = step
            .record()
            .detail()
            .find_map(|l| l.trim().strip_prefix("expect:"))
            .map(|e| e.trim().to_string());
        let under = step.under();
        let beside = under.map_or(Vec::new(), |u| u.children());
        Review {
            step: step.id().to_string(),
            ask: step.is_ask(),
            expect,
            as_planned: changed.is_none(),
            planned: parts
                .iter()
                .filter(|s| s.note().created_at <= began)
                .count(),
            added: parts
                .iter()
                .filter(|s| s.note().created_at > began)
                .map(line)
                .collect(),
            dropped: parts
                .iter()
                .filter_map(|s| {
                    let drop = s.dropped()?;
                    Some((
                        s.id().to_string(),
                        s.title().to_string(),
                        drop.content.clone(),
                    ))
                })
                .collect(),
            changed: done.chain(own).collect(),
            asked: asked.collect(),
            open: beside
                .iter()
                .filter(|s| s.id() != step.id() && s.closed().is_none())
                .map(line)
                .collect(),
            under: under.map(|u| u.id().to_string()),
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

/// The step `id` names for `--replaces`: open, or done; refuses a trail as a usage error and a dropped step
/// with `closed`.
fn replaced<'a>(steps: &'a Steps<'a>, id: &str) -> Result<Step<'a>, Failure> {
    let step = steps.find(id)?;
    if step.is_trail() {
        return Err(Failure::usage(
            "step add --replaces takes a step under a trail; edit a trail's title instead",
        ));
    }
    match step.dropped() {
        Some(_) => Err(Failure::closed(step.id())),
        None => Ok(step),
    }
}

/// Where `draft` goes: under its `--under`, else under the replaced open step `moved`'s `under`, else under
/// the session's open trail unless `--new`; none starts a trail. The step it goes under must be open.
fn place<'a>(
    steps: &'a Steps<'a>,
    draft: &Draft,
    moved: Option<Step<'a>>,
) -> Result<Option<Step<'a>>, Failure> {
    let named = draft
        .under
        .clone()
        .or(moved.and_then(|m| m.record().under.clone()));
    if let Some(id) = named {
        return steps.find_open(&id).map(Some);
    }
    Ok(match (&draft.session, draft.new) {
        (Some(session), false) => steps.session(&draft.anchor.project, session),
        _ => None,
    })
}

/// The open step of the project of `draft`, under `under`, with the same words.
fn same_open<'a>(steps: &'a Steps<'a>, draft: &Draft, under: Option<Step<'a>>) -> Option<Step<'a>> {
    let under = under.map(|u| u.id());
    steps.open().into_iter().find(|s| {
        let record = s.record();
        record.anchor.project == draft.anchor.project
            && record.under.as_deref() == under
            && s.note().content == draft.content
    })
}

/// The first open step under `under` that has not started, when `under` is below an ask.
fn ahead(under: Option<Step>) -> Option<String> {
    let parent = under.filter(|p| p.ask().is_some_and(|a| a.id() != p.id()))?;
    let children = parent.children().into_iter();
    let open = children.into_iter().find(|s| s.state() == State::Open)?;
    Some(open.id().to_string())
}
