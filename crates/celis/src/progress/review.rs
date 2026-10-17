//! How a done step went against its plan.

use celis_nostr::Note;
use celis_trails::{Mark, Step, StepId};

/// How a done step went against its plan: its expect line and its own changed, the steps under it planned
/// before the first started, added after, dropped with their reasons and changed, the person's prompts on
/// its tree, and the open steps beside it under `under`, each as an id and a title.
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
}

impl Review {
    /// How `step`, just done with `changed`, went against its plan; `records` are the store's before
    /// the done.
    pub fn of(records: &[Note], step: Step, changed: Option<&str>) -> Review {
        let tree = step.tree();
        let parts = &tree[1..];
        let began = parts
            .iter()
            .filter_map(|s| s.start())
            .map(|s| s.created_at)
            .min()
            .unwrap_or(u64::MAX);
        let line = |s: &Step| (s.step_id(), s.title().to_string());
        let done = parts.iter().filter_map(|s| {
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
            changed: done.chain(own).collect(),
            asked: asked.collect(),
            open: beside
                .iter()
                .filter(|s| s.id() != step.id() && s.closed().is_none())
                .map(line)
                .collect(),
            under: under.map(|u| u.step_id()),
        }
    }
}
