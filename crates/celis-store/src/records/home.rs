//! The records of every project on this machine, as a celis home keeps them.

mod blobs;

use crate::records::pack::{name, Pack};
use crate::{Link, Phrase, ProofFile, Refusal};
use anyhow::Result;
use blobs::Blobs;
use celis_nostr::{Account, Note, Pubkey};
use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The records in `$CELIS_HOME/trails/`, and the account of `$CELIS_HOME/phrase` that signs new ones. It
/// reads the records from disk once, keeps those it writes after, and the names of the files of
/// `shared/` it holds, and the blobs of `blobs/`. It reads and writes its linked folders too.
pub struct Home {
    root: PathBuf,
    phrase: Phrase,
    links: Vec<Link>,
    account: OnceCell<Account>,
    read: RefCell<Option<Vec<Note>>>,
    seen: RefCell<HashSet<String>>,
    pack: Pack,
    blobs: Blobs,
}

impl Home {
    /// Whether `home` holds a store.
    pub fn exists(home: &Path) -> bool {
        Home::root(home).is_dir()
    }

    /// The store in `home`, created when it is missing.
    pub fn open(home: &Path) -> Result<Home, Refusal> {
        let root = Home::root(home);
        std::fs::create_dir_all(root.join("shared")).map_err(|e| unwritten(&root, e))?;
        Ok(Home {
            phrase: Phrase::at(home),
            links: vec![],
            account: OnceCell::new(),
            read: RefCell::new(None),
            seen: RefCell::default(),
            pack: Pack::open(home.join("cache/pack")),
            blobs: Blobs::at(root.join("blobs")),
            root,
        })
    }

    /// The store that also reads and writes the folder of `link`.
    pub fn with(mut self, link: Link) -> Home {
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

    /// The public key of the kept account; none when no phrase is kept.
    pub fn author(&self) -> Result<Option<Pubkey>> {
        let kept = self.phrase.kept()?;
        Ok(kept.map(|identity| identity.account().pubkey()))
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

    /// Keeps `bytes` as a blob named with `ext` in `blobs/` and in each linked folder, unless it is there;
    /// its sha256.
    pub fn keep(&self, bytes: &[u8], ext: &str) -> Result<String> {
        let sha = self.blobs.keep(bytes, ext)?;
        for link in &self.links {
            link_blobs(link).keep(bytes, ext)?;
        }
        Ok(sha)
    }

    /// Removes the files of `records` from `shared/`, and the blobs `blobs` names from `blobs/`.
    pub fn remove(&self, records: &[Note], blobs: &[String]) -> Result<()> {
        for record in records {
            let path = self.root.join("shared").join(name(record));
            match std::fs::remove_file(&path) {
                Err(e) if e.kind() != ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            }
        }
        for sha in blobs {
            self.blobs.remove(sha);
        }
        *self.read.borrow_mut() = None;
        self.seen.borrow_mut().clear();
        Ok(())
    }

    /// Keeps `bytes` as the blob `sha256` named with `ext` when they hash to it, as `keep` does; whether
    /// they do.
    pub fn hold(&self, sha256: &str, bytes: &[u8], ext: &str) -> Result<bool> {
        if !self.blobs.hold(sha256, bytes, ext)? {
            return Ok(false);
        }
        self.keep(bytes, ext)?;
        Ok(true)
    }

    /// The blob `sha256` of `blobs/`, when its bytes hash to its name.
    pub fn blob(&self, sha256: &str) -> Option<PathBuf> {
        self.blobs.get(sha256)
    }

    /// Copies each record a linked folder holds and lacks into it, with the blobs its tags name, and each
    /// record of a linked folder the store lacks into `shared/`. Returns the counts of records copied out
    /// and in.
    pub fn sync(&self) -> Result<(usize, usize)> {
        let mut kept = read(&self.root.join("shared"), |_| true, &self.pack);
        let came = self.take_in(&mut kept)?;
        let all: HashMap<&str, &Note> = kept.iter().map(|r| (r.id.as_str(), r)).collect();
        let mut went = 0;
        for link in &self.links {
            for record in kept.iter().filter(|r| link.holds(r, &all)) {
                went += usize::from(put(&link.folder, record)?.1);
                for file in ProofFile::all(record) {
                    link_blobs(link).copy(&self.blobs, &file.sha256)?;
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
            let Some(record) = note(path, &self.pack) else {
                continue;
            };
            if !seen.insert(name(&record)) {
                continue;
            }
            let _ = put(&self.root.join("shared"), &record);
            taken.push(record);
        }
        read.extend(taken.iter().cloned());
        self.pack.save();
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
            &self.pack,
        );
        seen.extend(fresh.iter().map(name));
        fresh
    }

    /// Copies each record of the linked folders that `shared/` lacks into it, and adds it to `all`, and
    /// each blob of them that `blobs/` lacks; the count of records.
    fn take_in(&self, all: &mut Vec<Note>) -> Result<usize> {
        for link in &self.links {
            self.blobs.take(&link_blobs(link))?;
        }
        let mut came = 0;
        let linked = self.links.iter();
        for record in linked.flat_map(|l| read(&l.folder, |_| true, &self.pack)) {
            if put(&self.root.join("shared"), &record)?.1 {
                self.seen.borrow_mut().insert(name(&record));
                all.push(record);
                came += 1;
            }
        }
        Ok(came)
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

/// The blobs of the folder of `link`.
fn link_blobs(link: &Link) -> Blobs {
    Blobs::at(link.folder.join("blobs"))
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

/// Every record in `folder` of a file `wanted` names, that parses and verifies, read on as many threads
/// as the machine runs; a record `pack` holds is taken from it.
fn read(folder: &Path, wanted: impl Fn(&str) -> bool, pack: &Pack) -> Vec<Note> {
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
            .map(|part| scope.spawn(move || part.iter().filter_map(|p| note(p, pack)).collect()))
            .collect();
        let parts = parts
            .into_iter()
            .map(|part| part.join().unwrap_or_default());
        parts.flat_map(|part: Vec<Note>| part).collect()
    });
    pack.save();
    notes
}

/// The record of the `.json` file at `path`: the one `pack` holds, else the file's when it parses and
/// verifies, which joins the pack.
fn note(path: &Path, pack: &Pack) -> Option<Note> {
    if path.extension().is_none_or(|x| x != "json") {
        return None;
    }
    let file = path.file_name()?.to_str()?;
    if let Some(note) = pack.get(file) {
        return Some(note.clone());
    }
    let text = std::fs::read_to_string(path).ok()?;
    let note = serde_json::from_str::<Note>(&text).ok()?;
    if !note.verifies() {
        return None;
    }
    pack.add(&note);
    Some(note)
}

#[cfg(test)]
mod tests;
