//! The records of a home at one moment, and the signed append of new ones.

use crate::{Anchor, Home, ProofFile, Refusal, Secret, Source};
use celis_nostr::{Account, Note, Pubkey};
use std::path::PathBuf;

/// The records of a home at the time `now`, and the account's writes on them, each signed now. A
/// step's check runs for `limit` seconds at most. Readers take the time from its steps.
pub struct Store {
    now: u64,
    home: Home,
    records: Vec<Note>,
    limit: u64,
}

impl Store {
    /// The records of `home` at `now`.
    pub fn open(home: Home, now: u64, limit: u64) -> Store {
        Store {
            records: home.records(),
            home,
            now,
            limit,
        }
    }

    /// Moves it to `now`, with the records of its home as it last read them and as it wrote them since.
    pub fn at(&mut self, now: u64) {
        self.now = now;
        self.records = self.home.records();
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    /// The seconds a step's check runs at most.
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Adds the records of the files `paths` names that it lacks, as `Home::take` reads them; those
    /// records.
    pub fn take(&mut self, paths: &[PathBuf]) -> Vec<Note> {
        let taken = self.home.take(paths);
        self.records.extend(taken.iter().cloned());
        self.records
            .sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        taken
    }

    /// The account's public key; none when no phrase is kept.
    pub fn author(&self) -> Result<Option<Pubkey>, Refusal> {
        Ok(self.home.author()?)
    }

    /// Whether the home holds no record.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn records(&self) -> &[Note] {
        &self.records
    }

    /// The anchor of the newest record of the project `project`.
    pub fn anchor(&self, project: &str) -> Option<Anchor> {
        let mut anchors = self.records.iter().rev().filter_map(Anchor::read);
        anchors.find(|a| a.project == project)
    }

    /// Keeps the bytes of `source` as a blob, as a file of a proof at `place`; refuses one that holds a
    /// secret.
    pub fn keep(&self, place: String, source: &Source) -> Result<ProofFile, Refusal> {
        let (bytes, media) = source.read()?;
        if let Some(secret) = Secret::find(&String::from_utf8_lossy(&bytes)) {
            return Err(Refusal::Secret(format!(
                "the file {place} holds {}",
                secret.kind
            )));
        }
        let file = ProofFile::new(place, String::new(), bytes.len() as u64).typed(media, &bytes);
        let sha256 = self.home.keep(&bytes, &file.ext())?;
        Ok(ProofFile { sha256, ..file })
    }

    /// Keeps `bytes` as the blob of `file` when they hash to its sha256; whether they do.
    pub fn hold(&self, file: &ProofFile, bytes: &[u8]) -> Result<bool, Refusal> {
        Ok(self.home.hold(&file.sha256, bytes, &file.ext())?)
    }

    /// Removes the files of `records`, and the blobs `blobs` names.
    pub fn remove(&mut self, records: &[Note], blobs: &[String]) -> Result<(), Refusal> {
        self.home.remove(records, blobs)?;
        self.records = self.home.records();
        Ok(())
    }

    /// Writes `record`, signed by any account, as it is.
    pub fn put(&self, record: &Note) -> Result<(), Refusal> {
        self.home.write(record)?;
        Ok(())
    }

    /// The blob `sha256` of the home, when its bytes hash to its name.
    pub fn blob(&self, sha256: &str) -> Option<PathBuf> {
        self.home.blob(sha256)
    }

    /// Writes the record `record` gives for the account and now, and returns it; refuses one that holds
    /// a secret.
    pub fn sign(
        &self,
        record: impl FnOnce(&Account, u64) -> anyhow::Result<Note>,
    ) -> Result<Note, Refusal> {
        let note = self.seal(record)?;
        self.home.write(&note)?;
        Ok(note)
    }

    /// The record `record` gives for the account and now, not written; refuses one that holds a secret.
    pub fn seal(
        &self,
        record: impl FnOnce(&Account, u64) -> anyhow::Result<Note>,
    ) -> Result<Note, Refusal> {
        let note = record(self.home.account()?, self.now)?;
        let values = note.tags.iter().flat_map(|tag| tag.iter().skip(1));
        if let Some(secret) = std::iter::once(&note.content)
            .chain(values)
            .find_map(|t| Secret::find(t))
        {
            return Err(Refusal::Secret(format!("the record holds {}", secret.kind)));
        }
        Ok(note)
    }
}
