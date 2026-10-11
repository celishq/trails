//! A step, as an agent writes it.

use crate::Anchor;
use anyhow::Result;
use celis_wire::{Account, Note};

/// One part of the work on its project: the step it is a part of, the done step it fixes, the places the
/// work goes, the check that proves it, the session that wrote it, and the words. A step under no other
/// is a trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub anchor: Anchor,
    pub under: Option<String>,
    pub fixes: Option<String>,
    pub places: Vec<String>,
    pub check: Option<String>,
    pub session: Option<String>,
    pub content: String,
}

impl Step {
    /// This step as a kind 1111 record that `signer` signs at `at`.
    pub fn record(&self, signer: &Account, at: u64) -> Result<Note> {
        let tag = |name: &str, value: &str| vec![name.to_string(), value.to_string()];
        let mut tags = self.anchor.tags();
        tags.extend(self.under.iter().map(|u| tag("under", u)));
        tags.extend(self.fixes.iter().map(|f| tag("fixes", f)));
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
    pub fn read(note: &Note) -> Option<Step> {
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
        let project = note.tag("I")?.strip_prefix("project:")?.to_string();
        let name = note.tag("project").unwrap_or_default().to_string();
        let owned = |name: &str| note.tag(name).map(str::to_string);
        let tagged = |name: &str| {
            note.tags
                .iter()
                .filter(|t| t.len() >= 2 && t[0] == name)
                .map(|t| t[1].clone())
                .collect::<Vec<_>>()
        };
        Some(Step {
            anchor: Anchor { project, name },
            under: owned("under"),
            fixes: owned("fixes"),
            places: [tagged("at"), tagged("file")].concat(),
            check: owned("check"),
            session: owned("session"),
            content: note.content.clone(),
        })
    }

    /// The `--json` value for this step, as `note` holds it.
    pub fn value(&self, note: &Note) -> serde_json::Value {
        serde_json::json!({
            "id": note.id,
            "created_at": note.created_at,
            "project": self.anchor.project,
            "links": self.links(),
            "places": self.places,
            "check": self.check,
            "content": self.content,
        })
    }

    /// The `--json` links of this step: its `under` and `fixes`, in that order.
    pub fn links(&self) -> serde_json::Value {
        let links: Vec<serde_json::Value> = [("under", &self.under), ("fixes", &self.fixes)]
            .into_iter()
            .filter_map(|(kind, to)| Some(serde_json::json!({"to": to.as_ref()?, "kind": kind})))
            .collect();
        serde_json::Value::Array(links)
    }

    /// The first line of the text.
    pub fn title(&self) -> &str {
        self.content.lines().next().unwrap_or("")
    }

    /// The lines of the text after its title.
    pub fn detail(&self) -> impl Iterator<Item = &str> {
        self.content.lines().skip(1)
    }

    /// The text line for this step, as `note` holds it.
    pub fn line(&self, note: &Note) -> String {
        let id: String = crate::id::short(&note.id);
        let mut line = format!("{} [{}] {}", id, self.anchor.name, self.title());
        if let Some(at) = self.at() {
            line.push_str(&format!(" · {at}"));
        }
        line
    }

    /// The first place of this step, which its line shows.
    pub fn at(&self) -> Option<&str> {
        self.places.first().map(String::as_str)
    }
}

/// The label namespace a step record names.
pub(crate) const NAMESPACE: &str = "celis.step";
