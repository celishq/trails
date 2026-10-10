//! What an agent does to a step.

use crate::step::NAMESPACE;
use anyhow::Result;
use celis_wire::{Account, Note};

/// What an agent does to a step: takes it up, closes it done with its proof and what it learned, records
/// what the person asked for while it was open, or drops it with the reason its work is no longer wanted.
/// It gives a step new text. On a trail, a session joins it, the hooks record a prompt while no step
/// works, and the end of the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    Start,
    Done {
        proof: String,
        path: Option<String>,
        pitfall: Option<String>,
    },
    Steer {
        text: String,
    },
    Drop {
        reason: String,
    },
    Edit {
        text: String,
    },
    Ask {
        text: String,
    },
    Join {
        session: String,
    },
    End {
        session: Option<String>,
    },
}

impl Mark {
    /// This mark on the step `step` as a kind 1985 record that `signer` signs at `at`.
    pub fn record(&self, signer: &Account, step: &str, at: u64) -> Result<Note> {
        let (label, content) = match self {
            Mark::Start => ("start", ""),
            Mark::Done { proof, .. } => ("done", proof.as_str()),
            Mark::Steer { text } => ("steer", text.as_str()),
            Mark::Drop { reason } => ("drop", reason.as_str()),
            Mark::Edit { text } => ("edit", text.as_str()),
            Mark::Ask { text } => ("ask", text.as_str()),
            Mark::Join { .. } => ("join", ""),
            Mark::End { .. } => ("end", ""),
        };
        let mut tags = vec![
            vec!["L".to_string(), NAMESPACE.to_string()],
            vec!["l".to_string(), label.to_string(), NAMESPACE.to_string()],
            vec!["e".to_string(), step.to_string()],
        ];
        let session = match self {
            Mark::Join { session } => Some(session),
            Mark::End { session } => session.as_ref(),
            _ => None,
        };
        tags.extend(session.map(|s| vec!["session".to_string(), s.clone()]));
        if let Mark::Done { path, pitfall, .. } = self {
            tags.extend(path.iter().map(|p| vec!["path".to_string(), p.clone()]));
            tags.extend(
                pitfall
                    .iter()
                    .map(|p| vec!["pitfall".to_string(), p.clone()]),
            );
        }
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
                path: note.tag("path").map(str::to_string),
                pitfall: note.tag("pitfall").map(str::to_string),
            },
            "steer" => Mark::Steer {
                text: note.content.clone(),
            },
            "drop" => Mark::Drop {
                reason: note.content.clone(),
            },
            "edit" => Mark::Edit {
                text: note.content.clone(),
            },
            "ask" => Mark::Ask {
                text: note.content.clone(),
            },
            "join" => Mark::Join {
                session: note.tag("session")?.to_string(),
            },
            "end" => Mark::End {
                session: note.tag("session").map(str::to_string),
            },
            _ => return None,
        };
        Some((note.tag("e")?.to_string(), mark))
    }
}

#[cfg(test)]
mod tests;
