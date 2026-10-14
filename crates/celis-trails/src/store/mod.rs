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
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The records in `$CELIS_HOME/trails/`, and the account of `$CELIS_HOME/phrase` that signs new ones. It
/// reads the records from disk once, and keeps those it writes after. It reads and writes its
/// linked folders too.
pub struct Store {
    root: PathBuf,
    phrase: Phrase,
    links: Vec<Link>,
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
            links: vec![],
            account: OnceCell::new(),
            read: RefCell::new(None),
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
        let mut kept = read(&self.root.join("shared"));
        let came = self.take_in(&mut kept)?;
        let all: HashMap<&str, &Note> = kept.iter().map(|r| (r.id.as_str(), r)).collect();
        let mut went = 0;
        for link in &self.links {
            for record in kept.iter().filter(|r| link.holds(r, &all)) {
                went += usize::from(put(&link.folder, record)?.1);
            }
        }
        *self.read.borrow_mut() = None;
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

    /// Every record in `shared/` and in the linked folders that parses and verifies.
    fn files(&self) -> Vec<Note> {
        let mut all = read(&self.root.join("shared"));
        let _ = self.take_in(&mut all);
        all
    }

    /// Copies each record of the linked folders that `shared/` lacks into it, and adds it to `all`; the
    /// count.
    fn take_in(&self, all: &mut Vec<Note>) -> Result<usize> {
        let mut came = 0;
        for record in self.links.iter().flat_map(|l| read(&l.folder)) {
            if put(&self.root.join("shared"), &record)?.1 {
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
