//! The account's recovery phrase, kept in one file.

use anyhow::{Context, Result};
use celis_nostr::Identity;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// The file `$CELIS_HOME/phrase`, readable by its owner alone, that keeps the account's 12 words.
pub struct Phrase {
    path: PathBuf,
}

impl Phrase {
    /// The phrase file of `home`.
    pub fn at(home: &Path) -> Phrase {
        Phrase {
            path: home.join("phrase"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The identity the kept phrase gives, minted on first use. Racing mints end with one phrase.
    pub fn identity(&self) -> Result<Identity> {
        if !self.path.exists() {
            let temp = self.temp(&Identity::generate()?)?;
            let linked = std::fs::hard_link(&temp, &self.path);
            std::fs::remove_file(&temp)?;
            match linked {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        self.kept()?.context("the phrase is gone")
    }

    /// The identity the kept phrase gives; none when no phrase is kept.
    pub fn kept(&self) -> Result<Option<Identity>> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => Identity::from_phrase(text.trim())
                .with_context(|| format!("{} holds no phrase", self.path.display()))
                .map(Some),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Keeps the phrase of `identity`, in place of any kept one.
    pub fn keep(&self, identity: &Identity) -> Result<()> {
        let temp = self.temp(identity)?;
        std::fs::rename(&temp, &self.path)?;
        Ok(())
    }

    /// A new file beside the phrase that holds the phrase of `identity`, readable by its owner alone.
    fn temp(&self, identity: &Identity) -> Result<PathBuf> {
        if let Some(home) = self.path.parent() {
            std::fs::create_dir_all(home)?;
        }
        let temp = self
            .path
            .with_extension(format!("{}.tmp", std::process::id()));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?
            .write_all(format!("{}\n", identity.phrase()).as_bytes())?;
        Ok(temp)
    }
}
