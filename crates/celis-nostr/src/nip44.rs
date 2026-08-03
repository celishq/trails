//! **NIP-44 v2** — how a note's content is encrypted to one reader. Two accounts derive a conversation key
//! from their shared Diffie-Hellman secret once; every message then mints its own keys from that plus a
//! fresh 32-byte nonce, encrypts with ChaCha20 and authenticates with HMAC-SHA256 over the nonce and the
//! ciphertext.
//!
//! Plaintext is padded to a power-of-two-ish length before encryption, so the ciphertext reveals only a
//! coarse size band rather than the exact length of what was written. The whole thing is checked against
//! the specification's own test vectors — see the `vectors` module.

use anyhow::{anyhow, bail, Result};
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use data_encoding::BASE64;
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[cfg(test)]
mod vectors;

const VERSION: u8 = 2;
const SALT: &[u8] = b"nip44-v2";
const MAX_PLAINTEXT: usize = 65535;
/// The version byte, a nonce, the shortest padded plaintext there is, and the MAC.
const MIN_PAYLOAD: usize = 1 + 32 + (2 + 32) + 32;
/// The same, with the longest padded plaintext there is.
const MAX_PAYLOAD: usize = 1 + 32 + (2 + 65536) + 32;

/// The long-lived secret two accounts share, from the x-coordinate of their Diffie-Hellman point.
pub(crate) fn conversation_key(shared_x: &[u8; 32]) -> [u8; 32] {
    let (prk, _) = Hkdf::<Sha256>::extract(Some(SALT), shared_x);
    prk.into()
}

/// The nonce a plaintext gets when it is to encrypt the same way every time: `HMAC(conversation_key,
/// plaintext)`.
///
/// **Only for plaintext that is already addressed by its own content.** Reusing a nonce across two
/// *different* plaintexts under one key repeats the keystream and lets xoring the two ciphertexts strip it —
/// the failure that makes a random nonce the default. That cannot happen here: two plaintexts share this
/// nonce only if they collide under HMAC-SHA256, which is to say if they are the same plaintext.
///
/// What it does change is that the same plaintext gives the same ciphertext, so an observer can tell two
/// payloads carry the same thing. Use it where that is already announced — a note whose `d` tag is the hash
/// of what it carries — and nowhere else. Keyed rather than a plain hash of the plaintext, so nobody without
/// the conversation key can test a guess against the nonce.
pub(crate) fn nonce_for(conversation_key: &[u8; 32], plaintext: &[u8]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(conversation_key)
        .expect("hmac takes a key of any length");
    mac.update(plaintext);
    mac.finalize().into_bytes().into()
}

/// One message's keys, mint fresh from the conversation key and this message's nonce.
struct Keys {
    cipher: [u8; 32],
    nonce: [u8; 12],
    mac: [u8; 32],
}

fn keys(conversation_key: &[u8; 32], nonce: &[u8; 32]) -> Keys {
    let mut expanded = [0u8; 76];
    Hkdf::<Sha256>::from_prk(conversation_key)
        .expect("a conversation key is a full-length prk")
        .expand(nonce, &mut expanded)
        .expect("76 bytes is within HKDF's output limit");
    Keys {
        cipher: expanded[..32].try_into().expect("32 bytes"),
        nonce: expanded[32..44].try_into().expect("12 bytes"),
        mac: expanded[44..].try_into().expect("32 bytes"),
    }
}

/// The length plaintext of `unpadded` bytes is padded out to — coarse bands, so a ciphertext's size says
/// little about what it holds.
fn padded_len(unpadded: usize) -> usize {
    if unpadded <= 32 {
        return 32;
    }
    let next_power = 1usize << (usize::BITS - (unpadded - 1).leading_zeros());
    let chunk = if next_power <= 256 {
        32
    } else {
        next_power / 8
    };
    chunk * ((unpadded - 1) / chunk + 1)
}

fn pad(plaintext: &[u8]) -> Result<Vec<u8>> {
    if plaintext.is_empty() || plaintext.len() > MAX_PLAINTEXT {
        bail!("plaintext must be 1..=65535 bytes");
    }
    let mut padded = Vec::with_capacity(2 + padded_len(plaintext.len()));
    padded.extend_from_slice(&(plaintext.len() as u16).to_be_bytes());
    padded.extend_from_slice(plaintext);
    padded.resize(2 + padded_len(plaintext.len()), 0);
    Ok(padded)
}

