//! The patterns `search` gave on this machine.

use crate::text::clock::date;
use celis_nostr::Note;
use celis_store::records::id::short;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The patterns `search` gave on this machine: in `$CELIS_HOME/recalled/`, one file a pattern by its id,
/// one byte longer each time, last changed on the day it last came.
#[derive(Default)]
pub(crate) struct Recalls {
    by: HashMap<String, (String, u64)>,
}

impl Recalls {
    /// The recalls kept under `home`.
    pub fn read(home: &Path) -> Recalls {
        let entries = std::fs::read_dir(folder(home)).into_iter().flatten();
        let by = entries.flatten().filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            let at = meta.modified().ok()?;
            let at = at.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
            let id = entry.file_name().to_string_lossy().to_string();
            Some((short(&id), (date(at), meta.len())))
        });
        Recalls { by: by.collect() }
    }

    /// Keeps under `home` that `search` gave each of `notes` now.
    pub fn mark<'a>(home: &Path, notes: impl Iterator<Item = &'a Note>) {
        let folder = folder(home);
        let _ = std::fs::create_dir_all(&folder);
        for note in notes {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(folder.join(&note.id));
            let _ = file.and_then(|mut f| f.write_all(b"."));
        }
    }

    /// The day `search` last gave the pattern `id`.
    pub fn day(&self, id: &str) -> Option<&str> {
        self.by.get(&short(id)).map(|(day, _)| day.as_str())
    }

    /// How many times `search` gave the pattern `id`.
    pub fn count(&self, id: &str) -> u64 {
        self.by.get(&short(id)).map_or(0, |(_, n)| *n)
    }
}

fn folder(home: &Path) -> PathBuf {
    home.join("recalled")
}
