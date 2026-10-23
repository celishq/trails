//! How a done step went against its plan.

use super::Gap;
use celis_nostr::Note;
use celis_trails::{Done, Mark, ProofFile, Step, StepId};

/// How a done step went against its plan: its expect line and its own changed, the steps under it planned
/// before the first started, added after, dropped with their reasons and changed, the person's prompts on
/// its tree, and the open steps beside it under `under`, each as an id and a title; the files of its
/// proof, and when no step beside it is open, those of the done steps beside it; on an ask, its gap.
pub struct Review {
    pub step: StepId,
    pub under: Option<StepId>,
    pub ask: bool,
    pub expect: Option<String>,
    pub as_planned: bool,
    pub planned: usize,
    pub added: Vec<(StepId, String)>,
    pub dropped: Vec<(StepId, String, String)>,
    pub changed: Vec<(StepId, String)>,
    pub asked: Vec<String>,
    pub open: Vec<(StepId, String)>,
    pub files: Vec<ProofFile>,
    pub beside: Vec<(StepId, String, ProofFile)>,
    pub gap: Option<Gap>,
}

impl Review {
    /// How `step`, just closed `done` at `now`, went against its plan; `records` are the store's before
    /// the done.
    pub fn of(records: &[Note], step: Step, done: &Done, now: u64) -> Review {
        let changed = done.changed.as_deref();
        let tree = step.tree();
        let parts = &tree[1..];
        let began = parts
            .iter()
            .filter_map(|s| s.start())
            .map(|s| s.created_at)
            .min()
            .unwrap_or(u64::MAX);
        let line = |s: &Step| (s.step_id(), s.title().to_string());
        let turns = parts.iter().filter_map(|s| {
            let changed = s.done()?.tag("changed")?;
            Some((s.step_id(), changed.to_string()))
        });
        let own = changed.map(|c| (step.step_id(), c.to_string()));
        let trail = step.trail();
        let since = step.note().created_at;
        let ask = step.is_ask();
        let asked = records.iter().filter_map(|record| {
            let (on, Mark::Prompt { text }) = Mark::read(record)? else {
                return None;
            };
            (ask && on == trail.id() && record.created_at >= since).then_some(text)
        });
        let expect = step
            .record()
            .detail()
            .find_map(|l| l.trim().strip_prefix("expect:"))
            .map(|e| e.trim().to_string());
        let under = step.under();
        let beside = under.map_or(Vec::new(), |u| u.children());
        let others: Vec<Step> = beside.into_iter().filter(|s| s.id() != step.id()).collect();
        let open: Vec<Step> = others
            .iter()
            .copied()
            .filter(|s| s.closed().is_none())
            .collect();
        Review {
            step: step.step_id(),
            ask,
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
                    Some((s.step_id(), s.title().to_string(), drop.content.clone()))
                })
                .collect(),
            changed: turns.chain(own).collect(),
            asked: asked.collect(),
            open: open.iter().map(line).collect(),
            beside: match open.is_empty() {
                true => others
                    .iter()
                    .filter_map(|s| Some((s, s.closing()?.files)))
                    .flat_map(|(s, files)| {
                        files
                            .into_iter()
                            .map(move |f| (s.step_id(), s.title().to_string(), f))
                    })
                    .collect(),
                false => Vec::new(),
            },
            files: done.files.clone(),
            under: under.map(|u| u.step_id()),
            gap: ask.then(|| Gap::of(records, step, done, now)),
        }
    }
}
