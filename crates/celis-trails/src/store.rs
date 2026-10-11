//! The records of every project on this machine.

use crate::{Failure, Phrase};
use anyhow::{Context, Result};
use celis_wire::{Account, Note};
use std::cell::{OnceCell, RefCell};
use std::path::{Path, PathBuf};

/// The records in `$CELIS_HOME/trails/`, and the account of `$CELIS_HOME/phrase` that signs new ones. It
/// reads the records from disk once, and keeps those it writes after.
pub struct Store {
    root: PathBuf,
    phrase: Phrase,
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
            account: OnceCell::new(),
            read: RefCell::new(None),
        })
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

    /// Writes `record` once to `shared/`.
    pub fn write(&self, record: &Note) -> Result<PathBuf> {
        let folder = self.root.join("shared");
        let name = format!("{}-{}.json", record.created_at, record.id);
        let path = folder.join(&name);
        if !path.exists() {
            let temp = folder.join(format!(".{name}.{}", std::process::id()));
            std::fs::write(&temp, serde_json::to_string(record)?)?;
            std::fs::rename(&temp, &path)?;
            if let Some(read) = self.read.borrow_mut().as_mut() {
                read.push(record.clone());
            }
        }
        Ok(path)
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

    /// Every record in `shared/` that parses and verifies.
    fn files(&self) -> Vec<Note> {
        std::fs::read_dir(self.root.join("shared"))
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
}
