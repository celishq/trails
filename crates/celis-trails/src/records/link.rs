//! A folder that holds a copy of one project's records.

use crate::Project;
use anyhow::{Context, Result};
use celis_nostr::Note;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// What the README of a linked folder says to a reader without celis.
const README: &str = include_str!("link.md");

/// A folder that holds a copy of one project's records: in the repo, which git carries to other clones,
/// or anywhere else, such as a synced drive. It is on when `$CELIS_HOME/links` names it and it exists.
pub struct Link {
    pub folder: PathBuf,
    pub path: String,
    project: String,
}

impl Link {
    /// The folder a link goes to when none is named, from the project's top folder.
    pub const FOLDER: &str = ".celis/trails";

    /// The link of `project` to `place`: a path from its top folder, or a full path.
    pub fn new(project: &Project, place: &str) -> Link {
        Link {
            folder: project.folder.join(place),
            path: place.to_string(),
            project: project.id.clone(),
        }
    }

    /// The links that are on in `home`, of the project that holds `folder`.
    pub fn all(home: &Path, folder: &Path) -> Vec<Link> {
        let listed = Link::listed(home);
        if listed.is_empty() {
            return vec![];
        }
        let project = Project::at(folder);
        listed
            .iter()
            .filter(|(id, _)| *id == project.id)
            .map(|(_, place)| Link::new(&project, place))
            .filter(|link| link.folder.is_dir())
            .collect()
    }

    /// Whether the folder exists and `home` names the link.
    pub fn is_on(&self, home: &Path) -> bool {
        self.folder.is_dir() && Link::listed(home).contains(&self.entry())
    }

    /// Creates the folder and its README, and names the link in `home`.
    pub fn turn_on(&self, home: &Path) -> Result<()> {
        std::fs::create_dir_all(&self.folder).context("create the linked folder")?;
        let readme = self.folder.join("README.md");
        if !readme.exists() {
            std::fs::write(&readme, README).context("write the linked folder's README")?;
        }
        let mut listed = Link::listed(home);
        if listed.contains(&self.entry()) {
            return Ok(());
        }
        listed.push(self.entry());
        Link::save(home, &listed)
    }

    /// Takes the link out of `home`; the folder stays.
    pub fn turn_off(&self, home: &Path) -> Result<()> {
        let mut listed = Link::listed(home);
        listed.retain(|entry| *entry != self.entry());
        Link::save(home, &listed)
    }

    /// Whether the folder holds `record`, with `all` the records by id.
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

    fn entry(&self) -> (String, String) {
        (self.project.clone(), self.path.clone())
    }

    /// The project id and place of each line of `$CELIS_HOME/links`.
    fn listed(home: &Path) -> Vec<(String, String)> {
        std::fs::read_to_string(home.join("links"))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .map(|(id, place)| (id.to_string(), place.to_string()))
            .collect()
    }

    /// Writes `listed` to `$CELIS_HOME/links`, one `<project id>\t<place>` a line.
    fn save(home: &Path, listed: &[(String, String)]) -> Result<()> {
        std::fs::create_dir_all(home)?;
        let text: String = listed
            .iter()
            .map(|(id, p)| format!("{id}\t{p}\n"))
            .collect();
        std::fs::write(home.join("links"), text)?;
        Ok(())
    }
}
