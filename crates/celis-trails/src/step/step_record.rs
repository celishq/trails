//! A step, as an agent writes it.

use crate::{Anchor, StepId};
use anyhow::Result;
use celis_nostr::{Account, Note};

/// One part of the work on its project: the step it is a part of, the step it replaces, the steps and
/// decisions it uses, the places the work goes, the check that proves it, the session that wrote it, and
/// the words. A step under no other is a trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    pub anchor: Anchor,
    pub under: Option<StepId>,
    pub replaces: Option<StepId>,
    pub uses: Vec<String>,
    pub places: Vec<String>,
    pub check: Option<String>,
    pub session: Option<String>,
    pub content: String,
}

impl StepRecord {
    /// This step as a kind 1111 record that `signer` signs at `at`.
    pub fn record(&self, signer: &Account, at: u64) -> Result<Note> {
        let tag = |name: &str, value: &str| vec![name.to_string(), value.to_string()];
        let mut tags = self.anchor.tags();
        tags.extend(self.under.iter().map(|u| tag("under", u)));
        tags.extend(self.replaces.iter().map(|r| tag("replaces", r)));
        tags.extend(self.uses.iter().map(|u| tag("uses", u)));
        tags.extend(self.places.iter().map(|p| tag("at", p)));
        tags.extend(self.check.iter().map(|c| tag("check", c)));
        tags.extend(self.session.iter().map(|s| tag("session", s)));
        tags.extend([
            tag("L", NAMESPACE),
            vec!["l".to_string(), "step".to_string(), NAMESPACE.to_string()],
        ]);
        Note::write(signer, 1111u16, at, tags, self.content.clone())
    }

    /// The step `note` holds; `None` when it is not a step.
    pub fn read(note: &Note) -> Option<StepRecord> {
        if note.kind != 1111 {
            return None;
        }
        let labelled = note
            .tags
            .iter()
            .any(|t| t.len() >= 3 && t[0] == "l" && t[1] == "step" && t[2] == NAMESPACE);
        if !labelled {
            return None;
        }
        let owned = |name: &str| note.tag(name).map(str::to_string);
        let tagged = |name: &str| {
            note.tags
                .iter()
                .filter(|t| t.len() >= 2 && t[0] == name)
                .map(|t| t[1].clone())
                .collect::<Vec<_>>()
        };
        Some(StepRecord {
            anchor: Anchor::read(note)?,
            under: owned("under").map(StepId::new),
            replaces: owned("replaces").map(StepId::new),
            uses: tagged("uses"),
            places: tagged("at"),
            check: owned("check"),
            session: owned("session"),
            content: note.content.clone(),
        })
    }

    /// The first line of the text.
    pub fn title(&self) -> &str {
        self.content.lines().next().unwrap_or("")
    }

    /// The lines of the text after its title.
    pub fn detail(&self) -> impl Iterator<Item = &str> {
        self.content.lines().skip(1)
    }

    /// The first place of this step, which its line shows.
    pub fn at(&self) -> Option<&str> {
        self.places.first().map(String::as_str)
    }
}

/// The label namespace a step record names.
pub(crate) const NAMESPACE: &str = "celis.step";
