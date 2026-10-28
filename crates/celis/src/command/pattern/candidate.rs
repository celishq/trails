//! A live pattern that needs tidy, and why.

use crate::command::pattern::like;
use celis_patterns::{Flaw, Pattern, PatternStore};
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

/// A live pattern in scope, with its reasons to need tidy and the length of its `replaces` chain.
pub(crate) struct Candidate<'a> {
    pattern: Pattern<'a>,
    reasons: Vec<String>,
    chain: usize,
}

impl<'a> Candidate<'a> {
    /// Each live pattern in scope in `project` of `trails` that has a reason, oldest first (CIP-02,
    /// "Patterns").
    pub fn all(trails: &'a Store, project: &str) -> Vec<Candidate<'a>> {
        let mut found = Candidate::every(trails, project);
        found.retain(|c| !c.reasons.is_empty());
        found
    }

    /// Each live pattern in scope in `project` of `trails`, oldest first.
    pub fn every(trails: &'a Store, project: &str) -> Vec<Candidate<'a>> {
        let (steps, patterns) = (trails.steps(), trails.patterns());
        let live = patterns.view(project).live();
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
        live.iter()
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
                let record = &pattern.record;
                let flaws = Flaw::of(&record.content).into_iter();
                reasons.extend(flaws.map(|f| f.word().to_string()));
                Candidate {
                    pattern: pattern.clone(),
                    reasons,
                    chain,
                }
            })
            .collect()
    }

    pub fn pattern(&self) -> &Pattern<'a> {
        &self.pattern
    }

    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }

    /// How many `replaces` its chain holds.
    pub fn chain(&self) -> usize {
        self.chain
    }

    /// `<id8> <reason>[,<reason>]... <Name>`.
    pub fn line(&self) -> String {
        format!(
            "{} {} {}",
            short(self.pattern.id()),
            self.reasons.join(","),
            self.pattern.record.title()
        )
    }
}
