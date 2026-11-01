//! `celis setup`: the brief that adds the plugin to each agent on this machine.

use crate::command::{Args, Command};
use crate::failure::Failure;
use serde_json::json;
use std::io::Write;
use std::path::Path;

/// Where a person reads how to add the plugin by hand.
const STEPS: &str = "https://github.com/celishq/trails#install";

/// Each agent: its name for a person, its command on PATH, the commands that add
/// the plugin, and what only the person can do after.
const PLUGINS: [(&str, &str, &[&str], &str); 5] = [
    (
        "Claude Code",
        "claude",
        &[
            "claude plugin marketplace add celishq/trails",
            "claude plugin install celis@celis",
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
    /// Prints, for each agent on PATH, the commands that add the plugin and what the person does after.
    /// Changes nothing.
    pub(crate) fn setup(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let found: Vec<_> = PLUGINS
            .iter()
            .filter(|(_, bin, _, _)| self.on_path(bin))
            .collect();
        let mut lines = vec![format!("{} is installed.", crate::version()), String::new()];
        match found.is_empty() {
            true => lines.push(format!(
                "No agent that celis setup knows is on PATH. Add the plugin to your agent as {STEPS} shows."
            )),
            false => lines.extend(brief(&found)),
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

/// The commands that add the plugin to each of `found`, and what the person does after.
fn brief(found: &[&(&str, &str, &[&str], &str)]) -> Vec<String> {
    let mut lines = vec!["Run each command, and show the person each one first:".to_string()];
    for (_, _, commands, _) in found {
        lines.extend(commands.iter().map(|c| format!("  {c}")));
    }
    lines.push(String::new());
    lines.push("Then tell the person:".to_string());
    lines.push("  - Restart each agent above, so that the plugin loads.".to_string());
    for (_, _, _, after) in found {
        if !after.is_empty() {
            lines.push(format!("  - {after}"));
        }
    }
    lines.push(
        "  - In the first session, celis offers to learn patterns from your past chats."
            .to_string(),
    );
    lines.push(String::new());
    lines.push(
        "After the restart, the agent's first answer starts with the celis status line, which shows \
         that the plugin works."
            .to_string(),
    );
    lines
}
