//! The trails and rules of a store at one moment, and what the account writes on them.

pub mod draft;
pub mod notice;
pub mod written;

use crate::records::id::short;
use crate::{
    Anchor, Check, Draft, Mark, Name, Notice, Refusal, Rule, RuleId, Rules, Scope, Step, StepId,
    StepRecord, Steps, Store, Trail, Written,
};
use celis_nostr::{Account, Note};
use std::path::{Path, PathBuf};

/// The records of a store at the time `now`, and the account's writes on them, each signed now. A
/// step's check runs for `limit` seconds at most. Readers take the time from its steps.
pub struct Trails {
    now: u64,
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

    /// Moves it to `now`, with the records of its store as it last read them and as it wrote them since.
    pub fn at(&mut self, now: u64) {
        self.now = now;
        self.records = self.store.records();
    }

    /// Adds the records of the files `paths` names that it lacks, as `Store::take` reads them; those
    /// records.
    pub fn take(&mut self, paths: &[PathBuf]) -> Vec<Note> {
        let taken = self.store.take(paths);
        self.records.extend(taken.iter().cloned());
        self.records
            .sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        taken
    }

    /// Whether the store holds no record.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn records(&self) -> &[Note] {
        &self.records
    }

    pub fn steps(&self) -> Steps<'_> {
        Steps::new(&self.records, self.now)
    }

    pub fn rules(&self) -> Rules<'_> {
        Rules::new(&self.records)
    }

    /// Writes the step `draft` states, where `parent` puts it; a session's new trail holds its first ask,
    /// which takes the uses. A replaced open step's open tree drops; a replaced done step stays done. An
    /// open step with the same place and words is given back.
    pub fn add(&self, draft: Draft) -> Result<Written, Refusal> {
        let steps = self.steps();
        let replaced = draft.replaces.as_deref().map(|id| replaced(&steps, id));
        let replaced = replaced.transpose()?;
        let moved = replaced.filter(|s| s.done().is_none());
        let under = parent(&steps, &draft, moved)?;
        let trail = under.map(|u| u.trail().id());
        let uses = draft
            .uses
            .iter()
            .map(|id| self.used(&steps, &draft.anchor.project, trail, id));
        let mut uses = uses.collect::<Result<Vec<String>, _>>()?;
        let mut seen = std::collections::HashSet::new();
        uses.retain(|id| seen.insert(id.clone()));
        let mut written = Written::default();
        if let Some(same) = same_open(&steps, &draft, under) {
            if !uses.is_empty() || replaced.is_some_and(|r| r.done().is_some()) {
                return Err(Refusal::Usage(format!(
                    "the open step {} has the same text, and keeps its own links; give the new step other words",
                    short(same.id())
                )));
            }
            written.records.push(same.note().clone());
            return Ok(written);
        }
        let joined = match (&draft.under, under, replaced) {
            (None, Some(trail), None) => Some(trail.step_id()),
            _ => None,
        };
        let record = StepRecord {
            anchor: draft.anchor,
            under: under.map(|u| u.step_id()),
            replaces: replaced.map(|s| s.step_id()),
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

    /// Writes `ask` as a new trail with no links, places or check, then `ask` under it; the ask first.
    fn start_trail(&self, ask: StepRecord) -> Result<Written, Refusal> {
        let head = StepRecord {
            replaces: None,
            uses: Vec::new(),
            places: Vec::new(),
            check: None,
            ..ask.clone()
        };
        let trail = self.sign(|account, now| head.record(account, now))?;
        let ask = StepRecord {
            under: Some(StepId::new(&trail.id)),
            ..ask
        };
        let ask = self.sign(|account, now| ask.record(account, now))?;
        Ok(Written {
            notices: vec![Notice::Started {
                ask: StepId::new(&ask.id),
                trail: StepId::new(&trail.id),
            }],
            records: vec![ask, trail],
            ..Written::default()
        })
    }

    /// Starts the open step `id`, or gives back the first start of a working step; then the rules
    /// the step must keep.
    pub fn start(&self, id: &str) -> Result<Written, Refusal> {
        let steps = self.steps();
        let step = steps.find_open(id)?;
        let mut written = Written::default();
        match step.start() {
            Some(start) => written.records.push(start.clone()),
            None => written
                .records
                .push(self.sign(|account, now| Mark::Start.record(account, step.id(), now))?),
        }
        let kept = self.rules().kept(step).into_iter();
        written.rules = kept.map(|(said, rule)| (said.clone(), rule)).collect();
        Ok(written)
    }

    /// Closes the open step `id` done, with its proof and what changed from its plan, once nothing under
    /// it is open and its check passes in `folder`; a pitfall becomes a rule at its place, else on its
    /// trail.
    pub fn done(
        &self,
        id: &str,
        proof: String,
        changed: Option<String>,
        pitfall: Option<String>,
        folder: &Path,
    ) -> Result<Written, Refusal> {
        let steps = self.steps();
        let note = steps.find_open(id)?;
        if note.children().iter().any(|s| s.closed().is_none()) {
            return Err(Refusal::OpenSteps(note.id().to_string()));
        }
        let step = note.record();
        if let Some(check) = &step.check {
            self.check(check, folder)?;
        }
        let mark = Mark::Done { proof, changed };
        let mut written = Written::default();
        written
            .records
            .push(self.sign(|account, now| mark.record(account, note.id(), now))?);
        if let Some(pitfall) = pitfall {
            let scope = match step.at() {
                Some(at) => Scope::At(at.to_string()),
                None => Scope::Trail(note.trail().id().to_string()),
            };
            let rule = Rule {
                anchor: step.anchor.clone(),
                scope,
                replaces: None,
                content: pitfall,
            };
            written
                .records
                .push(self.sign(|account, now| rule.record(account, now))?);
        }
        Ok(written)
    }

    /// Drops the open step `id` and each open step under it, the done trail `id`, or the live rule `id`,
    /// with the reason it is no longer wanted; the step's own drop first.
    pub fn drop(&self, id: &str, reason: &str) -> Result<Written, Refusal> {
        let (steps, rules) = (self.steps(), self.rules());
        let rule = crate::records::id::find(rules.every().iter().map(|e| e.note), id);
        let mark = Mark::Drop {
            reason: reason.to_string(),
        };
        let records = match (rule, steps.find(id).ok()) {
            (Some(_), Some(_)) => {
                return Err(Refusal::Usage(format!(
                    "{id} starts the id of a rule and of a step; give more of it"
                )))
            }
            (Some(rule), None) if rules.get(&rule.id).is_some() => {
                vec![self.sign(|account, now| mark.record(account, &rule.id, now))?]
            }
            (Some(rule), None) => return Err(Refusal::Closed(rule.id.clone())),
            (None, Some(step)) if step.closed().is_none() => self.drop_open(step, reason)?,
            (None, Some(step))
                if step.is_trail()
                    && step.done().is_some()
                    && step.dropped_after_done().is_none() =>
            {
                vec![self.sign(|account, now| mark.record(account, step.id(), now))?]
            }
            (None, Some(step)) => return Err(Refusal::Closed(step.id().to_string())),
            (None, None) => return Err(Refusal::NoStep(id.to_string())),
        };
        Ok(Written {
            records,
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
    ) -> Result<Written, Refusal> {
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

    /// Names the trail `id` `name`, or gives back the name record when the trail already holds it;
    /// refuses a step that is not a trail, a text that is no name, and a name another trail holds.
    pub fn name(&self, id: &str, name: &str) -> Result<Written, Refusal> {
        let steps = self.steps();
        let trail = steps.find(id)?;
        if !trail.is_trail() {
            return Err(Refusal::Usage(format!(
                "{} is no trail; name its trail {}",
                short(trail.id()),
                short(trail.trail().id())
            )));
        }
        let name = Name::parse(name).ok_or_else(|| {
            Refusal::Usage(format!(
                "\"{name}\" is no name: a lowercase letter, then lowercase letters, digits and -"
            ))
        })?;
        let record = match steps.named(&name) {
            Some(held) if held.trail.id() == trail.id() => held.record.clone(),
            Some(held) => {
                return Err(Refusal::Usage(format!(
                    "the trail {} holds the name {}",
                    short(held.trail.id()),
                    name.as_str()
                )))
            }
            None => {
                let mark = Mark::Name {
                    name: name.as_str().to_string(),
                };
                self.sign(|account, now| mark.record(account, trail.id(), now))?
            }
        };
        Ok(Written {
            records: vec![record],
            ..Written::default()
        })
    }

    /// Writes `rule`, its trail scope taken whole from a trail of its project that is not done or
    /// dropped, replacing the live rule there, or of a named trail, that `replaces` names; tells each
    /// override that the new rule ends.
    pub fn decide(&self, mut rule: Rule, replaces: Option<&str>) -> Result<Written, Refusal> {
        let project = rule.anchor.project.clone();
        if let Scope::Trail(id) = &rule.scope {
            let steps = self.steps();
            let trail = steps
                .find(id)
                .map_err(|_| Refusal::NoTrail(id.to_string()))?;
            let open = trail.closed().is_none()
                && trail.is_trail()
                && trail.record().anchor.project == project;
            if !open {
                return Err(Refusal::NoTrail(id.to_string()));
            }
            rule.scope = Scope::Trail(trail.id().to_string());
        }
        let mut notices = Vec::new();
        if let Some(id) = replaces {
            let rules = self.rules();
            let found = rules.replaceable(&self.steps(), &project, id)?;
            rule.replaces = Some(RuleId::new(&found.id));
            if Anchor::read(found).is_some_and(|a| a.project == project) {
                let ended = rules.overrides(&found.id).into_iter();
                notices.extend(ended.map(|(note, old)| Notice::Ended {
                    rule: RuleId::new(&note.id),
                    project: old.anchor.name,
                }));
            }
        }
        let written = self.sign(|account, now| rule.record(account, now))?;
        Ok(Written {
            records: vec![written],
            notices,
            ..Written::default()
        })
    }

    /// Writes a hook's prompt `text` as a prompt mark on `trail`; gives back the record.
    pub fn prompt(&self, trail: Step, text: String) -> Result<Note, Refusal> {
        let mark = Mark::Prompt { text };
        self.sign(|account, now| mark.record(account, trail.id(), now))
    }

    /// The whole id of the trail named `id`, else of the live rule of `project` or the step of a trail
    /// other than `trail` that `id` names; a trail gives the step that holds its work (`Trail::work`).
    fn used(
        &self,
        steps: &Steps,
        project: &str,
        trail: Option<&str>,
        id: &str,
    ) -> Result<String, Refusal> {
        let named = Name::parse(id).and_then(|name| steps.named(&name));
        let rule = self.rules().find(project, id).ok();
        let step = steps.find(id).ok();
        let step = match (named, rule, step) {
            (Some(named), ..) => Trail::new(named.trail).work(),
            (None, Some(_), Some(_)) => {
                return Err(Refusal::Usage(format!(
                    "{id} starts the id of a rule and of a step; give more of it"
                )))
            }
            (None, Some(rule), None) => return Ok(rule.id.clone()),
            (None, None, Some(step)) => Trail::new(step).work(),
            (None, None, None) => return Err(Refusal::NoStep(id.to_string())),
        };
        if Some(step.trail().id()) == trail {
            return Err(Refusal::Usage(
                "step add --uses takes a step of another trail".into(),
            ));
        }
        Ok(step.id().to_string())
    }

    /// Drops each open step of the tree of `step` with `reason`.
    fn drop_open(&self, step: Step, reason: &str) -> Result<Vec<Note>, Refusal> {
        let open = step.tree().into_iter().filter(|s| s.closed().is_none());
        open.map(|s| {
            let mark = Mark::Drop {
                reason: reason.to_string(),
            };
            self.sign(|account, now| mark.record(account, s.id(), now))
        })
        .collect()
    }

    fn check(&self, check: &str, folder: &Path) -> Result<(), Refusal> {
        Check::new(check, folder, self.limit).run()
    }

    /// Writes the record `record` gives for the account and now, and returns it.
    fn sign(
        &self,
        record: impl FnOnce(&Account, u64) -> anyhow::Result<Note>,
    ) -> Result<Note, Refusal> {
        let note = record(self.store.account()?, self.now)?;
        self.store.write(&note)?;
        Ok(note)
    }
}

/// The step `id` names for `--replaces`, a trail as its first ask: open, or done; refuses a trail with no
/// first ask as a usage error and a dropped step with `closed`.
fn replaced<'a>(steps: &'a Steps<'a>, id: &str) -> Result<Step<'a>, Refusal> {
    let step = Trail::new(steps.find(id)?).work();
    if step.is_trail() {
        return Err(Refusal::Usage(
            "step add --replaces takes a step under a trail; edit a trail's title instead".into(),
        ));
    }
    match step.dropped() {
        Some(_) => Err(Refusal::Closed(step.id().to_string())),
        None => Ok(step),
    }
}

/// Where `draft` goes: under its `--under`, else under the replaced open step `moved`'s `under`, else under
/// the session's open trail unless `--new`; none starts a trail. The step it goes under must be open.
fn parent<'a>(
    steps: &'a Steps<'a>,
    draft: &Draft,
    moved: Option<Step<'a>>,
) -> Result<Option<Step<'a>>, Refusal> {
    let named = draft
        .under
        .clone()
        .or(moved.and_then(|m| m.record().under.as_ref().map(|u| u.to_string())));
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
    steps.all().into_iter().find(|s| {
        let record = s.record();
        s.closed().is_none()
            && record.anchor.project == draft.anchor.project
            && record.under.as_deref() == under
            && s.note().content == draft.content
    })
}

/// The first open step under `under` that has not started, when `under` is below an ask.
fn ahead(under: Option<Step>) -> Option<StepId> {
    let parent = under.filter(|p| p.ask().is_some_and(|a| a.id() != p.id()))?;
    let children = parent.children().into_iter();
    let open = children
        .into_iter()
        .find(|s| s.closed().is_none() && s.start().is_none())?;
    Some(open.step_id())
}
