//! CIP-05: a rule, one sentence that holds in its scope until another replaces it.

pub mod fit;
pub mod glob;
pub mod pair;
pub mod rule_end;
pub mod rule_entry;
pub mod rules;
pub mod scope;

use crate::{Anchor, Fit, Glob, Pair, Place, RuleId, Scope};
use anyhow::Result;
use celis_nostr::{Account, Note};

/// One rule, written on a project, that holds in its scope; with the globs of the files it acts on, its
/// `situation` and `promoted` labels, and the good and the bad step of its pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub anchor: Anchor,
    pub scope: Scope,
    pub replaces: Option<RuleId>,
    pub content: String,
    pub paths: Vec<String>,
    pub situation: bool,
    pub promoted: bool,
    pub pair: Pair,
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
        let pair = self.pair.sides();
        tags.extend(pair.map(|(side, id)| vec![side.to_string(), id.to_string()]));
        let paths = self.paths.iter();
        tags.extend(paths.map(|p| vec!["path".to_string(), p.clone()]));
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

    /// Whether it is a pattern: a rule with examples, which guides and never binds.
    pub fn is_pattern(&self) -> bool {
        self.pair.sides().next().is_some()
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
            paths: note
                .tags
                .iter()
                .filter(|t| t.len() >= 2 && t[0] == "path")
                .map(|t| t[1].clone())
                .collect(),
            situation: labelled(note, "situation"),
            promoted: labelled(note, "promoted"),
            pair: Pair::read(note),
        })
    }

    /// How narrowly the rule fits the narrowest of `paths`; `None` when it matches none of them
    /// (CIP-05, "Triggers").
    pub fn fit(&self, paths: &[String]) -> Option<Fit> {
        let place = match &self.scope {
            Scope::At(place) => Some(Place::new(place)),
            _ => None,
        };
        let globs: Vec<Glob> = self.paths.iter().map(|p| Glob::new(p)).collect();
        let fits = paths.iter().filter_map(|path| {
            let at = place.as_ref().filter(|p| p.meets(Place::new(path)));
            let at = at.map(|_| Fit::Named);
            let globs = globs.iter().filter(|g| g.matches(path));
            let globs = globs.map(|g| match g.is_literal() {
                true => Fit::Named,
                false => Fit::Glob,
            });
            at.into_iter().chain(globs).min()
        });
        fits.min()
    }
}

/// Whether `note` holds the label `label` of a rule.
fn labelled(note: &Note, label: &str) -> bool {
    note.tags
        .iter()
        .any(|t| t.len() >= 3 && t[0] == "l" && t[1] == label && t[2] == NAMESPACE)
}

/// The label namespace of a rule.
const NAMESPACE: &str = "celis.decision";
