//! `celis setup`: the brief that adds the plugin to each agent on this machine.

use crate::command::hook::{AGENTS, STARTED};
use crate::command::{Args, Command};
use crate::failure::Failure;
use serde_json::json;
use std::io::Write;
use std::path::Path;

/// Where a person reads how to add the plugin by hand.
const STEPS: &str = "https://github.com/celishq/trails#install";

/// Each agent of `AGENTS` in order: its name for a person, its command on PATH, the commands that add
/// the plugin, and what only the person can do after.
const PLUGINS: [(&str, &str, &[&str], &str); 5] = [
    (
        "Claude Code",
        "claude",
        &[
            "claude plugin marketplace add celishq/trails",
            "claude plugin install celis@celis",
            "claude plugin install celis-ui@celis",
        ],
        "",
    ),
    (
        "Codex",
        "codex",
        &[
            "codex plugin marketplace add celishq/trails",
            "codex plugin add celis@celis",
        ],
        "In Codex, trust the celis hooks with /hooks.",
    ),
    (
        "Cursor",
        "cursor-agent",
        &["cursor-agent plugin marketplace add https://github.com/celishq/trails"],
        "In Cursor, install celis from /plugins.",
    ),
    (
        "Gemini CLI",
        "gemini",
        &["gemini extensions install https://github.com/celishq/trails"],
        "",
    ),
    (
        "Antigravity",
        "agy",
        &["agy plugin install https://github.com/celishq/trails"],
        "",
    ),
];

impl Command {
    /// Prints, for each agent on PATH, whether its plugin started a session, and for those that did not,
    /// the commands that add the plugin and what the person does after. Changes nothing.
    pub(crate) fn setup(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let started = self.home()?.join(STARTED);
        let found: Vec<_> = AGENTS
            .iter()
            .zip(PLUGINS)
            .filter(|(_, (_, bin, _, _))| self.on_path(bin))
            .map(|(agent, plugin)| (started.join(agent).exists(), plugin))
            .collect();
        let mut lines = vec![format!("{} is installed.", crate::version()), String::new()];
        if found.is_empty() {
            let names: Vec<String> = PLUGINS
                .iter()
                .map(|(name, bin, _, _)| format!("{name} ({bin})"))
                .collect();
            lines.push(format!(
                "No agent that celis setup knows is on PATH: {}. Add the plugin to your agent as {STEPS} shows.",
                names.join(", ")
            ));
        } else {
            lines.push("Agents on this machine:".to_string());
            for (set, (name, _, _, _)) in &found {
                let state = match set {
                    true => "celis ran in a session",
                    false => "no session with celis yet",
                };
                lines.push(format!("  {name:<12} {state}"));
            }
            lines.push(String::new());
            let missing: Vec<_> = found
                .iter()
                .filter(|(set, _)| !set)
                .map(|(_, p)| p)
                .collect();
            match missing.is_empty() {
                true => lines.push("All set.".to_string()),
                false => lines.extend(brief(&missing)),
            }
        }
        let text = lines.join("\n");
        crate::command::show(out, args.json, "setup", json!(text), &text)
    }

    /// Whether a file named `bin` is in a folder of `$PATH`.
    fn on_path(&self, bin: &str) -> bool {
        let path = self.env.get("PATH").map_or("", String::as_str);
        std::env::split_paths(path).any(|folder| Path::new(&folder).join(bin).is_file())
    }
}

/// The commands that add the plugin to each of `missing`, and what the person does after.
fn brief(missing: &[&(&str, &str, &[&str], &str)]) -> Vec<String> {
    let mut lines = vec!["Run each command, and show the person each one first:".to_string()];
    for (_, _, commands, _) in missing {
        lines.extend(commands.iter().map(|c| format!("  {c}")));
    }
    let names: Vec<&str> = missing.iter().map(|(name, _, _, _)| *name).collect();
    let names = match names.split_last() {
        Some((last, [])) => last.to_string(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    };
    lines.push(String::new());
    lines.push("Then tell the person:".to_string());
    lines.push(format!("  - Restart {names}, so that the plugin loads."));
    for (_, _, _, after) in missing {
        if !after.is_empty() {
            lines.push(format!("  - {after}"));
        }
    }
    lines.push(
        "  - In the first session, celis offers to learn patterns from your past chats."
            .to_string(),
    );
    lines.push(String::new());
    lines.push("When the person has restarted, run celis setup again to check.".to_string());
    lines
}