fn unpad(padded: &[u8]) -> Result<Vec<u8>> {
    let stated = padded
        .get(..2)
        .map(|len| u16::from_be_bytes([len[0], len[1]]) as usize)
        .ok_or_else(|| anyhow!("invalid padding"))?;
    let plaintext = padded
        .get(2..2 + stated)
        .ok_or_else(|| anyhow!("invalid padding"))?;
    if stated == 0 || padded.len() != 2 + padded_len(stated) {
        bail!("invalid padding");
    }
    Ok(plaintext.to_vec())
}

fn tag(key: &[u8; 32], nonce: &[u8; 32], ciphertext: &[u8]) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts a key of any length");
    mac.update(nonce);
    mac.update(ciphertext);
    mac
}

/// Encrypt `plaintext` under `conversation_key` with this message's `nonce`, as a base64 payload.
pub(crate) fn encrypt(
    conversation_key: &[u8; 32],
    nonce: &[u8; 32],
    plaintext: &[u8],
) -> Result<String> {
    let keys = keys(conversation_key, nonce);
    let mut ciphertext = pad(plaintext)?;
    ChaCha20::new(&keys.cipher.into(), &keys.nonce.into()).apply_keystream(&mut ciphertext);

    let mut payload = Vec::with_capacity(1 + 32 + ciphertext.len() + 32);
    payload.push(VERSION);
    payload.extend_from_slice(nonce);
    payload.extend_from_slice(&ciphertext);
    payload.extend_from_slice(&tag(&keys.mac, nonce, &ciphertext).finalize().into_bytes());
    Ok(BASE64.encode(&payload))
}

/// Open a payload under `conversation_key`. Errors on a wrong key, a tampered payload, or a version this
/// does not speak — the MAC is checked before anything is decrypted.
pub(crate) fn decrypt(conversation_key: &[u8; 32], payload: &str) -> Result<Vec<u8>> {
    if payload.starts_with('#') {
        bail!("unknown encryption version");
    }
    let bytes = BASE64
        .decode(payload.as_bytes())
        .map_err(|e| anyhow!("invalid base64: {e}"))?;
    if !(MIN_PAYLOAD..=MAX_PAYLOAD).contains(&bytes.len()) {
        bail!("invalid payload size");
    }
    if bytes[0] != VERSION {
        bail!("unknown encryption version");
    }

    let nonce: [u8; 32] = bytes[1..33].try_into().expect("32 bytes");
    let (ciphertext, mac) = bytes[33..].split_at(bytes.len() - 33 - 32);
    let keys = keys(conversation_key, &nonce);
    tag(&keys.mac, &nonce, ciphertext)
        .verify_slice(mac)
        .map_err(|_| anyhow!("invalid mac"))?;

    let mut plaintext = ciphertext.to_vec();
    ChaCha20::new(&keys.cipher.into(), &keys.nonce.into()).apply_keystream(&mut plaintext);
    unpad(&plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_falls_into_coarse_bands() {
        assert_eq!(padded_len(1), 32);
        assert_eq!(padded_len(32), 32);
        assert_eq!(padded_len(33), 64);
        assert_eq!(padded_len(37), 64);
        assert_eq!(padded_len(45), 64);
        assert_eq!(padded_len(65), 96);
        assert_eq!(padded_len(1000), 1024);
        assert_eq!(padded_len(65535), 65536);
    }

    #[test]
    fn padding_round_trips_and_a_wrong_length_is_refused() {
        let padded = pad(b"a message").unwrap();
        assert_eq!(padded.len(), 34);
        assert_eq!(unpad(&padded).unwrap(), b"a message");

        let mut lying = padded.clone();
        lying[1] = 200;
        assert!(unpad(&lying).is_err(), "a length past the buffer");
        assert!(unpad(&[0, 0, 0, 0]).is_err(), "a zero-length plaintext");
    }

    #[test]
    fn an_empty_or_oversized_plaintext_is_refused() {
        assert!(pad(b"").is_err());
        assert!(pad(&vec![0u8; MAX_PLAINTEXT + 1]).is_err());
    }
}
