//! **The secret behind an identity** — the 32-byte secp256k1 key an account signs and decrypts with. It
//! never leaves the device that holds it. Everything the network sees is derived from it: the [`Pubkey`]
//! it publishes, the Schnorr signature on every note, and the shared secret that encrypts a note's content
//! to one reader.
//!
//! Encrypting to yourself is not a special case. Your own stream — everything this account authors, read by
//! its own devices — is this account encrypting to its own public key, which is why syncing between your
//! devices needs no key beyond the one the recovery phrase already restores.

use crate::nip44;
use crate::nip59::{self, Rumor};
use crate::note::Note;
use crate::pubkey::Pubkey;
use anyhow::{anyhow, bail, Result};
use bech32::{Bech32, Hrp};
use k256::schnorr::signature::hazmat::PrehashSigner;
use k256::schnorr::SigningKey;
use k256::SecretKey;

const NSEC: Hrp = Hrp::parse_unchecked("nsec");

/// One account's secret key: what it signs with, and what it derives a reader's shared secret from.
#[derive(Clone)]
pub struct Account {
    secret: SecretKey,
}

impl Account {
    /// The account these secret bytes name. Errors when they are not a valid scalar.
    pub fn new(secret: &[u8; 32]) -> Result<Self> {
        Ok(Self {
            secret: SecretKey::from_slice(secret).map_err(|e| anyhow!("not a secret key: {e}"))?,
        })
    }

    /// A fresh account nobody has ever held — the throwaway key that carries one gift
    /// wrap.
    pub fn generate() -> Result<Self> {
        let mut secret = [0u8; 32];
        getrandom::getrandom(&mut secret).map_err(|e| anyhow!("no randomness: {e}"))?;
        Self::new(&secret)
    }

