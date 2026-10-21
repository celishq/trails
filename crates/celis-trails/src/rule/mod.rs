//! CIP-05: a rule, one sentence that holds in its scope until another replaces it.

pub mod call;
pub mod fit;
pub mod glob;
pub mod rule_end;
pub mod rule_entry;
pub mod rules;
pub mod scope;
pub mod trigger;

use crate::{Anchor, Call, Fit, Glob, Place, RuleId, Scope, Trigger};
use anyhow::Result;
use celis_nostr::{Account, Note};
use regex::Regex;

/// One rule, written on a project, that holds in its scope; with the calls it acts on, and its
/// `situation` and `promoted` labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub anchor: Anchor,
    pub scope: Scope,
    pub replaces: Option<RuleId>,
    pub content: String,
    pub trigger: Trigger,
    pub situation: bool,
    pub promoted: bool,
}

impl Rule {
    /// This rule as a kind 1111 record that `signer` signs at `at`.
    pub fn record(&self, signer: &Account, at: u64) -> Result<Note> {
        let mut tags = self.anchor.tags();
        let scope = (self.scope != Scope::Project).then(|| self.scope.text());
        let replaces = self.replaces.as_ref().map(|r| r.to_string());
        let optional = [("scope", &scope), ("replaces", &replaces)];
        tags.extend(
            optional
                .into_iter()
                .filter_map(|(name, value)| Some(vec![name.to_string(), value.clone()?])),
        );
        tags.extend(self.trigger.tags());
        tags.push(vec!["L".to_string(), NAMESPACE.to_string()]);
        let labels = [("decision", true), ("situation", self.situation)];
        let labels = labels.into_iter().chain([("promoted", self.promoted)]);
        tags.extend(
            labels
                .filter(|(_, on)| *on)
                .map(|(label, _)| vec!["l".to_string(), label.to_string(), NAMESPACE.to_string()]),
        );
        Note::write(signer, 1111u16, at, tags, self.content.clone())
    }

    /// The text up to `, because`: when it applies and what to do, without the reason.
    pub fn brief(&self) -> &str {
        self.content
            .split_once(", because ")
            .map_or(self.content.as_str(), |(rule, _)| rule)
    }

    /// The rule `note` holds; `None` when it is not a rule, or names a scope no reader knows.
    pub fn read(note: &Note) -> Option<Rule> {
        if note.kind != 1111 || !labelled(note, "decision") || note.tag("K")? != "project" {
            return None;
        }
        let scope = match note.tag("scope") {
            Some(text) => Scope::parse(text)?,
            None => Scope::Project,
        };
        let owned = |name: &str| note.tag(name).map(str::to_string);
        Some(Rule {
            anchor: Anchor::read(note)?,
            scope,
            replaces: owned("replaces").map(RuleId::new),
            content: note.content.clone(),
            trigger: Trigger::read(note),
            situation: labelled(note, "situation"),
            promoted: labelled(note, "promoted"),
        })
    }

    /// How narrowly the rule fits `call`; `None` when it does not match it (CIP-05, "Triggers").
    pub fn fit(&self, call: &Call) -> Option<Fit> {
        match call {
            Call::File { path, text } => self.file(path, text.as_deref()),
            Call::Command { line, files } => {
                let command = self.trigger.command.as_deref();
                let command = command.filter(|c| found(c, line)).map(|_| Fit::Pattern);
                let files = files.iter().filter_map(|f| self.file(f, Some(line)));
                files.chain(command).min()
            }
            Call::Tool { name } => {
                let tool = self.trigger.tool.as_deref();
                tool.filter(|t| found(t, name)).map(|_| Fit::Pattern)
            }
        }
    }

    /// How narrowly the rule fits a call on the file `path` whose text is `text`.
    fn file(&self, path: &str, text: Option<&str>) -> Option<Fit> {
        let content = self.trigger.content.as_deref();
        if content.is_some_and(|c| !text.is_some_and(|t| found(c, t))) {
            return None;
        }
        let at = match &self.scope {
            Scope::At(place) => Place::new(place)
                .meets(Place::new(path))
                .then_some(Fit::Named),
            _ => None,
        };
        let globs = self.trigger.paths.iter().map(|p| Glob::new(p));
        let globs = globs
            .filter(|g| g.matches(path))
            .map(|g| match g.is_literal() {
                true => Fit::Named,
                false if content.is_some() => Fit::Pattern,
                false => Fit::Glob,
            });
        at.into_iter().chain(globs).min()
    }
}

/// Whether the regex `pattern` finds a match in `text`; a pattern that does not compile finds none.
fn found(pattern: &str, text: &str) -> bool {
    Regex::new(pattern).is_ok_and(|r| r.is_match(text))
}

/// Whether `note` holds the label `label` of a rule.
fn labelled(note: &Note, label: &str) -> bool {
    note.tags
        .iter()
        .any(|t| t.len() >= 3 && t[0] == "l" && t[1] == label && t[2] == NAMESPACE)
}

/// The label namespace of a rule.
const NAMESPACE: &str = "celis.decision";
