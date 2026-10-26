//! What a pattern says.

use crate::{Examples, Glob, Scope};
use anyhow::Result;
use celis_nostr::{Account, Note};
use celis_store::{Anchor, PatternId, Place};

/// What one pattern says: written on a project, it holds in its scope; with the globs of the files it
/// acts on, the larger patterns it completes, its examples, and whether an older writer gave it the
/// `binds` label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternRecord {
    pub anchor: Anchor,
    pub scope: Scope,
    pub replaces: Option<PatternId>,
    pub content: String,
    pub paths: Vec<String>,
    pub old_rule: bool,
    pub within: Vec<PatternId>,
    pub examples: Examples,
}

impl PatternRecord {
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
        let within = self.within.iter();
        tags.extend(within.map(|id| vec!["within".to_string(), id.to_string()]));
        tags.push(vec!["L".to_string(), NAMESPACE.to_string()]);
        tags.push(vec![
            "l".to_string(),
            "pattern".to_string(),
            NAMESPACE.to_string(),
        ]);
        Note::write(signer, 1111u16, at, tags, self.content.clone())
    }

    /// The first line of its text; of a text of one line, the part before its first `:`.
    pub fn when(&self) -> &str {
        let first = self.content.lines().next().unwrap_or("");
        match self.content.trim_end().contains('\n') {
            true => first,
            false => first.split_once(':').map_or(first, |(when, _)| when),
        }
    }

    /// When it applies and what to do, on one line: `<When>: <Do>`, or its first line when it has no `Do`.
    pub fn brief(&self) -> String {
        let when = self.content.lines().next().unwrap_or("");
        let mut lines = self.content.lines();
        match lines.find_map(|l| l.strip_prefix("Do:").map(str::trim)) {
            Some(what) => format!("{when}: {what}"),
            None => when.to_string(),
        }
    }

    /// The pattern `note` holds; `None` when it is not a pattern, or names a scope no reader knows, such
    /// as `trail:<id>`.
    pub fn read(note: &Note) -> Option<PatternRecord> {
        let pattern = [(NAMESPACE, "pattern"), (OLD_NAMESPACE, "decision")]
            .into_iter()
            .any(|(namespace, label)| labelled(note, label, namespace));
        if !pattern || note.kind != 1111 || note.tag("K")? != "project" {
            return None;
        }
        let mut paths: Vec<String> = note
            .tags
            .iter()
            .filter(|t| t.len() >= 2 && t[0] == "path")
            .map(|t| t[1].clone())
            .collect();
        let scope = match note.tag("scope") {
            Some(text) => match text.strip_prefix("at:").filter(|p| !p.is_empty()) {
                Some(place) => {
                    let path = Place::new(place).path().trim_end_matches('/');
                    if !matches!(path, "." | "") {
                        paths.extend([path.to_string(), format!("{path}/**")]);
                    }
                    Scope::Project
                }
                None => Scope::parse(text)?,
            },
            None => Scope::Project,
        };
        let owned = |name: &str| note.tag(name).map(str::to_string);
        Some(PatternRecord {
            anchor: Anchor::read(note)?,
            scope,
            replaces: owned("replaces").map(PatternId::new),
            content: note.content.clone(),
            paths,
            old_rule: [NAMESPACE, OLD_NAMESPACE]
                .iter()
                .any(|namespace| labelled(note, "binds", namespace)),
            within: note
                .tags
                .iter()
                .filter(|t| t.len() >= 2 && t[0] == "within")
                .map(|t| PatternId::new(&t[1]))
                .collect(),
            examples: Examples::read(note),
        })
    }

    /// The whole id of the trail that `note`, a pattern with a `trail:<id>` scope that older writers
    /// wrote, names; `None` for any other record (CIP-05, "Reading").
    pub fn trail(note: &Note) -> Option<&str> {
        let pattern = [(NAMESPACE, "pattern"), (OLD_NAMESPACE, "decision")];
        let pattern = pattern
            .into_iter()
            .any(|(namespace, label)| labelled(note, label, namespace));
        let trail = note.tag("scope")?.strip_prefix("trail:")?;
        (pattern && note.kind == 1111 && !trail.is_empty()).then_some(trail)
    }

    /// Whether the pattern matches one of `paths` (CIP-05, "Paths").
    pub fn matches(&self, paths: &[String]) -> bool {
        let globs: Vec<Glob> = self.paths.iter().map(|p| Glob::new(p)).collect();
        paths
            .iter()
            .any(|path| globs.iter().any(|g| g.matches(path)))
    }
}

/// Whether `note` holds the label `label` in `namespace`.
fn labelled(note: &Note, label: &str, namespace: &str) -> bool {
    note.tags
        .iter()
        .any(|t| t.len() >= 3 && t[0] == "l" && t[1] == label && t[2] == namespace)
}

/// The label namespace of a pattern.
const NAMESPACE: &str = "celis.pattern";

/// The label namespace of a pattern that older versions wrote.
const OLD_NAMESPACE: &str = "celis.decision";
