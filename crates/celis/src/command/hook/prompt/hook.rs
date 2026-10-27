//! A hook's JSON, as `celis hook` reads it.

use serde_json::Value;

/// A person's prompt: its session, its first line, and whether it is a task for a step.
pub(super) struct Prompt {
    pub session: String,
    pub text: String,
    pub task: bool,
}

impl Prompt {
    /// The prompt `json` holds: its session is `conversation_id`, else `session_id`; its text is the
    /// prompt's first line, cut to 200 characters; a task names a step with `celis step <id>`. None
    /// for a session start, a harness message or a hook with no session.
    pub(super) fn of(json: &Value) -> Option<Prompt> {
        let session = json["conversation_id"]
            .as_str()
            .or(json["session_id"].as_str())
            .unwrap_or("")
            .to_string();
        if session.is_empty() || json["source"].is_string() {
            return None;
        }
        let prompt = json["prompt"].as_str().unwrap_or("");
        let first = prompt.lines().next().unwrap_or("").trim();
        let text: String = first.chars().take(200).collect();
        let notified = ["<task-notification>", "<agent-message"]
            .iter()
            .any(|t| text.starts_with(t));
        if text.is_empty() || notified {
            return None;
        }
        Some(Prompt {
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
