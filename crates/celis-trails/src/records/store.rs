//! The records of every project on this machine.

use crate::records::verified::Verified;
use crate::{Link, Phrase, ProofFile, Refusal};
use anyhow::Result;
use celis_nostr::{Account, Note};
use sha2::{Digest, Sha256};
use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The records in `$CELIS_HOME/trails/`, and the account of `$CELIS_HOME/phrase` that signs new ones. It
/// reads the records from disk once, keeps those it writes after, and the names of the files of
/// `shared/` it holds, and the blobs of `blobs/`. It reads and writes its linked folders too.
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

    /// Keeps `bytes` as a blob in `blobs/` and in each linked folder, unless it is there; its sha256.
    pub fn keep(&self, bytes: &[u8]) -> Result<String> {
        let sha = sha256(bytes);
        put_blob(&self.root.join("blobs"), &sha, bytes)?;
        for link in &self.links {
            put_blob(&link.folder.join("blobs"), &sha, bytes)?;
        }
        Ok(sha)
    }

    /// The blob `sha256` of `blobs/`, when its bytes hash to its name.
    pub fn blob(&self, sha256: &str) -> Option<PathBuf> {
        let path = self.root.join("blobs").join(sha256);
        let bytes = std::fs::read(&path).ok()?;
        (self::sha256(&bytes) == sha256).then_some(path)
    }

    /// Copies each record a linked folder holds and lacks into it, with the blobs its files name, and
    /// each record of a linked folder the store lacks into `shared/`. Returns the counts of records
    /// copied out and in.
    pub fn sync(&self) -> Result<(usize, usize)> {
        let mut kept = read(&self.root.join("shared"), |_| true, &self.verified);
        let came = self.take_in(&mut kept)?;
        let all: HashMap<&str, &Note> = kept.iter().map(|r| (r.id.as_str(), r)).collect();
        let mut went = 0;
        for link in &self.links {
            for record in kept.iter().filter(|r| link.holds(r, &all)) {
                went += usize::from(put(&link.folder, record)?.1);
                for file in record.tags.iter().filter_map(|t| ProofFile::read(t)) {
                    if let Some(path) = self.blob(&file.sha256) {
                        put_blob(
                            &link.folder.join("blobs"),
                            &file.sha256,
                            &std::fs::read(path)?,
                        )?;
                    }
                }
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

    /// Copies each record of the linked folders that `shared/` lacks into it, and adds it to `all`, and
    /// each blob of them that `blobs/` lacks; the count of records.
    fn take_in(&self, all: &mut Vec<Note>) -> Result<usize> {
        for link in &self.links {
            self.take_blobs(&link.folder.join("blobs"))?;
        }
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

    /// Copies each blob of `folder` that `blobs/` lacks, and whose bytes hash to its name, into it.
    fn take_blobs(&self, folder: &Path) -> Result<()> {
        for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || self.root.join("blobs").join(&name).exists() {
                continue;
            }
            let bytes = std::fs::read(entry.path())?;
            if sha256(&bytes) == name {
                put_blob(&self.root.join("blobs"), &name, &bytes)?;
            }
        }
        Ok(())
    }
}

/// Writes `bytes` to `folder` as the file `name` at once, unless it is there; its path, and whether it
/// wrote it.
fn put_bytes(folder: &Path, name: &str, bytes: &[u8]) -> Result<(PathBuf, bool)> {
    let path = folder.join(name);
    if path.exists() {
        return Ok((path, false));
    }
    let temp = folder.join(format!(".{name}.{}", std::process::id()));
    std::fs::write(&temp, bytes)
        .and_then(|()| std::fs::rename(&temp, &path))
        .map_err(|e| unwritten(folder, e))?;
    Ok((path, true))
}

/// Writes `bytes` to `folder` as the blob `sha256`, unless it is there.
fn put_blob(folder: &Path, sha256: &str, bytes: &[u8]) -> Result<()> {
    std::fs::create_dir_all(folder).map_err(|e| unwritten(folder, e))?;
    put_bytes(folder, sha256, bytes).map(|_| ())
}

/// Writes `record` to `folder` as `<created_at>-<id>.json`, unless it is there; the path, and whether it
/// wrote it.
fn put(folder: &Path, record: &Note) -> Result<(PathBuf, bool)> {
    put_bytes(
        folder,
        &name(record),
        serde_json::to_string(record)?.as_bytes(),
    )
}

/// The sha256 of `bytes`, in lower-case hex.
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
