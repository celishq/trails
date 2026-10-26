//! The trails of a store, and what the account writes on them.

pub mod draft;
pub mod notice;
mod target;
pub mod written;

use crate::{Check, Done, Draft, Mark, Notes, Notice, Step, StepRecord, Steps, Trail, Written};
use celis_nostr::Note;
use celis_patterns::{Pattern, PatternStore, Patterns};
use celis_store::records::id::short;
use celis_store::{Refusal, StepId, Store};
use std::collections::HashSet;
use std::path::Path;
use target::Target;

/// The steps and notes of a store, and the account's writes on them, each signed at the store's time.
pub trait TrailStore {
    fn steps(&self) -> Steps<'_>;

    fn notes(&self) -> Notes<'_>;

    /// Writes the step `draft` states, where `parent` puts it; a session's new trail holds its first ask,
    /// which takes the uses. A replaced open step's open tree drops; a replaced done step stays done. An
    /// open step with the same place and words is given back.
    fn add(&self, draft: Draft) -> Result<Written<'_>, Refusal>;

    /// Starts the open step `id`, or gives back the first start of a started step; then the patterns
    /// the step must keep, and the notes of its trail.
    fn start(&self, id: &str) -> Result<Written<'_>, Refusal>;

    /// Closes the open step `id` `done`, once nothing under it is open and its check passes
    /// in `folder`.
    fn done(&self, id: &str, done: Done, folder: &Path) -> Result<Written<'_>, Refusal>;

    /// Drops the open step `id` and each open step under it, the done trail `id`, or the live pattern `id`,
    /// with the reason it is no longer wanted; the step's own drop first.
    fn drop(&self, id: &str, reason: &str) -> Result<Written<'_>, Refusal>;

    /// Gives the step `id` a new text, places or check.
    fn edit(
        &self,
        id: &str,
        text: String,
        places: Vec<String>,
        check: Option<String>,
    ) -> Result<Written<'_>, Refusal>;

    /// Writes `text` as a note on the step `id`.
    fn note(&self, id: &str, text: String) -> Result<Written<'_>, Refusal>;

    /// Writes a hook's prompt `text` as a prompt mark on `trail`; gives back the record.
    fn prompt(&self, trail: Step, text: String) -> Result<Note, Refusal>;
}

impl TrailStore for Store {
    fn steps(&self) -> Steps<'_> {
        Steps::new(self.records(), self.now())
    }

    fn notes(&self) -> Notes<'_> {
        Notes::new(self.records())
    }

    fn add(&self, draft: Draft) -> Result<Written<'_>, Refusal> {
        let steps = self.steps();
        let replaced = draft.replaces.as_deref().map(|id| replaced(&steps, id));
        let replaced = replaced.transpose()?;
        let moved = replaced.filter(|s| s.done_record().is_none());
        let under = parent(&steps, &draft, moved)?;
        let uses = draft
            .uses
            .iter()
            .map(|id| used(self, &steps, &draft.anchor.project, id));
        let mut uses = uses.collect::<Result<Vec<String>, _>>()?;
        let mut seen = std::collections::HashSet::new();
        uses.retain(|id| seen.insert(id.clone()));
        let recalled = recalled(self, &draft, under)?;
        let mut written = Written::default();
        if let Some(same) = same_open(&steps, &draft, under) {
            let linked = !uses.is_empty() || !recalled.is_empty();
            if linked || replaced.is_some_and(|r| r.done_record().is_some()) {
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
            recalled,
            places: draft.places,
            check: draft.check,
            session: draft.session,
            content: draft.content,
        };
        if record.under.is_none() && record.session.is_some() {
            return start_trail(self, record);
        }
        written.notices.extend(ahead(under).map(Notice::NotStarted));
        let note = self.sign(|account, now| record.record(account, now))?;
        let reason = format!("replaced by {}", short(&note.id));
        let ask = under
            .is_some_and(|u| u.is_trail())
            .then(|| StepId::new(&note.id));
        written.records.push(note);
        if let Some(old) = moved {
            written.records.extend(drop_open(self, old, &reason)?);
        }
        written.notices.extend(joined.map(Notice::Joined));
        written.notices.extend(ask.map(Notice::Ask));
        Ok(written)
    }

    fn start(&self, id: &str) -> Result<Written<'_>, Refusal> {
        let steps = self.steps();
        let step = steps.find_open(id)?;
        let mut written = Written::default();
        match step.start() {
            Some(start) => written.records.push(start.clone()),
            None => written
                .records
                .push(self.sign(|account, now| Mark::Start.record(account, step.id(), now))?),
        }
        written.patterns = kept(&self.patterns(), step);
        written.notes = self.notes().of(step.trail());
        Ok(written)
    }

    fn done(&self, id: &str, done: Done, folder: &Path) -> Result<Written<'_>, Refusal> {
        let steps = self.steps();
        let note = steps.find_open(id)?;
        if note.children().iter().any(|s| s.closed().is_none()) {
            return Err(Refusal::OpenSteps(note.id().to_string()));
        }
        let step = note.record();
        if let Some(check) = &step.check {
            Check::new(check, folder, self.limit()).run()?;
        }
        let mut written = Written::default();
        written.records.push(
            self.sign(|account, now| Mark::Done(done.clone()).record(account, note.id(), now))?,
        );
        Ok(written)
    }

    fn drop(&self, id: &str, reason: &str) -> Result<Written<'_>, Refusal> {
        let (steps, patterns) = (self.steps(), self.patterns());
        let pattern = celis_store::records::id::find(patterns.every().iter().map(|e| e.note), id);
        let mark = Mark::Drop {
            reason: reason.to_string(),
        };
        let records = match Target::of(id, pattern, steps.find(id).ok())? {
            Target::Pattern(pattern) if patterns.find(None, &pattern.id).is_ok() => {
                vec![self.sign(|account, now| mark.record(account, &pattern.id, now))?]
            }
            Target::Pattern(pattern) => return Err(Refusal::Closed(pattern.id.clone())),
            Target::Step(step) if step.closed().is_none() => drop_open(self, step, reason)?,
            Target::Step(step) if step.is_trail() && step.drop_mark().is_none() => {
                vec![self.sign(|account, now| mark.record(account, step.id(), now))?]
            }
            Target::Step(step) => return Err(Refusal::Closed(step.id().to_string())),
        };
        Ok(Written {
            records,
            ..Written::default()
        })
    }

    fn edit(
        &self,
        id: &str,
        text: String,
        places: Vec<String>,
        check: Option<String>,
    ) -> Result<Written<'_>, Refusal> {
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

    fn note(&self, id: &str, text: String) -> Result<Written<'_>, Refusal> {
        let steps = self.steps();
        let step = steps.find(id)?;
        let mark = Mark::Note { text };
        let note = self.sign(|account, now| mark.record(account, step.id(), now))?;
        Ok(Written {
            records: vec![note],
            ..Written::default()
        })
    }

    fn prompt(&self, trail: Step, text: String) -> Result<Note, Refusal> {
        let mark = Mark::Prompt { text };
        self.sign(|account, now| mark.record(account, trail.id(), now))
    }
}