    /// The account an `nsec` names (NIP-19) — the form a person exports and imports.
    pub fn parse_nsec(nsec: &str) -> Result<Self> {
        let (hrp, bytes) = bech32::decode(nsec).map_err(|e| anyhow!("not bech32: {e}"))?;
        if hrp != NSEC {
            bail!("not an nsec");
        }
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow!("a secret key is 32 bytes"))?;
        Self::new(&bytes)
    }

    /// This account's secret as an `nsec`. Whoever reads it becomes this account.
    pub fn nsec(&self) -> String {
        bech32::encode::<Bech32>(NSEC, &self.secret.to_bytes())
            .expect("an nsec is well under the length limit")
    }

    /// The secret itself, for the one caller that has to store it.
    pub fn secret(&self) -> [u8; 32] {
        self.secret.to_bytes().into()
    }

    /// The public identity everything is addressed to.
    pub fn pubkey(&self) -> Pubkey {
        let verifying = *SigningKey::from(&self.secret).verifying_key();
        Pubkey::new(verifying.to_bytes().into()).expect("a signing key has a public key")
    }

    /// Sign a 32-byte `message` — BIP-340 Schnorr over the bytes themselves, which for a note means its
    /// id. Nostr signs the id directly, so nothing is hashed again here.
    pub fn sign(&self, message: &[u8; 32]) -> Result<[u8; 64]> {
        Ok(SigningKey::from(&self.secret)
            .sign_prehash(message)
            .map_err(|e| anyhow!("signing failed: {e}"))?
            .to_bytes())
    }

    /// Encrypt `plaintext` so only `reader` can open it — NIP-44 v2, with a fresh nonce each call.
    pub fn encrypt(&self, reader: &Pubkey, plaintext: &str) -> Result<String> {
        let mut nonce = [0u8; 32];
        getrandom::getrandom(&mut nonce).map_err(|e| anyhow!("no randomness: {e}"))?;
        nip44::encrypt(
            &self.conversation_key(reader)?,
            &nonce,
            plaintext.as_bytes(),
        )
    }

    /// Encrypt `plaintext` so only `reader` can open it, the same way every time — the same plaintext gives
    /// the same ciphertext, byte for byte, on this device and on every other holding this key.
    ///
    /// **For plaintext that is already addressed by its own content, and nothing else.** What this gives up
    /// against [`encrypt`](Self::encrypt) is that two payloads carrying the same thing look alike, so it is
    /// only safe where that is already said out loud — a log entry travels under a `d` tag that is the hash
    /// of the entry, which announces it. A message, an invite or a receipt keeps a fresh nonce.
    ///
    /// What it buys is a note that need not be stored: every field of a note is a function of the entry it
    /// carries once the ciphertext is, so a device recomputes the note rather than remembering it, and two
    /// devices offering one entry offer one note.
    pub fn encrypt_deterministic(&self, reader: &Pubkey, plaintext: &str) -> Result<String> {
        let key = self.conversation_key(reader)?;
        let nonce = nip44::nonce_for(&key, plaintext.as_bytes());
        nip44::encrypt(&key, &nonce, plaintext.as_bytes())
    }

    /// Open what `writer` encrypted to this account. Errors on a wrong key, a tampered payload, or a
    /// version this does not speak.
    pub fn decrypt(&self, writer: &Pubkey, payload: &str) -> Result<String> {
        let plaintext = nip44::decrypt(&self.conversation_key(writer)?, payload)?;
        String::from_utf8(plaintext).map_err(|e| anyhow!("not utf-8: {e}"))
    }

    /// Wrap `content` so only `reader` can open it, and only they learn who sent it (NIP-59). `kind` says
    /// which of the sender's messages this is; it stays inside the ciphertext. `expires_at` is the second
    /// after which the relay stops serving it (NIP-40), `None` for a message that stands until answered.
    pub fn wrap(
        &self,
        reader: &Pubkey,
        kind: u16,
        created_at: u64,
        expires_at: Option<u64>,
        content: &str,
    ) -> Result<Note> {
        nip59::wrap(self, reader, kind, created_at, expires_at, content)
    }

    /// Open a gift wrap addressed to this account, answering what it carries and the author it proves.
    pub fn unwrap(&self, wrap: &Note) -> Result<Rumor> {
        nip59::unwrap(self, wrap)
    }

    /// The long-lived secret this account shares with `peer` — the same value from either side, so a
    /// writer and a reader derive it independently and it is never sent.
    pub(crate) fn conversation_key(&self, peer: &Pubkey) -> Result<[u8; 32]> {
        let shared =
            k256::ecdh::diffie_hellman(self.secret.to_nonzero_scalar(), peer.point()?.as_affine());
        let mut x = [0u8; 32];
        x.copy_from_slice(shared.raw_secret_bytes());
        Ok(nip44::conversation_key(&x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(byte: u8) -> Account {
        Account::new(&[byte; 32]).unwrap()
    }

    #[test]
    fn a_secret_reads_the_same_as_an_nsec() {
        let mut secret = [0u8; 32];
        secret[31] = 1;
        let account = Account::new(&secret).unwrap();
        assert_eq!(
            account.nsec(),
            "nsec1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsmhltgl",
            "the nsec other clients show for the scalar 1"
        );
        assert_eq!(
            Account::parse_nsec(&account.nsec()).unwrap().pubkey(),
            account.pubkey()
        );
    }

    #[test]
    fn a_signature_verifies_against_the_account_that_made_it_and_no_other() {
        let mine = account(3);
        let theirs = account(4);
        let message = [7u8; 32];
        let signature = mine.sign(&message).unwrap();

        assert!(mine.pubkey().verifies(&message, &signature));
        assert!(!theirs.pubkey().verifies(&message, &signature));
        assert!(!mine.pubkey().verifies(&[8u8; 32], &signature));
    }

    #[test]
    fn two_accounts_derive_the_same_conversation_key_from_opposite_sides() {
        let mine = account(3);
        let theirs = account(4);
        assert_eq!(
            mine.conversation_key(&theirs.pubkey()).unwrap(),
            theirs.conversation_key(&mine.pubkey()).unwrap()
        );
    }

    #[test]
    fn a_reader_opens_what_was_encrypted_to_them_and_a_stranger_does_not() {
        let mine = account(3);
        let theirs = account(4);
        let stranger = account(5);

        let payload = mine.encrypt(&theirs.pubkey(), "the entry").unwrap();
        assert_eq!(
            theirs.decrypt(&mine.pubkey(), &payload).unwrap(),
            "the entry"
        );
        assert!(stranger.decrypt(&mine.pubkey(), &payload).is_err());
    }

    #[test]
    fn encrypting_to_yourself_is_the_ordinary_case_with_no_extra_key() {
        let mine = account(3);
        let payload = mine.encrypt(&mine.pubkey(), "a fleet entry").unwrap();
        assert_eq!(
            mine.decrypt(&mine.pubkey(), &payload).unwrap(),
            "a fleet entry"
        );
    }

    #[test]
    fn the_same_plaintext_encrypts_differently_every_time() {
        let mine = account(3);
        let reader = account(4).pubkey();
        assert_ne!(
            mine.encrypt(&reader, "same words").unwrap(),
            mine.encrypt(&reader, "same words").unwrap(),
            "a fresh nonce each call"
        );
    }

    #[test]
    /// The deterministic path answers the same bytes every time, and from any device holding the key — which
    /// is what lets a note be recomputed rather than kept.
    fn the_same_plaintext_always_encrypts_the_same_way_deterministically() {
        let mine = account(3);
        let elsewhere = account(3);
        let reader = account(4).pubkey();

        let once = mine.encrypt_deterministic(&reader, "same words").unwrap();
        assert_eq!(
            once,
            mine.encrypt_deterministic(&reader, "same words").unwrap()
        );
        assert_eq!(
            once,
            elsewhere
                .encrypt_deterministic(&reader, "same words")
                .unwrap(),
            "another device holding the same key derives the same ciphertext"
        );
        assert_ne!(
            once,
            mine.encrypt_deterministic(&reader, "other words").unwrap(),
            "a different plaintext takes a different nonce"
        );
        assert_eq!(
            account(4).decrypt(&mine.pubkey(), &once).unwrap(),
            "same words",
            "and it is still an ordinary NIP-44 payload the reader opens"
        );
    }

    #[test]
    /// A note signed twice over one id is the same note, so deriving one rather than keeping it does not
    /// hand two relays two signatures for one address.
    fn signing_the_same_id_twice_gives_the_same_signature() {
        let mine = account(3);
        assert_eq!(
            mine.sign(&[7u8; 32]).unwrap(),
            mine.sign(&[7u8; 32]).unwrap()
        );
    }
}
