//! A live pattern that needs tidy, and why.

use crate::command::record::pattern::like;
use celis_patterns::{Pattern, PatternStore};
use celis_store::records::id::short;
use celis_store::Store;
use celis_trails::TrailStore;
use std::collections::HashMap;

/// How many steps a pattern misled before it needs tidy.
const MISLED: usize = 2;

/// How many days a pattern goes unused before it needs tidy.
const DAYS: u64 = 60;

/// How many `replaces` in a pattern's chain before it needs tidy.
const CHURN: usize = 3;

/// A live pattern in scope that needs tidy, with its reasons.
pub(crate) struct Candidate<'a> {
    pattern: Pattern<'a>,
    reasons: Vec<String>,
}

impl<'a> Candidate<'a> {
    /// Each live pattern in scope in `project` of `trails` that has a reason, oldest first (CIP-02,
    /// "Patterns").
    pub fn all(trails: &'a Store, project: &str) -> Vec<Candidate<'a>> {
        let (steps, patterns) = (trails.steps(), trails.patterns());
        let live = patterns.live(project);
        let since = steps.now().saturating_sub(DAYS * 86_400);
        let using = steps
            .all()
            .into_iter()
            .any(|s| s.record().anchor.project == project && !s.uses().is_empty());
        let replaced: HashMap<&str, Option<&str>> = patterns
            .every()
            .iter()
            .map(|p| (p.id(), p.record.replaces.as_deref()))
            .collect();
        let mut found: Vec<Candidate<'a>> = live
            .iter()
            .rev()
            .map(|pattern| {
                let id = pattern.id();
                let mut reasons = Vec::new();
                if steps.misled(id).len() >= MISLED {
                    reasons.push("misled".to_string());
                }
                if let Some(near) =
                    like(&live, &pattern.record.content, Some(pattern.clone())).first()
                {
                    reasons.push(format!("near:{}", short(near.id())));
                }
                let used = steps.uses(id).first().map(|s| s.note().created_at);
                if using && pattern.note.created_at <= since && used.is_none_or(|at| at < since) {
                    reasons.push("unused".to_string());
                }
                let mut chain = 0;
                let mut at = replaced.get(id).copied().flatten();
                while let Some(older) = at {
                    chain += 1;
                    at = replaced.get(older).copied().flatten();
                }
                if chain >= CHURN {
                    reasons.push("churn".to_string());
                }
                Candidate {
                    pattern: pattern.clone(),
                    reasons,
                }
            })
            .collect();
        found.retain(|c| !c.reasons.is_empty());
        found
    }

    /// `<id8> <reason>[,<reason>]... <When>`.
    pub fn line(&self) -> String {
        format!(
            "{} {} {}",
            short(self.pattern.id()),
            self.reasons.join(","),
            self.pattern.record.when()
        )
    }
}
