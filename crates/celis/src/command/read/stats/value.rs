//! What shows that celis pays: the value of the patterns in the last 28 days.

use super::lifecycle::promoted;
use crate::progress::Model;
use celis_patterns::{PatternEnd, Scope};
use celis_trails::{State, Step};
use serde_json::json;
use std::collections::{BTreeSet, HashSet};

/// The seconds of the span the value counts.
const SPAN: u64 = 28 * 86_400;

/// The value of the last 28 days: the done asks, those that used a pattern, their corrections, the
/// failed and the missed among them, the first-pass percent with and without a pattern, the recalls,
/// uses and held uses, the uses of a `me` pattern in another project, the agents that used a pattern,
/// and the patterns promoted.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Value {
    pub asks: usize,
    pub covered: usize,
    pub corrections: usize,
    pub failed: usize,
    pub missed: usize,
    pub with: Option<u64>,
    pub without: Option<u64>,
    pub recalled: usize,
    pub used: usize,
    pub held: usize,
    pub reach: usize,
    pub agents: Vec<String>,
    pub promoted: usize,
}

impl Value {
    /// The value of the asks of `project` and the patterns in scope in it, or of every ask and pattern
    /// with none (CIP-02, "Stats").
    pub fn of(model: &Model, project: Option<&str>) -> Value {
        let steps = model.steps;
        let since = steps.now().saturating_sub(SPAN);
        let ours = |s: &Step| project.is_none_or(|p| s.record().anchor.project == p);
        let asks: Vec<Step> = steps
            .roots()
            .into_iter()
            .filter(|s| ours(s) && s.state() == State::Done)
            .filter(|s| s.done_record().is_some_and(|d| d.created_at >= since))
            .collect();
        let trees: Vec<Step> = asks.iter().flat_map(|a| a.tree()).collect();
        let patterns = match project {
            Some(project) => model.patterns.scoped(project),
            None => model.patterns.every().to_vec(),
        };
        let mut value = Value {
            asks: asks.len(),
            covered: asks.iter().filter(|a| uses_any(**a)).count(),
            ..Value::default()
        };
        for fix in trees.iter().filter(|s| s.fixes()) {
            value.corrections += 1;
            let old = fix.record().replaces.as_deref();
            let Some(old) = old.and_then(|id| steps.find(id).ok()) else {
                continue;
            };
            if uses_any(old.trail()) {
                value.failed += 1;
                continue;
            }
            let began = fix.trail().note().created_at;
            let made = old.note().created_at;
            let missed = patterns.iter().any(|p| {
                let replaced = p.record.replaces.as_deref();
                let older = replaced.and_then(|id| patterns.iter().find(|o| o.id() == id));
                p.note.created_at >= began && older.is_some_and(|o| o.note.created_at < made)
            });
            value.missed += usize::from(missed);
        }
        let done = trees
            .iter()
            .filter(|s| s.is_work() && s.state() == State::Done);
        let (led, plain): (Vec<Step>, Vec<Step>) = done.partition(|s| !s.uses().is_empty());
        value.with = first_pass(&led);
        value.without = first_pass(&plain);
        for step in &trees {
            value.recalled += step.recalls().len();
            value.used += step.uses().len();
            if step.revised().is_empty() && step.misled().is_empty() {
                value.held += step.uses().len();
            }
        }
        let agents = trees.iter().filter(|s| !s.uses().is_empty());
        let agents = agents.filter_map(|s| s.record().agent.clone());
        value.agents = agents.collect::<BTreeSet<_>>().into_iter().collect();
        let mine: HashSet<&str> = patterns
            .iter()
            .filter(|p| p.record.scope == Scope::Me)
            .map(|p| p.id())
            .collect();
        for step in steps.all() {
            if step.note().created_at < since {
                continue;
            }
            let project = &step.record().anchor.project;
            let other = step.uses().into_iter().filter(|id| {
                let pattern = patterns.iter().find(|p| p.id() == *id);
                mine.contains(id) && pattern.is_some_and(|p| &p.record.anchor.project != project)
            });
            value.reach += other.count();
        }
        value.promoted = patterns
            .iter()
            .filter(|p| promoted(p))
            .filter(|p| matches!(p.end, Some(PatternEnd::Dropped(d)) if d.created_at >= since))
            .count();
        value
    }

    /// Its line of `stats` (CIP-02, "Stats").
    pub fn line(&self) -> String {
        let percent = |p: Option<u64>| p.map_or("-".to_string(), |p| format!("{p}%"));
        let agents = match self.agents.is_empty() {
            true => "-".to_string(),
            false => self.agents.join(","),
        };
        format!(
            "value last 28 days: asks:{} covered:{} corrections:{} failed:{} missed:{} first-pass:{}/{} \
             recalled:{} used:{} held:{} reach:{} agents:{agents} promoted:{}",
            self.asks,
            self.covered,
            self.corrections,
            self.failed,
            self.missed,
            percent(self.with),
            percent(self.without),
            self.recalled,
            self.used,
            self.held,
            self.reach,
            self.promoted
        )
    }

    /// Its object of `stats --json` (CIP-02, "Stats").
    pub fn json(&self) -> serde_json::Value {
        json!({"asks": self.asks, "covered": self.covered, "corrections": self.corrections,
            "failed": self.failed, "missed": self.missed, "with": self.with,
            "without": self.without, "recalled": self.recalled, "used": self.used,
            "held": self.held, "reach": self.reach, "agents": self.agents,
            "promoted": self.promoted})
    }
}

/// Whether a step of the tree of `ask` uses a pattern.
fn uses_any(ask: Step) -> bool {
    ask.tree().iter().any(|s| !s.uses().is_empty())
}

/// The percent of `steps` that no step revised; none with no step.
fn first_pass(steps: &[Step]) -> Option<u64> {
    let held = steps.iter().filter(|s| s.revised().is_empty()).count();
    (!steps.is_empty()).then(|| (held * 100 / steps.len()) as u64)
}

#[cfg(test)]
mod tests;
