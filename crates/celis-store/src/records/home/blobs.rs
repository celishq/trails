//! A folder of blobs.

use super::{put_bytes, unwritten};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// A folder of blobs: files named by the sha256 of their bytes, in lower-case hex (`spec/08-files.md`,
/// "Blobs").
pub(super) struct Blobs {
    folder: PathBuf,
}

impl Blobs {
    pub fn at(folder: PathBuf) -> Blobs {
        Blobs { folder }
    }

    /// Keeps `bytes` as a blob, unless it holds them; their sha256.
    pub fn keep(&self, bytes: &[u8]) -> Result<String> {
        let sha = sha256(bytes);
        std::fs::create_dir_all(&self.folder).map_err(|e| unwritten(&self.folder, e))?;
        put_bytes(&self.folder, &sha, bytes)?;
        Ok(sha)
    }

    /// The path of the blob `sha256`, when its bytes hash to its name.
    pub fn get(&self, sha256: &str) -> Option<PathBuf> {
        let path = self.folder.join(sha256);
        let bytes = std::fs::read(&path).ok()?;
        (self::sha256(&bytes) == sha256).then_some(path)
    }

    /// Copies the blob `sha256` of `from` into it, when `from` holds it.
    pub fn copy(&self, from: &Blobs, sha256: &str) -> Result<()> {
        match from.get(sha256) {
            Some(path) => self.keep(&std::fs::read(path)?).map(|_| ()),
            None => Ok(()),
        }
    }

    /// Copies each blob of `from` that it lacks into it.
    pub fn take(&self, from: &Blobs) -> Result<()> {
        for entry in std::fs::read_dir(&from.folder)
            .into_iter()
            .flatten()
            .flatten()
        {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with('.') && !self.folder.join(&name).exists() {
                self.copy(from, &name)?;
            }
        }
        Ok(())
    }
}

/// The sha256 of `bytes`, in lower-case hex.
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
