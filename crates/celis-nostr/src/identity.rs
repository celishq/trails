//! The **identity** — the account key derived from one BIP39 recovery phrase: the secp256k1 key every note
//! is signed and encrypted with, and the whole of who this account is on the network. The phrase is the single
//! backup that restores both.

use crate::account::Account;
use anyhow::{anyhow, Result};
use bip39::Mnemonic;
use hkdf::Hkdf;
use sha2::Sha256;

/// The HKDF salt binding every derivation to this application, so the same phrase used elsewhere never
/// yields these keys.
const SALT: &[u8] = b"celis-identity-v1";
/// The HKDF info label for the nostr account key — the secp256k1 secret every note is signed and encrypted
/// with. Its own label, kept as it is: changing it would derive a different account from the same phrase.
const ACCOUNT_INFO: &[u8] = b"account-secp256k1";

/// An account derived from a recovery phrase: the account key, and the recovery `mnemonic` to show the user
/// for backup. Deterministic — the same phrase always rebuilds the same identity.
pub struct Identity {
    mnemonic: Mnemonic,
    account: Account,
}

impl Identity {
    /// Create a brand-new account: a fresh 12-word recovery phrase and the keys derived from it.
    pub fn generate() -> Result<Self> {
        let mnemonic = Mnemonic::generate(12).map_err(|e| anyhow!("mnemonic: {e}"))?;
        Self::from_mnemonic(mnemonic)
    }

    /// Rebuild an identity from an existing recovery phrase; errors on an invalid phrase.
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse(phrase).map_err(|e| anyhow!("bad recovery phrase: {e}"))?;
        Self::from_mnemonic(mnemonic)
    }

    /// The recovery phrase — the one backup the user must record. It restores the account key, so a new
    /// device reads the whole history the moment it holds the phrase.
    pub fn phrase(&self) -> String {
        self.mnemonic.to_string()
    }

    /// This account on nostr — the secp256k1 key every note is signed and encrypted with.
    pub fn account(&self) -> &Account {
        &self.account
    }

    /// Derive the account key from a mnemonic's seed via HKDF-SHA256.
    fn from_mnemonic(mnemonic: Mnemonic) -> Result<Self> {
        let seed = mnemonic.to_seed("");
        let hk = Hkdf::<Sha256>::new(Some(SALT), &seed);

        let mut account = [0u8; 32];
        hk.expand(ACCOUNT_INFO, &mut account)
            .map_err(|e| anyhow!("account derive: {e}"))?;
        let account = Account::new(&account)?;

        Ok(Identity { mnemonic, account })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phrase_deterministically_rebuilds_the_same_nostr_account() {
        let original = Identity::generate().unwrap();
        let restored = Identity::from_phrase(&original.phrase()).unwrap();
        assert_eq!(
            original.account().pubkey(),
            restored.account().pubkey(),
            "the phrase restores the account every note is signed by"
        );

        let other = Identity::generate().unwrap();
        assert_ne!(
            original.account().pubkey(),
            other.account().pubkey(),
            "another phrase is another account"
        );
    }

    #[test]
    fn a_note_this_account_signs_verifies_as_this_account() {
        let id = Identity::generate().unwrap();
        let note = crate::Note::write(id.account(), 1u16, 1_700_000_000, vec![], "hello").unwrap();
        assert!(note.verifies());
        assert_eq!(note.pubkey, id.account().pubkey());
    }

    #[test]
    fn a_bad_phrase_is_rejected() {
        assert!(Identity::from_phrase("not a real mnemonic phrase at all").is_err());
    }
}
