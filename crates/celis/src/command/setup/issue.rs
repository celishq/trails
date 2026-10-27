//! `celis issue`: what went wrong, with the facts of this machine, as Markdown for the celis team.

use crate::command::read::report::markdown;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_store::records::id::short;
use celis_store::{Home, Phrase};
use celis_trails::{Steps, TrailStore};
use serde_json::json;
use std::io::Write;

/// The lines of the log an issue shows, at most.
const LOG: usize = 50;

impl Command {
    /// Prints the text `args` gives, the facts, the end of the log and the trails its ids name. Writes no
    /// record.
    pub(crate) fn issue(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let home = self.home()?;
        let text = args.text.clone().unwrap_or_default();
        let mut lines = text.trim().lines();
        let mut parts = vec![format!("## {}", lines.next().unwrap_or("celis issue"))];
        let rest: Vec<&str> = lines.collect();
        if !rest.is_empty() {
            parts.push(rest.join("\n").trim().to_string());
        }
        parts.push(self.facts()?);
        let log = std::fs::read_to_string(home.join("log")).unwrap_or_default();
        let log: Vec<&str> = log.lines().collect();
        if !log.is_empty() {
            let end = log[log.len().saturating_sub(LOG)..].join("\n");
            parts.push(format!("**Log**\n\n```\n{end}\n```"));
        }
        let ids = &args.words[1..];
        if !ids.is_empty() {
            if !self.held()? {
                return Err(Failure::no_step(&ids[0]));
            }
            let trails = self.trails()?;
            let text = markdown(&trails.steps(), &trails.notes(), ids)?;
            parts.push(text.trim_end().to_string());
        }
        let text = parts.join("\n\n");
        crate::command::show(out, args.json, "issue", json!(text), &text)
    }

    /// The facts of this machine: version, system, sessions, project, home, phrase and records.
    fn facts(&self) -> Result<String, Failure> {
        let home = self.home()?;
        let sessions: Vec<&str> = [
            "CLAUDE_CODE_SESSION_ID",
            "CODEX_THREAD_ID",
            "CELIS_SESSION_ID",
        ]
        .into_iter()
        .filter(|name| self.env.get(*name).is_some_and(|v| !v.is_empty()))
        .collect();
        let sessions = match sessions.is_empty() {
            true => "none".to_string(),
            false => sessions.join(", "),
        };
        let project = self.project();
        let shown = match self.env.get("HOME").map(|h| home.strip_prefix(h)) {
            Some(Ok(rest)) => format!("~/{}", rest.display()),
            _ => home.display().to_string(),
        };
        let phrase = match Phrase::at(&home).path().exists() {
            true => "kept",
            false => "missing",
        };
        let files = std::fs::read_dir(Home::root(&home).join("shared"))
            .into_iter()
            .flatten()
            .filter(|e| {
                e.as_ref()
                    .is_ok_and(|e| e.path().extension().is_some_and(|x| x == "json"))
            })
            .count();
        let (read, steps) = match self.held()? {
            true => {
                let records = Home::open(&home)?.records();
                (records.len(), Steps::new(&records, 0).all().len())
            }
            false => (0, 0),
        };
        Ok([
            "**Facts**".to_string(),
            String::new(),
            format!("- version: {}", crate::version()),
            format!(
                "- system: {} {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            ),
            format!("- sessions: {sessions}"),
            format!(
                "- project: {} `{}`",
                project.anchor.name,
                short(&project.anchor.project)
            ),
            format!("- home: `{shown}`"),
            format!("- phrase: {phrase}"),
            format!("- records: {files} files, {read} read, {steps} steps"),
        ]
        .join("\n"))
    }
}
