//! A project's copy of its records, in the project's own folder.

use crate::Project;
use anyhow::{Context, Result};
use celis_nostr::Note;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// What `.celis/README.md` says to a reader without celis.
const README: &str = include_str!("mirror.md");

/// The records of one project in `<top folder>/.celis/trails/`, which git carries to other clones. It is
/// on when `$CELIS_HOME/mirrors` names the project and the folder exists.
pub struct Mirror {
    pub folder: PathBuf,
    project: String,
}

impl Mirror {
    /// The mirror of `project`, on or not.
    pub fn of(project: &Project) -> Mirror {
        Mirror {
            folder: project.folder.join(".celis").join("trails"),
            project: project.id.clone(),
        }
    }

    /// The mirror of the project that holds `folder`, when it is on in `home`.
    pub fn on(home: &Path, folder: &Path) -> Option<Mirror> {
        if Mirror::listed(home).is_empty() {
            return None;
        }
        let mirror = Mirror::of(&Project::at(folder));
        mirror.is_on(home).then_some(mirror)
    }

    /// Whether the folder exists and `home` names the project.
    pub fn is_on(&self, home: &Path) -> bool {
        self.folder.is_dir() && Mirror::listed(home).contains(&self.project)
    }

    /// Creates the folder and its README, and names the project in `home`.
    pub fn turn_on(&self, home: &Path) -> Result<()> {
        std::fs::create_dir_all(&self.folder).context("create the mirror")?;
        let readme = self.folder.with_file_name("README.md");
        if !readme.exists() {
            std::fs::write(&readme, README).context("write the mirror's README")?;
        }
        let mut listed = Mirror::listed(home);
        if !listed.contains(&self.project) {
            listed.push(self.project.clone());
            std::fs::create_dir_all(home)?;
            std::fs::write(home.join("mirrors"), listed.join("\n") + "\n")?;
        }
        Ok(())
    }

    /// Takes the project out of `home`; the folder stays.
    pub fn turn_off(&self, home: &Path) -> Result<()> {
        let listed = Mirror::listed(home);
        if !listed.contains(&self.project) {
            return Ok(());
        }
        let kept: Vec<String> = listed.into_iter().filter(|p| *p != self.project).collect();
        let text = kept.iter().map(|p| format!("{p}\n")).collect::<String>();
        std::fs::write(home.join("mirrors"), text)?;
        Ok(())
    }

    /// Whether the mirror holds `record`, with `all` the records by id.
    pub fn holds(&self, record: &Note, all: &HashMap<&str, &Note>) -> bool {
        if let Some(anchor) = record.tag("i") {
            return anchor == format!("project:{}", self.project)
                && record.tag("scope") != Some("me");
        }
        record
            .tags
            .iter()
            .filter(|t| t.first().is_some_and(|n| n == "e"))
            .filter_map(|t| all.get(t.get(1)?.as_str()))
            .any(|named| self.holds(named, all))
    }

    /// The project ids `$CELIS_HOME/mirrors` names.
    fn listed(home: &Path) -> Vec<String> {
        std::fs::read_to_string(home.join("mirrors"))
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    }
}
