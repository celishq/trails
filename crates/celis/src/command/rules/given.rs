//! The log of the rules given in each session.

use crate::failure::Failure;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

/// `$CELIS_HOME/given`: one JSON line for each rule given in a session, and each miss.
pub(crate) struct Given {
    path: PathBuf,
}

impl Given {
    /// The log in the home `home`.
    pub fn new(home: &Path) -> Given {
        Given {
            path: home.join("given"),
        }
    }

    /// The ids of the rules given in `session`.
    pub fn of(&self, session: &str) -> HashSet<String> {
        let lines = self.lines().into_iter();
        let lines = lines.filter(|l| l["session"] == session && l["miss"] != true);
        lines
            .filter_map(|l| Some(l["rule"].as_str()?.to_string()))
            .collect()
    }

    /// The ids of the rules a miss names.
    pub fn misses(&self) -> HashSet<String> {
        let lines = self.lines().into_iter().filter(|l| l["miss"] == true);
        lines
            .filter_map(|l| Some(l["rule"].as_str()?.to_string()))
            .collect()
    }

    /// The ids of the rules given while each step worked, by step id, in the order given, each once.
    pub fn steps(&self) -> HashMap<String, Vec<String>> {
        let mut steps: HashMap<String, Vec<String>> = HashMap::new();
        for line in self.lines().into_iter().filter(|l| l["miss"] != true) {
            if let (Some(step), Some(rule)) = (line["step"].as_str(), line["rule"].as_str()) {
                let rules = steps.entry(step.to_string()).or_default();
                if !rules.iter().any(|r| r == rule) {
                    rules.push(rule.to_string());
                }
            }
        }
        steps
    }

    /// Logs each rule of `rules` as given in `session` at `at`, while `step` worked.
    pub fn give(
        &self,
        session: &str,
        rules: &[&str],
        step: Option<&str>,
        at: u64,
    ) -> Result<(), Failure> {
        let lines = rules.iter().map(|r| {
            let mut line = json!({"at": at, "session": session, "rule": r});
            if let Some(step) = step {
                line["step"] = json!(step);
            }
            line
        });
        self.add(lines.collect())
    }

    /// Logs a miss of the rule `rule` in `session` at `at`.
    pub fn miss(&self, session: &str, rule: &str, at: u64) -> Result<(), Failure> {
        self.add(vec![
            json!({"at": at, "session": session, "rule": rule, "miss": true}),
        ])
    }

    /// Each line of the log that is JSON.
    fn lines(&self) -> Vec<Value> {
        let text = std::fs::read_to_string(&self.path).unwrap_or_default();
        let lines = text.lines();
        lines.filter_map(|l| serde_json::from_str(l).ok()).collect()
    }

    /// Adds `lines` to the end of the log, and makes its folder.
    fn add(&self, lines: Vec<Value>) -> Result<(), Failure> {
        if lines.is_empty() {
            return Ok(());
        }
        let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
        if let Some(folder) = self.path.parent() {
            std::fs::create_dir_all(folder).map_err(anyhow::Error::from)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(anyhow::Error::from)?;
        file.write_all(text.as_bytes())
            .map_err(anyhow::Error::from)?;
        Ok(())
    }
}
