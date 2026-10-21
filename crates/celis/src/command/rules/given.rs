//! The log of the rules given in each session.

use crate::failure::Failure;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

/// `$CELIS_HOME/given`: one JSON line for each rule given in a session, and for each compaction of a
/// session.
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

    /// The ids of the rules given in `session` after its last compaction.
    pub fn of(&self, session: &str) -> HashSet<String> {
        let text = std::fs::read_to_string(&self.path).unwrap_or_default();
        let mut ids = HashSet::new();
        let lines = text
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok());
        for line in lines.filter(|l| l["session"] == session && l["miss"] != true) {
            match (line["compact"] == true, line["rule"].as_str()) {
                (true, _) => ids.clear(),
                (false, Some(rule)) => {
                    ids.insert(rule.to_string());
                }
                _ => {}
            }
        }
        ids
    }

    /// Logs each rule of `rules` as given in `session` at `at`.
    pub fn give(&self, session: &str, rules: &[&str], at: u64) -> Result<(), Failure> {
        let lines = rules
            .iter()
            .map(|r| json!({"at": at, "session": session, "rule": r}));
        self.add(lines.collect())
    }

    /// Logs a compaction of `session` at `at`.
    pub fn compact(&self, session: &str, at: u64) -> Result<(), Failure> {
        self.add(vec![json!({"at": at, "session": session, "compact": true})])
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
