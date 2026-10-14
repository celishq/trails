//! The records of every project on this machine.

pub mod anchor;
pub mod mirror;
pub mod phrase;
pub mod project;

use crate::{Failure, Phrase};
use anyhow::{Context, Result};
use celis_nostr::{Account, Note};
use mirror::Mirror;
use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The records in `$CELIS_HOME/trails/`, and the account of `$CELIS_HOME/phrase` that signs new ones. It
/// reads the records from disk once, and keeps those it writes after. With a mirror, it reads and
/// writes the mirror too.
pub struct Store {
    root: PathBuf,
    phrase: Phrase,
    mirror: Option<Mirror>,
    account: OnceCell<Account>,
    read: RefCell<Option<Vec<Note>>>,
}

impl Store {
    /// Whether `home` holds a store.
    pub fn exists(home: &Path) -> bool {
        Store::root(home).is_dir()
    }

    /// The store in `home`, created when it is missing.
    pub fn open(home: &Path) -> Result<Store, Failure> {
        let root = Store::root(home);
        std::fs::create_dir_all(root.join("shared")).context("create the store")?;
        Ok(Store {
            root,
            phrase: Phrase::at(home),
            mirror: None,
            account: OnceCell::new(),
            read: RefCell::new(None),
        })
    }

    /// The store that also reads and writes `mirror`.
    pub fn with(self, mirror: Mirror) -> Store {
        Store {
            mirror: Some(mirror),
            ..self
        }
    }

    /// The folder of the store in `home`.
    pub fn root(home: &Path) -> PathBuf {
        home.join("trails")
    }

    /// The account that signs new records, its phrase minted on first use.
    pub fn account(&self) -> Result<&Account> {
        if let Some(account) = self.account.get() {
            return Ok(account);
        }
        let account = self.phrase.identity()?.account().clone();
        Ok(self.account.get_or_init(|| account))
    }

    /// Writes `record` once to `shared/`, and to the mirror when it holds the record.
    pub fn write(&self, record: &Note) -> Result<PathBuf> {
        let (path, new) = put(&self.root.join("shared"), record)?;
        if new {
            if let Some(read) = self.read.borrow_mut().as_mut() {
                read.push(record.clone());
            }
        }
        if let Some(mirror) = &self.mirror {
            let records = self.records();
            let all = records.iter().map(|r| (r.id.as_str(), r)).collect();
            if mirror.holds(record, &all) {
                put(&mirror.folder, record)?;
            }
        }
        Ok(path)
    }

    /// Copies each record the mirror holds that it lacks into it, and each record of the mirror the
    /// store lacks into `shared/`. Returns the counts copied out and in.
    pub fn sync(&self) -> Result<(usize, usize)> {
        let Some(mirror) = &self.mirror else {
            return Ok((0, 0));
        };
        let shared = self.root.join("shared");
        let mut kept = read(&shared);
        let held = read(&mirror.folder);
        let known: HashSet<String> = kept.iter().map(|r| r.id.clone()).collect();
        let mirrored: HashSet<String> = held.iter().map(|r| r.id.clone()).collect();
        let came: Vec<Note> = held
            .into_iter()
            .filter(|r| !known.contains(&r.id))
            .collect();
        for record in &came {
            put(&shared, record)?;
        }
        kept.extend(came.iter().cloned());
        let all: HashMap<&str, &Note> = kept.iter().map(|r| (r.id.as_str(), r)).collect();
        let mut went = 0;
        for record in kept.iter().filter(|r| !mirrored.contains(&r.id)) {
            if mirror.holds(record, &all) {
                put(&mirror.folder, record)?;
                went += 1;
            }
        }
        *self.read.borrow_mut() = None;
        Ok((went, came.len()))
    }

    /// Every record that parses and verifies, whatever account signed it.
    pub fn records(&self) -> Vec<Note> {
        let mut all = self
            .read
            .borrow_mut()
            .get_or_insert_with(|| self.files())
            .clone();
        all.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        all
    }

    /// Every record in `shared/` and in the mirror that parses and verifies; it copies those of the
    /// mirror into `shared/`.
    fn files(&self) -> Vec<Note> {
        let shared = self.root.join("shared");
        let mut all = read(&shared);
        if let Some(mirror) = &self.mirror {
            let known: HashSet<String> = all.iter().map(|r| r.id.clone()).collect();
            for record in read(&mirror.folder) {
                if !known.contains(&record.id) {
                    let _ = put(&shared, &record);
                    all.push(record);
                }
            }
        }
        all
    }
}

/// Writes `record` to `folder` as `<created_at>-<id>.json`, unless it is there; the path, and whether it
/// wrote it.
fn put(folder: &Path, record: &Note) -> Result<(PathBuf, bool)> {
    let name = format!("{}-{}.json", record.created_at, record.id);
    let path = folder.join(&name);
    if path.exists() {
        return Ok((path, false));
    }
    let temp = folder.join(format!(".{name}.{}", std::process::id()));
    std::fs::write(&temp, serde_json::to_string(record)?)?;
    std::fs::rename(&temp, &path)?;
    Ok((path, true))
}

/// Every record in `folder` that parses and verifies.
fn read(folder: &Path) -> Vec<Note> {
    std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let named = path.extension().is_some_and(|x| x == "json");
            let text = std::fs::read_to_string(path).ok().filter(|_| named)?;
            serde_json::from_str::<Note>(&text).ok()
        })
        .filter(Note::verifies)
        .collect()
}
