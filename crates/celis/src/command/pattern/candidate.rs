//! A live pattern that needs tidy, and why.

use crate::command::pattern::{nearest, when_words};
use celis_patterns::{Flaw, Pattern, PatternStore};
use celis_store::records::id::short;
use celis_store::Store;
use celis_trails::{Steps, TrailStore};
use std::collections::{HashMap, HashSet};

/// How many steps a pattern misled before it needs tidy.
const MISLED: usize = 2;

/// How many days a pattern goes unused before it needs tidy.
const DAYS: u64 = 60;

/// How many days a pattern goes unused before the index hides it.
const HIDE_DAYS: u64 = 7;

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

    /// The ids of the live patterns in scope in `project` of `trails` that the index hides: each that
    /// misled, and each unused for 7 days (CIP-02, "Patterns").
    pub fn hidden(trails: &'a Store, project: &str) -> HashSet<&'a str> {
        let steps = trails.steps();
        let tally = Tally::of(&steps, project);
        let since = steps.now().saturating_sub(HIDE_DAYS * 86_400);
        let live = trails.patterns().view(project).live();
        live.iter()
            .filter(|p| tally.misled(p.id()) || tally.unused(p, since))
            .map(|p| p.note.id.as_str())
            .collect()
    }

    /// Each live pattern in scope in `project` of `trails`, oldest first.
    pub fn every(trails: &'a Store, project: &str) -> Vec<Candidate<'a>> {
        let (steps, patterns) = (trails.steps(), trails.patterns());
        let live = patterns.view(project).live();
        let since = steps.now().saturating_sub(DAYS * 86_400);
        let tally = Tally::of(&steps, project);
        let sets = when_words(&live);
        let replaced: HashMap<&str, Option<&str>> = patterns
            .every()
            .iter()
            .map(|p| (p.id(), p.record.replaces.as_deref()))
            .collect();
        live.iter()
            .zip(&sets)
            .rev()
            .map(|(pattern, set)| {
                let id = pattern.id();
                let mut reasons = Vec::new();
                if tally.misled(id) {
                    reasons.push("misled".to_string());
                }
                if let Some(near) = nearest(&live, &sets, set, Some(pattern)).first() {
                    reasons.push(format!("near:{}", short(near.id())));
                }
                if tally.unused(pattern, since) {
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

/// How many steps misled each pattern, when a step last used each, and whether a step of the project
/// used a pattern.
struct Tally<'a> {
    misled: HashMap<&'a str, usize>,
    used: HashMap<&'a str, u64>,
    using: bool,
}

impl<'a> Tally<'a> {
    /// The tally of `steps`, with the uses of `project`.
    fn of(steps: &'a Steps<'a>, project: &str) -> Tally<'a> {
        let using = steps
            .all()
            .into_iter()
            .any(|s| s.record().anchor.project == project && !s.uses().is_empty());
        let mut misled: HashMap<&str, usize> = HashMap::new();
        let mut used: HashMap<&str, u64> = HashMap::new();
        for step in steps.all() {
            for (id, _) in step.misled() {
                *misled.entry(id).or_default() += 1;
            }
            for id in step.uses() {
                used.insert(id, step.note().created_at);
            }
        }
        Tally {
            misled,
            used,
            using,
        }
    }

    /// Whether the pattern `id` misled 2 or more steps.
    fn misled(&self, id: &str) -> bool {
        self.misled.get(id).copied().unwrap_or(0) >= MISLED
    }

    /// Whether `pattern` was made at or before `since` and no step used it since, while a step of the
    /// project used a pattern.
    fn unused(&self, pattern: &Pattern, since: u64) -> bool {
        let used = self.used.get(pattern.id()).copied();
        self.using && pattern.note.created_at <= since && used.is_none_or(|at| at < since)
    }
}
