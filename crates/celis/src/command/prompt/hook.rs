//! A hook's JSON, as `celis prompt` reads it.

use crate::failure::Failure;
use serde_json::Value;
use std::io::Read;

/// What a hook gives `prompt`: a session start after a compaction, a person's prompt, or nothing to
/// act on, such as a harness message or a hook with no session.
pub(super) enum Hook {
    Compaction { session: String },
    Prompt(Prompt),
    Nothing,
}

/// A person's prompt: its session, its first line, and whether it is a task for a step.
pub(super) struct Prompt {
    pub session: String,
    pub text: String,
    pub task: bool,
}

impl Hook {
    /// The hook on stdin; a usage error when it is no JSON.
    pub(super) fn read() -> Result<Hook, Failure> {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .map_err(anyhow::Error::from)?;
        let json: Value = serde_json::from_str(&input)
            .map_err(|_| Failure::usage("expected a hook's JSON on stdin"))?;
        Ok(Hook::of(&json))
    }

    /// The hook `json` holds: its session is `conversation_id`, else `session_id`; its text is the
    /// prompt's first line, cut to 200 characters; a task names a step with `celis step <id>`.
    pub(super) fn of(json: &Value) -> Hook {
        let session = json["conversation_id"]
            .as_str()
            .or(json["session_id"].as_str())
            .unwrap_or("")
            .to_string();
        if session.is_empty() {
            return Hook::Nothing;
        }
        if json["source"] == "compact" {
            return Hook::Compaction { session };
        }
        let prompt = json["prompt"].as_str().unwrap_or("");
        let first = prompt.lines().next().unwrap_or("").trim();
        let text: String = first.chars().take(200).collect();
        let notified = ["<task-notification>", "<agent-message"]
            .iter()
            .any(|t| text.starts_with(t));
        if text.is_empty() || notified {
            return Hook::Nothing;
        }
        Hook::Prompt(Prompt {
            session,
            text,
            task: names_a_step(prompt),
        })
    }
}

/// Whether `text` names a step with `celis step <id>`, its id 8 or more hex digits.
fn names_a_step(text: &str) -> bool {
    text.match_indices("celis step ").any(|(i, m)| {
        let rest = &text[i + m.len()..];
        rest.chars().take_while(char::is_ascii_hexdigit).count() >= 8
    })
}
