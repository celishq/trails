//! The record files a reader verified, so it reads them again with no check.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

/// The record files whose records verified, by name with the size and time each had then, kept in a
/// file of lines `<name> <size> <nanoseconds>`. A file that changed since is not in it.
pub struct Verified {
    file: PathBuf,
    known: HashMap<String, Stamp>,
    added: Mutex<Vec<(String, Stamp)>>,
}

/// The size of a file and the time it last changed, in nanoseconds since the epoch.
type Stamp = (u64, u128);

impl Verified {
    /// The files `file` lists; none when it is missing. A list with more lines than files is written
    /// again with one line a file.
    pub fn open(file: PathBuf) -> Verified {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        let known: HashMap<String, Stamp> = text
            .lines()
            .filter_map(|line| {
                let mut parts = line.split(' ');
                let name = parts.next()?.to_string();
                Some((
                    name,
                    (parts.next()?.parse().ok()?, parts.next()?.parse().ok()?),
                ))
            })
            .collect();
        if text.lines().count() > known.len() {
            let lines: String = known.iter().map(|(n, s)| line(n, s)).collect();
            let _ = std::fs::write(&file, lines);
        }
        Verified {
            file,
            known,
            added: Mutex::default(),
        }
    }

    /// Whether the record of the file at `path` verified before it last changed; else `check` says
    /// whether it verifies, and a file that does is added.
    pub fn holds(&self, path: &Path, check: impl FnOnce() -> bool) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(stamp) = stamp(path) else {
            return check();
        };
        if self.known.get(name) == Some(&stamp) {
            return true;
        }
        let verifies = check();
        if verifies {
            if let Ok(mut added) = self.added.lock() {
                added.push((name.to_string(), stamp));
            }
        }
        verifies
    }

    /// Adds the files verified since the last save to its file, when it can write it.
    pub fn save(&self) {
        let Ok(mut added) = self.added.lock() else {
            return;
        };
        if added.is_empty() {
            return;
        }
        let lines: String = added
            .drain(..)
            .map(|(name, stamp)| line(&name, &stamp))
            .collect();
        if let Some(folder) = self.file.parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        let file = std::fs::File::options()
            .create(true)
            .append(true)
            .open(&self.file);
        let _ = file.and_then(|mut f| f.write_all(lines.as_bytes()));
    }
}

/// The line of the file `name` with its `stamp`.
fn line(name: &str, (size, time): &Stamp) -> String {
    format!("{name} {size} {time}\n")
}

/// The stamp of the file at `path`.
fn stamp(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    let time = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    Some((meta.len(), time.as_nanos()))
}

#[cfg(test)]
mod tests;
