//! `celis hook`: what an agent's hook reads, in the shape the agent reads it.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::PatternStore;
use celis_trails::TrailStore;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

/// The agents whose hooks run `celis hook`.
const AGENTS: [&str; 5] = ["claude", "codex", "cursor", "gemini", "antigravity"];

/// The start of the title of the ask that `/celis:tidy` adds.
const TIDY: &str = "Tidy the celis trails";

/// How many days after a tidy the session start asks to offer the next.
const TIDY_DAYS: u64 = 7;

impl Command {
    /// Acts on the hook's JSON on stdin for `hook start <agent> <plugin folder>` or `hook ask <agent>`,
    /// and prints what the agent reads. Only a usage error fails.
    pub(crate) fn hook(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let words: Vec<&str> = args.words.iter().map(String::as_str).collect();
        let (event, agent, root) =
            match words[..] {
                [_, "start", agent, root] => ("start", agent, Some(Path::new(root))),
                [_, "ask", agent] => ("ask", agent, None),
                _ => return Err(Failure::usage(
                    "expected: celis hook start <agent> <plugin folder>, or celis hook ask <agent>",
                )),
            };
        if !AGENTS.contains(&agent) {
            return Err(Failure::usage(format!(
                "hook takes claude, codex, cursor, gemini or antigravity, not {agent}"
            )));
        }
        let mut input = String::new();
        let _ = std::io::stdin().read_to_string(&mut input);
        let json: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
        let text = match root {
            Some(root) => self.start(agent, &json, root),
            None => self.note(&json),
        };
        match (agent, event) {
            ("cursor", "ask") => crate::command::put(out, r#"{"continue": true}"#),
            _ if text.is_empty() => Ok(()),
            (_, "start") => crate::command::put(out, &shape(agent, "SessionStart", &text)),
            _ => crate::command::put(out, &shape(agent, "BeforeAgent", &text)),
        }
    }

    /// The card, where the plugin's files are, the index of the patterns in the project the JSON names,
    /// and a version notice.
    fn start(&self, agent: &str, json: &Value, root: &Path) -> String {
        let card = std::fs::read_to_string(root.join("card.md")).unwrap_or_default();
        let mut text = format!(
            "How to do an ask with the celis plugin:\n{}\n\n",
            card.trim_end()
        );
        if agent != "claude" {
            let session = ["conversation_id", "conversationId", "session_id"]
                .iter()
                .find_map(|key| json[key].as_str())
                .unwrap_or("");
            text.push_str(&format!(
                "celis session: {session}. Pass --session {session} to each celis step add.\n"
            ));
        }
        text.push_str(&format!("steps.md: {}", root.join("steps.md").display()));
        if agent != "claude" {
            let judge = root.join("agents/judge.md").display().to_string();
            text.push_str(&format!(
                "\njudge.md: {judge}. With no subagents, do what it says yourself, with the brief \
                 that done prints."
            ));
            let lesson = root.join("agents/lesson.md").display().to_string();
            text.push_str(&format!(
                "\nlesson.md: {lesson}. With no subagents, do what it says yourself, with the brief \
                 that celis lesson prints."
            ));
            let recall = root.join("agents/recall.md").display().to_string();
            text.push_str(&format!(
                "\nrecall.md: {recall}. With no subagents, do what it says yourself, with the ask \
                 and its files."
            ));
        }
        if let Some(index) = self.here(json).index(&[]).ok().flatten() {
            text.push_str(&format!("\n\nThe patterns in scope here:\n{index}"));
        }
        if let Some(line) = self.tidy() {
            text.push_str(&format!("\n\n{line}"));
        }
        let manifest = std::fs::read_to_string(root.join(".claude-plugin/plugin.json"));
        let plugin = manifest
            .ok()
            .and_then(|m| serde_json::from_str::<Value>(&m).ok())
            .and_then(|m| m["version"].as_str().map(str::to_string))
            .unwrap_or_default();
        if let Some(notice) = notice(crate::VERSION, &plugin) {
            text.push_str(&format!("\n\n{notice}"));
        }
        text
    }

    /// The line that asks to offer `/celis:tidy` once in the session: when the store holds a live pattern,
    /// and no step whose title starts with `TIDY` began in the last `TIDY_DAYS` days.
    fn tidy(&self) -> Option<String> {
        if !self.held().ok()? {
            return None;
        }
        let trails = self.trails().ok()?;
        if !trails.patterns().every().iter().any(|e| e.is_live()) {
            return None;
        }
        let steps = trails.steps();
        let last = steps
            .all()
            .into_iter()
            .filter(|s| s.title().starts_with(TIDY))
            .map(|s| s.note().created_at)
            .max();
        let days = last.map(|at| crate::command::now().saturating_sub(at) / 86_400);
        let since = match days {
            Some(days) if days < TIDY_DAYS => return None,
            Some(days) => format!("the last tidy was {days} days ago"),
            None => "no tidy has run yet".to_string(),
        };
        Some(format!(
            "celis: {since}; once in this session, after the person's first ask, offer them /celis:tidy."
        ))
    }

    /// The note on the prompt, after it is acted on in the project the JSON names.
    fn note(&self, json: &Value) -> String {
        let mut note = Vec::new();
        if self.here(json).act_on(json, &mut note).is_err() {
            return String::new();
        }
        String::from_utf8_lossy(&note).trim_end().to_string()
    }

    /// This run in the folder the JSON names in `cwd`, else in the first of `workspace_roots` or of
    /// `workspacePaths`, else in its own.
    fn here(&self, json: &Value) -> Command {
        let folder = json["cwd"]
            .as_str()
            .or(json["workspace_roots"][0].as_str())
            .or(json["workspacePaths"][0].as_str())
            .map_or(self.folder.clone(), PathBuf::from);
        Command {
            args: vec![],
            env: self.env.clone(),
            folder,
        }
    }
}

/// `text` in the shape `agent` reads on its hook `event`.
fn shape(agent: &str, event: &str, text: &str) -> String {
    match agent {
        "cursor" => json!({"additional_context": text}).to_string(),
        "gemini" => {
            json!({"hookSpecificOutput": {"hookEventName": event, "additionalContext": text}})
                .to_string()
        }
        "antigravity" => json!({"injectSteps": [{"ephemeralMessage": text}]}).to_string(),
        _ => text.to_string(),
    }
}

/// The notice that `celis` and the plugin differ in version; none when they match or one is a build
/// from the repo.
fn notice(installed: &str, plugin: &str) -> Option<String> {
    let unknown = |v: &str| v.is_empty() || v == "0.0.0";
    if unknown(installed) || unknown(plugin) || installed == plugin {
        return None;
    }
    Some(format!(
        "celis {installed} is installed, and the celis plugin is {plugin}. Tell the person to update the \
         older one: the command with `celis update`, the plugin with their agent's plugin update."
    ))
}
