//! What a pattern says.

use crate::{Examples, Glob, Scope};
use anyhow::Result;
use celis_nostr::{Account, Note};
use celis_store::{Anchor, PatternId};

/// What one pattern says: written on a project, it holds in its scope; with the globs of the files it
/// acts on, the larger patterns it completes, and its examples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternRecord {
    pub anchor: Anchor,
    pub scope: Scope,
    pub replaces: Option<PatternId>,
    pub content: String,
    pub paths: Vec<String>,
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

    /// Its Name, or `None` when its text has none (CIP-02, "Patterns").
    pub fn name(&self) -> Option<&str> {
        name(&self.content)
    }

    /// Its When (CIP-02, "Patterns").
    pub fn when(&self) -> &str {
        when(&self.content)
    }

    /// Its Name, else its When: the line that shows it in a list.
    pub fn title(&self) -> &str {
        self.name().unwrap_or(self.when())
    }

    /// When it applies and what to do, on one line: `<When>: <Do>`, or its first line when it has no `Do`.
    pub fn brief(&self) -> String {
        let when = self.when();
        let mut lines = self.content.lines();
        match lines.find_map(|l| l.strip_prefix("Do:").map(str::trim)) {
            Some(what) => format!("{when}: {what}"),
            None => when.to_string(),
        }
    }

    /// The pattern `note` holds; `None` when it is not a pattern, or names a scope no reader knows.
    pub fn read(note: &Note) -> Option<PatternRecord> {
        let pattern = note.labels(NAMESPACE).any(|l| l == "pattern");
        if !pattern || note.kind != 1111 || note.tag("K")? != "project" {
            return None;
        }
        let paths: Vec<String> = note
            .tags
            .iter()
            .filter(|t| t.len() >= 2 && t[0] == "path")
            .map(|t| t[1].clone())
            .collect();
        let scope = match note.tag("scope") {
            Some(text) => Scope::parse(text)?,
            None => Scope::Project,
        };
        let owned = |name: &str| note.tag(name).map(str::to_string);
        Some(PatternRecord {
            anchor: Anchor::read(note)?,
            scope,
            replaces: owned("replaces").map(PatternId::new),
            content: note.content.clone(),
            paths,
            within: note
                .tags
                .iter()
                .filter(|t| t.len() >= 2 && t[0] == "within")
                .map(|t| PatternId::new(&t[1]))
                .collect(),
            examples: Examples::read(note),
        })
    }

    /// Whether the pattern matches one of `paths` (CIP-05, "Paths").
    pub fn matches(&self, paths: &[String]) -> bool {
        let globs: Vec<Glob> = self.paths.iter().map(|p| Glob::new(p)).collect();
        paths
            .iter()
            .any(|path| globs.iter().any(|g| g.matches(path)))
    }
}

/// The Name of a pattern's `content`: the text after `Name:` on its first line, or a first line that is
/// not a When before a line that is; `None` when it has none.
pub fn name(content: &str) -> Option<&str> {
    let mut lines = content.lines();
    let first = lines.next()?;
    if let Some(name) = first.strip_prefix("Name:") {
        return Some(name.trim());
    }
    let when = |line: &str| line.starts_with("When");
    (!when(first) && lines.next().is_some_and(when)).then(|| first.trim())
}

/// The When of a pattern's `content`: its first line, or its next one after a Name; of a text of one
/// line, the part before its first `:`.
pub fn when(content: &str) -> &str {
    let mut lines = content.lines();
    if name(content).is_some() {
        lines.next();
    }
    let first = lines.next().unwrap_or("");
    match content.trim_end().contains('\n') {
        true => first,
        false => first.split_once(':').map_or(first, |(when, _)| when),
    }
}

/// The label namespace of a pattern.
const NAMESPACE: &str = "celis.pattern";