/// Writes `ask` in `store` as a new trail with no links, places or check, then `ask` under it; the ask
/// first.
fn start_trail(store: &Store, ask: StepRecord) -> Result<Written<'_>, Refusal> {
    let head = StepRecord {
        replaces: None,
        uses: Vec::new(),
        recalled: Vec::new(),
        places: Vec::new(),
        check: None,
        ..ask.clone()
    };
    let trail = store.sign(|account, now| head.record(account, now))?;
    let ask = StepRecord {
        under: Some(StepId::new(&trail.id)),
        ..ask
    };
    let ask = store.sign(|account, now| ask.record(account, now))?;
    Ok(Written {
        notices: vec![
            Notice::Started {
                ask: StepId::new(&ask.id),
                trail: StepId::new(&trail.id),
            },
            Notice::Ask(StepId::new(&ask.id)),
        ],
        records: vec![ask, trail],
        ..Written::default()
    })
}

/// The whole id of the live pattern of `project` that `id` names, in `store`; refuses a step as a usage
/// error.
fn used(store: &Store, steps: &Steps, project: &str, id: &str) -> Result<String, Refusal> {
    let patterns = store.patterns();
    let found = patterns.find(Some(project), id).ok().map(|p| p.note);
    match Target::of(id, found, steps.find(id).ok())? {
        Target::Pattern(pattern) => Ok(pattern.id.clone()),
        Target::Step(_) => Err(Refusal::Usage(
            "step add --uses takes the id of a pattern".into(),
        )),
    }
}

/// The whole id of each live pattern in scope that `draft` recalls, once each, in `store`; refuses an id
/// that names none with `no-pattern`, and a recall on a step under `under` that is not an ask as a usage
/// error.
fn recalled(store: &Store, draft: &Draft, under: Option<Step>) -> Result<Vec<String>, Refusal> {
    if draft.recalled.is_empty() {
        return Ok(Vec::new());
    }
    let ask = match under {
        Some(under) => under.is_trail(),
        None => draft.session.is_some(),
    };
    if !ask {
        return Err(Refusal::Usage(
            "step add --recalled takes an ask: a step right under a trail".into(),
        ));
    }
    let patterns = store.patterns();
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for id in &draft.recalled {
        let pattern = patterns.find(Some(&draft.anchor.project), id)?;
        if seen.insert(pattern.note.id.clone()) {
            found.push(pattern.note.id.clone());
        }
    }
    Ok(found)
}

/// Drops each open step of the tree of `step` with `reason`, in `store`.
fn drop_open(store: &Store, step: Step, reason: &str) -> Result<Vec<Note>, Refusal> {
    let open = step.tree().into_iter().filter(|s| s.closed().is_none());
    open.map(|s| {
        let mark = Mark::Drop {
            reason: reason.to_string(),
        };
        store.sign(|account, now| mark.record(account, s.id(), now))
    })
    .collect()
}

/// The live patterns of `patterns` the plan of `step` links, each once: each pattern a step on its
/// `under` chain uses.
fn kept<'a>(patterns: &Patterns<'a>, step: Step) -> Vec<Pattern<'a>> {
    let mut found = Vec::new();
    let chain = std::iter::successors(Some(step), |s| s.under());
    for id in chain.flat_map(Step::uses) {
        found.extend(patterns.find(None, id));
    }
    let mut seen = HashSet::new();
    found.retain(|p| seen.insert(p.note.id.clone()));
    found
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
