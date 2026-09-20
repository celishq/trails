//! **Where a socket leaves what landed, under its stream, for the turn that lands it.**

use crate::Note;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// What arrives on the held subscriptions, held per stream until a landing takes it. A socket hands each
/// note over unopened; a connect rings with none, which says the walk is owed (`spec/runtime.md`, "What
/// arrives is data").
///
/// **A note is held before whoever is listening is told**, so a host owes a ring nothing but a turn, and
/// losing what is held costs the walk on the next connect rather than the record. One per node, never a
/// process-wide sink: a test binary opens many.
#[derive(Default)]
pub struct Mailbox {
    listening: Mutex<Option<Ring>>,
    held: Mutex<BTreeMap<String, Held>>,
}

/// What one stream has said since a landing last took it: the notes its socket handed over, and whether
/// the walk from the cursor is owed — a connect says so and carries no note.
#[derive(Debug, Default)]
pub struct Held {
    pub notes: Vec<Note>,
    pub walk: bool,
}

impl Held {
    /// A stream owed its walk, with nothing heard.
    pub fn walking() -> Self {
        Self {
            notes: Vec::new(),
            walk: true,
        }
    }
}

/// Whoever hears the mailbox, told the stream a relay handed something over on.
type Ring = Arc<dyn Fn(&str) + Send + Sync>;

impl Mailbox {
    /// Let `ring` hear this mailbox from now on, in place of whoever heard it before.
    pub fn heard_by(&self, ring: impl Fn(&str) + Send + Sync + 'static) {
        if let Ok(mut held) = self.listening.lock() {
            *held = Some(Arc::new(ring));
        }
    }

    /// Hold `note`, which landed on `stream`, for the turn that lands it.
    pub fn heard(&self, stream: &str, note: Note) {
        if let Ok(mut held) = self.held.lock() {
            held.entry(stream.to_string()).or_default().notes.push(note);
        }
        self.tell(stream);
    }

    /// Say a relay may hold something on `stream` that no socket heard — what a connect says.
    pub fn ring(&self, stream: &str) {
        if let Ok(mut held) = self.held.lock() {
            held.entry(stream.to_string()).or_default().walk = true;
        }
        self.tell(stream);
    }

    /// Whoever is listening is told outside the lock: a host that takes its time, or rings back in from
    /// inside the call, must not hold every relay's thread with it.
    fn tell(&self, stream: &str) {
        let ring = self.listening.lock().ok().and_then(|held| held.clone());
        if let Some(ring) = ring {
            ring(stream);
        }
    }

    /// Every stream heard from and not taken since.
    pub fn all(&self) -> BTreeSet<String> {
        self.held
            .lock()
            .map(|held| held.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// [`all`](Self::all), narrowed to `named` — and the rest forgotten. A relay taken out stops being a
    /// stream this vault reads, and what was heard about one is answered by nothing.
    pub fn only(&self, named: &[String]) -> BTreeSet<String> {
        let Ok(mut held) = self.held.lock() else {
            return BTreeSet::new();
        };
        held.retain(|it, _| named.contains(it));
        held.keys().cloned().collect()
    }

    /// Take what `streams` hold — the landing's now, and heard no more. Taken before the walk, so a note
    /// landing while it runs is heard again rather than lost.
    pub fn take(&self, streams: &[String]) -> BTreeMap<String, Held> {
        let Ok(mut held) = self.held.lock() else {
            return BTreeMap::new();
        };
        streams
            .iter()
            .filter_map(|it| held.remove_entry(it))
            .collect()
    }
}

#[cfg(test)]
mod tests;
