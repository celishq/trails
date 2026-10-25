//! CIP-05: a pattern, what to choose in a context and why, that holds in its scope until another replaces
//! it.

pub mod example;
pub mod examples;
pub mod fit;
pub mod glob;
pub mod pattern_end;
pub mod pattern_entry;
pub mod patterns;
pub mod scope;

use crate::{Anchor, Examples, Fit, Glob, PatternId, Place, Scope};
use anyhow::Result;
use celis_nostr::{Account, Note};

/// One pattern, written on a project, that holds in its scope; with the globs of the files it acts on,
/// its examples, and its `binds` label, which makes it a rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub anchor: Anchor,
    pub scope: Scope,
    pub replaces: Option<PatternId>,
    pub content: String,
    pub paths: Vec<String>,
    pub binds: bool,
    pub examples: Examples,
}

impl Pattern {
    /// This pattern as a kind 1111 record that `signer` signs at `at`.
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
        let pubkey = signer.pubkey().hex();
        let examples = self.examples.sides();
        tags.extend(examples.map(|(side, example)| example.tag(side, &pubkey)));
        let paths = self.paths.iter();
        tags.extend(paths.map(|p| vec!["path".to_string(), p.clone()]));
        tags.push(vec!["L".to_string(), NAMESPACE.to_string()]);
        let labels = [("decision", true), ("binds", self.binds)].into_iter();
        tags.extend(
            labels
                .filter(|(_, on)| *on)
                .map(|(label, _)| vec!["l".to_string(), label.to_string(), NAMESPACE.to_string()]),
        );
        Note::write(signer, 1111u16, at, tags, self.content.clone())
    }

    /// When it applies and what to do, on one line, without the reason: `<When>: <Do>`.
    pub fn brief(&self) -> String {
        let line = |name: &str| {
            let mut lines = self.content.lines();
            lines.find_map(|l| l.strip_prefix(name).map(str::trim))
        };
        match (self.content.lines().next(), line("Do:")) {
            (Some(when), Some(what)) => format!("{when}: {what}"),
            _ => self
                .content
                .split_once(", because ")
                .map_or(self.content.as_str(), |(brief, _)| brief)
                .to_string(),
        }
    }

    /// The pattern `note` holds; `None` when it is not a pattern, or names a scope no reader knows.
    pub fn read(note: &Note) -> Option<Pattern> {
        if note.kind != 1111 || !labelled(note, "decision") || note.tag("K")? != "project" {
            return None;
        }
        let scope = match note.tag("scope") {
            Some(text) => Scope::parse(text)?,
            None => Scope::Project,
        };
        let owned = |name: &str| note.tag(name).map(str::to_string);
        Some(Pattern {
            anchor: Anchor::read(note)?,
            scope,
            replaces: owned("replaces").map(PatternId::new),
            content: note.content.clone(),
            paths: note
                .tags
                .iter()
                .filter(|t| t.len() >= 2 && t[0] == "path")
                .map(|t| t[1].clone())
                .collect(),
            binds: labelled(note, "binds") || labelled(note, "promoted"),
            examples: Examples::read(note),
        })
    }

    /// How narrowly the pattern fits the narrowest of `paths`; `None` when it matches none of them
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

/// Whether `note` holds the label `label` of a pattern.
fn labelled(note: &Note, label: &str) -> bool {
    note.tags
        .iter()
        .any(|t| t.len() >= 3 && t[0] == "l" && t[1] == label && t[2] == NAMESPACE)
}

/// The label namespace of a pattern.
const NAMESPACE: &str = "celis.decision";
