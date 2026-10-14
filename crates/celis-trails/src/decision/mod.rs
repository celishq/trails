//! A decision: one sentence that holds in its scope until another replaces it.

pub mod decisions;
pub mod scope;

use crate::{Anchor, DecisionId, Scope};
use anyhow::Result;
use celis_nostr::{Account, Note};

/// One decision, written on a project, that holds in its scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub anchor: Anchor,
    pub scope: Scope,
    pub replaces: Option<DecisionId>,
    pub content: String,
}

impl Decision {
    /// This decision as a kind 1111 record that `signer` signs at `at`.
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
        tags.extend([
            vec!["L".to_string(), NAMESPACE.to_string()],
            vec![
                "l".to_string(),
                "decision".to_string(),
                NAMESPACE.to_string(),
            ],
        ]);
        Note::write(signer, 1111u16, at, tags, self.content.clone())
    }

    /// The text up to `, because`: when it applies and what to do, without the reason.
    pub fn brief(&self) -> &str {
        self.content
            .split_once(", because ")
            .map_or(self.content.as_str(), |(rule, _)| rule)
    }

    /// The decision `note` holds; `None` when it is not a decision, or names a scope no reader knows.
    pub fn read(note: &Note) -> Option<Decision> {
        let labelled = note
            .tags
            .iter()
            .any(|t| t.len() >= 3 && t[0] == "l" && t[1] == "decision" && t[2] == NAMESPACE);
        if note.kind != 1111 || !labelled || note.tag("K")? != "project" {
            return None;
        }
        let scope = match note.tag("scope") {
            Some(text) => Scope::parse(text)?,
            None => Scope::Project,
        };
        let owned = |name: &str| note.tag(name).map(str::to_string);
        Some(Decision {
            anchor: Anchor::read(note)?,
            scope,
            replaces: owned("replaces").map(DecisionId::new),
            content: note.content.clone(),
        })
    }
}

/// The label namespace of a decision.
const NAMESPACE: &str = "celis.decision";
