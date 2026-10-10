//! What an agent does to a step.

use crate::step::NAMESPACE;
use anyhow::Result;
use celis_wire::{Account, Note};

/// What an agent does to a step: takes it up, closes it done with its proof, or records what the person
/// asked for while it was open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    Start,
    Done { proof: String },
    Steer { text: String },
}

impl Mark {
    /// This mark on the step `step` as a kind 1985 record that `signer` signs at `at`.
    pub fn record(&self, signer: &Account, step: &str, at: u64) -> Result<Note> {
        let (label, content) = match self {
            Mark::Start => ("start", ""),
            Mark::Done { proof } => ("done", proof.as_str()),
            Mark::Steer { text } => ("steer", text.as_str()),
        };
        let tags = vec![
            vec!["L".to_string(), NAMESPACE.to_string()],
            vec!["l".to_string(), label.to_string(), NAMESPACE.to_string()],
            vec!["e".to_string(), step.to_string()],
        ];
        Note::write(signer, 1985u16, at, tags, content)
    }

    /// The step id and the mark `note` holds; `None` when it is not a mark.
    pub fn read(note: &Note) -> Option<(String, Mark)> {
        if note.kind != 1985 {
            return None;
        }
        let label = note
            .tags
            .iter()
            .find(|t| t.len() >= 3 && t[0] == "l" && t[2] == NAMESPACE)?;
        let mark = match label[1].as_str() {
            "start" => Mark::Start,
            "done" => Mark::Done {
                proof: note.content.clone(),
            },
            "steer" => Mark::Steer {
                text: note.content.clone(),
            },
            _ => return None,
        };
        Some((note.tag("e")?.to_string(), mark))
    }
}

#[cfg(test)]
mod tests;
