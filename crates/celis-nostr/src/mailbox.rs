//! **Where a socket leaves what landed, under its stream, for the turn that lands it.**

use crate::Note;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

mod held;
pub use held::Held;

/// What arrives on the held subscriptions, held per stream until a landing takes it: a socket hands each
/// note over unopened, and a connect rings with none, meaning the walk is owed.
///
/// A note is held before whoever listens is told, so losing it costs the walk again, never the record.
#[derive(Default)]
pub struct Mailbox {
    listening: Mutex<Option<Ring>>,
    held: Mutex<BTreeMap<String, Held>>,
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
