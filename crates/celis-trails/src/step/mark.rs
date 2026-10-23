//! What an agent does to a step.

use crate::step::step_record::NAMESPACE;
use crate::{By, Done, ProofFile};
use anyhow::Result;
use celis_nostr::{Account, Note};
use std::collections::HashMap;

/// What an agent does to a step: takes it up, closes it done, or drops it with the reason its work is no
/// longer wanted. It gives a step a new title or text, places or check. On a trail, the hook records the person's
/// prompt, and a name lets other work use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    Start,
    Done(Done),
    Drop {
        reason: String,
    },
    Edit {
        text: String,
        places: Vec<String>,
        check: Option<String>,
    },
    Prompt {
        text: String,
    },
    Name {
        name: String,
    },
}

impl Mark {
    /// This mark on the step `step` as a kind 1985 record that `signer` signs at `at`.
    pub fn record(&self, signer: &Account, step: &str, at: u64) -> Result<Note> {
        let (label, content) = match self {
            Mark::Start => ("start", ""),
            Mark::Done(done) => ("done", done.proof.as_str()),
            Mark::Drop { reason } => ("drop", reason.as_str()),
            Mark::Edit { text, .. } => ("edit", text.as_str()),
            Mark::Prompt { text } => ("prompt", text.as_str()),
            Mark::Name { name } => ("name", name.as_str()),
        };
        let pubkey = signer.pubkey().hex();
        let mut content = content.to_string();
        let mut tags = vec![
            vec!["L".to_string(), NAMESPACE.to_string()],
            vec!["l".to_string(), label.to_string(), NAMESPACE.to_string()],
            vec!["e".to_string(), step.to_string()],
        ];
        if let Mark::Done(done) = self {
            let changed = done
                .changed
                .iter()
                .map(|c| vec!["changed".to_string(), c.clone()]);
            tags.extend(changed);
            let by = done
                .by
                .iter()
                .map(|b| vec!["by".to_string(), b.text().to_string()]);
            tags.extend(by);
            for file in &done.files {
                content.push_str(&format!("\n{}", file.url(&pubkey)));
                tags.push(file.tag(&pubkey));
            }
        }
        if let Mark::Edit { places, check, .. } = self {
            tags.extend(places.iter().map(|p| vec!["at".to_string(), p.clone()]));
            tags.extend(check.iter().map(|c| vec!["check".to_string(), c.clone()]));
        }
        Note::write(signer, 1985u16, at, tags, &content)
    }

    /// The first drop of each record id that `records` holds, by when it was written, then by id.
    pub fn drops(records: &[Note]) -> HashMap<String, &Note> {
        let mut drops: HashMap<String, &Note> = HashMap::new();
        for note in records {
            if let Some((id, Mark::Drop { .. })) = Mark::read(note) {
                let first = drops.entry(id).or_insert(note);
                if (note.created_at, &note.id) < (first.created_at, &first.id) {
                    *first = note;
                }
            }
        }
        drops
    }

    /// The step id and the mark `note` holds; `None` when it is not a mark, or is a done or a drop with
    /// no content.
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
            "done" | "drop" if note.content.trim().is_empty() => return None,
            "done" => {
                let files: Vec<ProofFile> = note
                    .tags
                    .iter()
                    .filter_map(|t| ProofFile::read(t))
                    .collect();
                Mark::Done(Done {
                    proof: proof(&note.content, &files),
                    changed: note.tag("changed").map(str::to_string),
                    by: note.tag("by").and_then(By::parse),
                    files,
                })
            }
            "drop" => Mark::Drop {
                reason: note.content.clone(),
            },
            "edit" => Mark::Edit {
                text: note.content.clone(),
                places: note
                    .tags
                    .iter()
                    .filter(|t| t.len() >= 2 && t[0] == "at")
                    .map(|t| t[1].clone())
                    .collect(),
                check: note.tag("check").map(str::to_string),
            },
            "prompt" => Mark::Prompt {
                text: note.content.clone(),
            },
            "name" => Mark::Name {
                name: note.content.clone(),
            },
            _ => return None,
        };
        Some((note.tag("e")?.to_string(), mark))
    }
}

/// The proof of a done whose `content` ends with the URL of each of its `files`: the content without
/// them.
fn proof(content: &str, files: &[ProofFile]) -> String {
    if files.is_empty() {
        return content.to_string();
    }
    let lines: Vec<&str> = content.lines().collect();
    let url = |line: &&&str| {
        files
            .iter()
            .any(|f| line.starts_with(&format!("blossom:{}.", f.sha256)))
    };
    let kept = lines.len() - lines.iter().rev().take_while(url).count();
    lines[..kept].join("\n")
}

#[cfg(test)]
mod tests;
