//! The secret behind an identity — the 32-byte secp256k1 key an account signs and decrypts with. It never
//! leaves the device that holds it. Everything the network sees is derived from it: the [`Pubkey`] it
//! publishes, the Schnorr signature on every note, and the shared secret that encrypts a note's content to
//! one reader.

use crate::nip44;
use crate::pubkey::Pubkey;
use anyhow::{anyhow, bail, Result};
use bech32::{Bech32, Hrp};
use hkdf::Hkdf;
use k256::schnorr::signature::hazmat::PrehashSigner;
use k256::schnorr::SigningKey;
use k256::SecretKey;
use sha2::Sha256;

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

    /// A fresh account nobody has ever held — the throwaway key that carries one gift wrap.
    pub fn generate() -> Result<Self> {
        let mut secret = [0u8; 32];
        getrandom::getrandom(&mut secret).map_err(|e| anyhow!("no randomness: {e}"))?;
        Self::new(&secret)
    }

    /// The key this account derives from `nonce` — a second identity of its own, recoverable from the
    /// recovery phrase and this nonce alone. **The nonce is not a secret**: it is what makes two derivations
    /// of one account different, and the account key it is mixed with is the secret.
    pub fn derive(&self, nonce: &[u8]) -> Result<Self> {
        let mut derived = [0u8; 32];
        Hkdf::<Sha256>::new(Some(nonce), &self.secret.to_bytes())
            .expand(b"celis/server", &mut derived)
            .map_err(|e| anyhow!("cannot derive a key: {e}"))?;
        Self::new(&derived)
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

    /// Encrypt `plaintext` so only `reader` can open it, the same way every time. Safe only where the
    /// plaintext is already addressed by its own content — a log entry under a `d` tag that is the hash of
    /// the entry.
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
mod tests;
