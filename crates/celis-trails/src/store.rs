//! The records of every project on this machine.

use crate::Failure;
use anyhow::{anyhow, Context, Result};
use celis_wire::{Account, Note};
use std::cell::RefCell;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The records in `$CELIS_HOME/trails/`, and the account of `$CELIS_HOME/key` that signs new ones. It
/// reads the records from disk once, and keeps those it writes after.
pub struct Store {
    root: PathBuf,
    account: Account,
    read: RefCell<Option<Vec<Note>>>,
}

impl Store {
    /// Whether `home` holds a store.
    pub fn exists(home: &Path) -> bool {
        Store::root(home).is_dir()
    }

    /// The store in `home`, created with the account's key when either is missing.
    pub fn open(home: &Path) -> Result<Store, Failure> {
        let root = Store::root(home);
        std::fs::create_dir_all(root.join("shared")).context("create the store")?;
        let account = Store::key(&home.join("key"))?;
        Ok(Store {
            root,
            account,
            read: RefCell::new(None),
        })
    }

    /// The folder of the store in `home`.
    pub fn root(home: &Path) -> PathBuf {
        home.join("trails")
    }

    pub fn account(&self) -> &Account {
        &self.account
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

    /// Every record that counts: it parses and verifies, and the account signed it.
    pub fn records(&self) -> Vec<Note> {
        let mut all = self
            .read
            .borrow_mut()
            .get_or_insert_with(|| self.files())
            .clone();
        all.retain(|n| n.pubkey == self.account.pubkey());
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

    /// The account in the key at `path`, minted on first use. Racing mints end with one key.
    fn key(path: &Path) -> Result<Account> {
        if !path.exists() {
            let account = Account::generate()?;
            let hex: String = account
                .secret()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            let temp = path.with_extension(format!(
                "{}.{}.tmp",
                std::process::id(),
                OPENS.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temp)?
                .write_all(format!("{hex}\n").as_bytes())?;
            let linked = std::fs::hard_link(&temp, path);
            std::fs::remove_file(&temp)?;
            match linked {
                Ok(()) => return Ok(account),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        Store::read(path)
    }

    /// The account whose secret `path` holds as 64 hex characters.
    fn read(path: &Path) -> Result<Account> {
        let text = std::fs::read_to_string(path)?;
        let text = text.trim();
        let bytes = (0..32)
            .map(|i| {
                text.get(i * 2..i * 2 + 2)
                    .and_then(|b| u8::from_str_radix(b, 16).ok())
            })
            .collect::<Option<Vec<u8>>>()
            .filter(|_| text.len() == 64)
            .ok_or_else(|| anyhow!("{} is not 64 hex characters", path.display()))?;
        Account::new(&bytes.try_into().expect("32 bytes"))
    }
}

static OPENS: AtomicU64 = AtomicU64::new(0);
