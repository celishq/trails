//! The records of every project on this machine.

pub mod anchor;
pub mod link;
pub mod phrase;
pub mod project;

use crate::{Failure, Phrase};
use anyhow::{Context, Result};
use celis_nostr::{Account, Note};
use link::Link;
use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The records in `$CELIS_HOME/trails/`, and the account of `$CELIS_HOME/phrase` that signs new ones. It
/// reads the records from disk once, keeps those it writes after, and the names of the files of
/// `shared/` it holds. It reads and writes its linked folders too.
pub struct Store {
    root: PathBuf,
    phrase: Phrase,
    links: Vec<Link>,
    account: OnceCell<Account>,
    read: RefCell<Option<Vec<Note>>>,
    seen: RefCell<HashSet<String>>,
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
            links: vec![],
            account: OnceCell::new(),
            read: RefCell::new(None),
            seen: RefCell::default(),
        })
    }

    /// The store that also reads and writes the folder of `link`.
    pub fn with(mut self, link: Link) -> Store {
        self.links.push(link);
        self
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

    /// Writes `record` once to `shared/`, and to each linked folder that holds it.
    pub fn write(&self, record: &Note) -> Result<PathBuf> {
        let (path, new) = put(&self.root.join("shared"), record)?;
        if new {
            if let Some(read) = self.read.borrow_mut().as_mut() {
                read.push(record.clone());
                self.seen.borrow_mut().insert(name(record));
            }
        }
        if !self.links.is_empty() {
            let records = self.records();
            let all = records.iter().map(|r| (r.id.as_str(), r)).collect();
            for link in self.links.iter().filter(|l| l.holds(record, &all)) {
                put(&link.folder, record)?;
            }
        }
        Ok(path)
    }

    /// Copies each record a linked folder holds and lacks into it, and each record of a linked folder
    /// the store lacks into `shared/`. Returns the counts copied out and in.
    pub fn sync(&self) -> Result<(usize, usize)> {
        let mut kept = read(&self.root.join("shared"), |_| true);
        let came = self.take_in(&mut kept)?;
        let all: HashMap<&str, &Note> = kept.iter().map(|r| (r.id.as_str(), r)).collect();
        let mut went = 0;
        for link in &self.links {
            for record in kept.iter().filter(|r| link.holds(r, &all)) {
                went += usize::from(put(&link.folder, record)?.1);
            }
        }
        *self.read.borrow_mut() = None;
        self.seen.borrow_mut().clear();
        Ok((went, came))
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

    /// The records in `shared/` written since the last read, that parse and verify; they join the
    /// records. None before the first read.
    pub fn fresh(&self) -> Vec<Note> {
        let mut read = self.read.borrow_mut();
        let Some(read) = read.as_mut() else {
            return Vec::new();
        };
        let fresh = self.unseen();
        read.extend(fresh.iter().cloned());
        fresh
    }

    /// Every record in `shared/` and in the linked folders that parses and verifies.
    fn files(&self) -> Vec<Note> {
        let mut all = self.unseen();
        let _ = self.take_in(&mut all);
        all
    }

    /// Every record in `shared/` of a file not read before, that parses and verifies.
    fn unseen(&self) -> Vec<Note> {
        let mut seen = self.seen.borrow_mut();
        let fresh = read(&self.root.join("shared"), |file| !seen.contains(file));
        seen.extend(fresh.iter().map(name));
        fresh
    }

    /// Copies each record of the linked folders that `shared/` lacks into it, and adds it to `all`; the
    /// count.
    fn take_in(&self, all: &mut Vec<Note>) -> Result<usize> {
        let mut came = 0;
        for record in self.links.iter().flat_map(|l| read(&l.folder, |_| true)) {
            if put(&self.root.join("shared"), &record)?.1 {
                self.seen.borrow_mut().insert(name(&record));
                all.push(record);
                came += 1;
            }
        }
        Ok(came)
    }
}

/// Writes `record` to `folder` as `<created_at>-<id>.json`, unless it is there; the path, and whether it
/// wrote it.
fn put(folder: &Path, record: &Note) -> Result<(PathBuf, bool)> {
    let name = name(record);
    let path = folder.join(&name);
    if path.exists() {
        return Ok((path, false));
    }
    let temp = folder.join(format!(".{name}.{}", std::process::id()));
    std::fs::write(&temp, serde_json::to_string(record)?)?;
    std::fs::rename(&temp, &path)?;
    Ok((path, true))
}

/// The name of the file of `record`: `<created_at>-<id>.json`.
fn name(record: &Note) -> String {
    format!("{}-{}.json", record.created_at, record.id)
}

/// Every record in `folder` of a file `wanted` names, that parses and verifies.
fn read(folder: &Path, wanted: impl Fn(&str) -> bool) -> Vec<Note> {
    std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let named = path.extension().is_some_and(|x| x == "json");
            let file = path.file_name()?.to_str()?;
            if !named || !wanted(file) {
                return None;
            }
            let text = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str::<Note>(&text).ok()
        })
        .filter(Note::verifies)
        .collect()
}

#[cfg(test)]
mod tests;
