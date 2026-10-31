//! The records a reader verified, kept in one file, so it reads them again with no file of each.

use celis_nostr::Note;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// The records whose files verified, by the name of their file, kept as one JSON record a line.
pub struct Pack {
    file: PathBuf,
    known: HashMap<String, Note>,
    added: Mutex<Vec<Note>>,
}

impl Pack {
    /// The records `file` holds; none when it is missing. A line that does not parse is skipped.
    pub fn open(file: PathBuf) -> Pack {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        let known = text
            .lines()
            .filter_map(|line| serde_json::from_str::<Note>(line).ok())
            .map(|note| (name(&note), note))
            .collect();
        Pack {
            file,
            known,
            added: Mutex::default(),
        }
    }

    /// The record of the file `name`, when it verified before.
    pub fn get(&self, name: &str) -> Option<&Note> {
        self.known.get(name)
    }

    /// Adds `note`, which verified, at the next save.
    pub fn add(&self, note: &Note) {
        if let Ok(mut added) = self.added.lock() {
            added.push(note.clone());
        }
    }

    /// Adds the records verified since the last save to its file, when it can write it.
    pub fn save(&self) {
        let Ok(mut added) = self.added.lock() else {
            return;
        };
        if added.is_empty() {
            return;
        }
        let lines: String = added
            .drain(..)
            .filter_map(|note| serde_json::to_string(&note).ok())
            .map(|line| line + "\n")
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

/// The name of the file of `record`: `<created_at>-<id>.json`.
pub fn name(record: &Note) -> String {
    format!("{}-{}.json", record.created_at, record.id)
}

#[cfg(test)]
mod tests;
