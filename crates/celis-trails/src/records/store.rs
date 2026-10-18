//! The records of every project on this machine.

use crate::records::verified::Verified;
use crate::{Link, Phrase, Refusal};
use anyhow::Result;
use celis_nostr::{Account, Note};
use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
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
    verified: Verified,
}

impl Store {
    /// Whether `home` holds a store.
    pub fn exists(home: &Path) -> bool {
        Store::root(home).is_dir()
    }

    /// The store in `home`, created when it is missing.
    pub fn open(home: &Path) -> Result<Store, Refusal> {
        let root = Store::root(home);
        std::fs::create_dir_all(root.join("shared")).map_err(|e| unwritten(&root, e))?;
        Ok(Store {
            root,
            phrase: Phrase::at(home),
            links: vec![],
            account: OnceCell::new(),
            read: RefCell::new(None),
            seen: RefCell::default(),
            verified: Verified::open(home.join("cache/verified")),
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
        let mut kept = read(&self.root.join("shared"), |_| true, &self.verified);
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

    /// The records of the files `paths` names that it lacks, in `shared/` or a linked folder, that parse and
    /// verify; each is copied into `shared/` when it is not there, and they join the records. None before
    /// the first read.
    pub fn take(&self, paths: &[PathBuf]) -> Vec<Note> {
        let mut read = self.read.borrow_mut();
        let Some(read) = read.as_mut() else {
            return Vec::new();
        };
        let mut seen = self.seen.borrow_mut();
        let mut taken = Vec::new();
        for path in paths {
            let file = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if seen.contains(file) {
                continue;
            }
            let Some(record) = note(path, &self.verified) else {
                continue;
            };
            if !seen.insert(name(&record)) {
                continue;
            }
            let _ = put(&self.root.join("shared"), &record);
            taken.push(record);
        }
        read.extend(taken.iter().cloned());
        self.verified.save();
        taken
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
        let fresh = read(
            &self.root.join("shared"),
            |file| !seen.contains(file),
            &self.verified,
        );
        seen.extend(fresh.iter().map(name));
        fresh
    }

    /// Copies each record of the linked folders that `shared/` lacks into it, and adds it to `all`; the
    /// count.
    fn take_in(&self, all: &mut Vec<Note>) -> Result<usize> {
        let mut came = 0;
        let linked = self.links.iter();
        for record in linked.flat_map(|l| read(&l.folder, |_| true, &self.verified)) {
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
    let written = std::fs::write(&temp, serde_json::to_string(record)?);
    written
        .and_then(|()| std::fs::rename(&temp, &path))
        .map_err(|e| unwritten(folder, e))?;
    Ok((path, true))
}

/// A failed write in `folder`: it names the folder, and when the system refused the write, how to let an
/// agent's sandbox write it.
fn unwritten(folder: &Path, error: std::io::Error) -> anyhow::Error {
    let hint = match error.kind() {
        ErrorKind::ReadOnlyFilesystem | ErrorKind::PermissionDenied => {
            " (in a sandbox, let the agent write it; Codex: add it to \
             sandbox_workspace_write.writable_roots)"
        }
        _ => "",
    };
    anyhow::Error::new(error).context(format!("cannot write {}{hint}", folder.display()))
}

/// The name of the file of `record`: `<created_at>-<id>.json`.
fn name(record: &Note) -> String {
    format!("{}-{}.json", record.created_at, record.id)
}

/// Every record in `folder` of a file `wanted` names, that parses and verifies, read on as many threads
/// as the machine runs; a file `verified` holds is not checked again.
fn read(folder: &Path, wanted: impl Fn(&str) -> bool, verified: &Verified) -> Vec<Note> {
    let paths: Vec<PathBuf> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|f| f.to_str())
                .is_some_and(&wanted)
        })
        .collect();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let size = paths.len().div_ceil(threads).max(1);
    let notes = std::thread::scope(|scope| {
        let parts: Vec<_> = paths
            .chunks(size)
            .map(|part| {
                scope.spawn(move || part.iter().filter_map(|p| note(p, verified)).collect())
            })
            .collect();
        let parts = parts
            .into_iter()
            .map(|part| part.join().unwrap_or_default());
        parts.flat_map(|part: Vec<Note>| part).collect()
    });
    verified.save();
    notes
}

/// The record of the `.json` file at `path`, when it parses and verifies or `verified` holds it.
fn note(path: &Path, verified: &Verified) -> Option<Note> {
    if path.extension().is_none_or(|x| x != "json") {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    let note = serde_json::from_str::<Note>(&text).ok()?;
    verified.holds(path, || note.verifies()).then_some(note)
}

#[cfg(test)]
mod tests;
