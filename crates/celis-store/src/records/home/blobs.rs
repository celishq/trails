//! A folder of blobs.

use super::{put_bytes, unwritten};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// A folder of blobs: files named by the sha256 of their bytes and an extension, in lower-case hex
/// (`spec/08-files.md`, "Blobs").
pub(super) struct Blobs {
    folder: PathBuf,
}

impl Blobs {
    pub fn at(folder: PathBuf) -> Blobs {
        Blobs { folder }
    }

    /// Keeps `bytes` as a blob named with `ext`, unless the folder holds it; their sha256.
    pub fn keep(&self, bytes: &[u8], ext: &str) -> Result<String> {
        let sha = sha256(bytes);
        self.put(&sha, ext, bytes)?;
        Ok(sha)
    }

    /// Keeps `bytes` as the blob `sha256` named with `ext` when they hash to it; whether they do.
    pub fn hold(&self, sha256: &str, bytes: &[u8], ext: &str) -> Result<bool> {
        if self::sha256(bytes) != sha256 {
            return Ok(false);
        }
        self.put(sha256, ext, bytes)?;
        Ok(true)
    }

    /// The path of the blob `sha256`, when its bytes hash to its name.
    pub fn get(&self, sha256: &str) -> Option<PathBuf> {
        self.named(sha256)
            .find(|path| std::fs::read(path).is_ok_and(|bytes| self::sha256(&bytes) == sha256))
    }

    /// Removes each file of the blob `sha256`.
    pub fn remove(&self, sha256: &str) {
        for path in self.named(sha256) {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Copies the blob `sha256` of `from` into it with the extension it has there, when `from` holds it.
    pub fn copy(&self, from: &Blobs, sha256: &str) -> Result<()> {
        let Some(path) = from.get(sha256) else {
            return Ok(());
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let ext = name.strip_prefix(sha256).unwrap_or_default();
        self.put(sha256, ext.trim_start_matches('.'), &std::fs::read(&path)?)
    }

    /// Copies each blob of `from` that it lacks into it.
    pub fn take(&self, from: &Blobs) -> Result<()> {
        for entry in std::fs::read_dir(&from.folder)
            .into_iter()
            .flatten()
            .flatten()
        {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with('.') {
                self.copy(from, name.split('.').next().unwrap_or_default())?;
            }
        }
        Ok(())
    }

    /// Writes `bytes` as the blob `sha256` named with `ext`, unless the folder holds the blob.
    fn put(&self, sha256: &str, ext: &str, bytes: &[u8]) -> Result<()> {
        if self.get(sha256).is_some() {
            return Ok(());
        }
        std::fs::create_dir_all(&self.folder).map_err(|e| unwritten(&self.folder, e))?;
        put_bytes(&self.folder, &name(sha256, ext), bytes)?;
        Ok(())
    }

    /// The files of the folder that are the blob `sha256`, named with or without an extension.
    fn named<'a>(&'a self, sha256: &'a str) -> impl Iterator<Item = PathBuf> + 'a {
        let entries = std::fs::read_dir(&self.folder)
            .into_iter()
            .flatten()
            .flatten();
        entries
            .map(|entry| entry.path())
            .filter(move |path| is_blob_of(path, sha256))
    }
}

/// The sha256 of `bytes`, in lower-case hex.
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The file name of the blob `sha256` with `ext`: `<sha256>.<ext>`, or `<sha256>` when `ext` is empty.
fn name(sha256: &str, ext: &str) -> String {
    match ext {
        "" => sha256.to_string(),
        ext => format!("{sha256}.{ext}"),
    }
}

/// Whether `path` is named `<sha256>`, or `<sha256>.<ext>`.
fn is_blob_of(path: &Path, sha256: &str) -> bool {
    let name = path.file_name().and_then(|n| n.to_str());
    name.is_some_and(|n| {
        n == sha256
            || n.strip_prefix(sha256)
                .is_some_and(|rest| rest.starts_with('.'))
    })
}
