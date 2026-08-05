//! **Who someone is** — 32 bytes of x-only secp256k1, the whole of a nostr identity. It is written as
//! lower-hex on the wire and as an `npub` to a person (NIP-19), and it is the only thing needed to check
//! that a note is genuine or to encrypt something only its holder can read.

use anyhow::{anyhow, bail, Result};
use bech32::{Bech32, Hrp};
use data_encoding::HEXLOWER;
use k256::schnorr::signature::hazmat::PrehashVerifier;
use k256::schnorr::{Signature, VerifyingKey};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const NPUB: Hrp = Hrp::parse_unchecked("npub");

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Pubkey([u8; 32]);

impl Pubkey {
    /// The key these bytes name. Errors when they are not a point on the curve.
    pub fn new(bytes: [u8; 32]) -> Result<Self> {
        VerifyingKey::from_bytes(&bytes).map_err(|e| anyhow!("not a public key: {e}"))?;
        Ok(Self(bytes))
    }

    /// The key written as 64 lower-hex characters — the form every relay uses.
    pub fn parse(hex: &str) -> Result<Self> {
        let bytes = HEXLOWER
            .decode(hex.as_bytes())
            .map_err(|e| anyhow!("not hex: {e}"))?;
        Self::new(
            bytes
                .try_into()
                .map_err(|_| anyhow!("a public key is 32 bytes"))?,
        )
    }

    /// The key written as an `npub` — the form a person copies and pastes (NIP-19).
    pub fn parse_npub(npub: &str) -> Result<Self> {
        let (hrp, bytes) = bech32::decode(npub).map_err(|e| anyhow!("not bech32: {e}"))?;
        if hrp != NPUB {
            bail!("not an npub");
        }
        Self::new(
            bytes
                .try_into()
                .map_err(|_| anyhow!("a public key is 32 bytes"))?,
        )
    }

    pub fn hex(&self) -> String {
        HEXLOWER.encode(&self.0)
    }

    pub fn npub(&self) -> String {
        bech32::encode::<Bech32>(NPUB, &self.0).expect("an npub is well under the length limit")
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Whether `signature` over the 32-byte `message` is this account's — BIP-340 Schnorr over the bytes
    /// themselves, as NIP-01 requires.
    pub fn verifies(&self, message: &[u8; 32], signature: &[u8]) -> bool {
        let Ok(key) = VerifyingKey::from_bytes(&self.0) else {
            return false;
        };
        let Ok(signature) = Signature::try_from(signature) else {
            return false;
        };
        key.verify_prehash(message, &signature).is_ok()
    }

    /// The key as an SEC1 point with an even y — how BIP-340 lifts an x-only key, and what NIP-44's
    /// Diffie-Hellman needs.
    pub(crate) fn point(&self) -> Result<k256::PublicKey> {
        let mut sec1 = [0u8; 33];
        sec1[0] = 0x02;
        sec1[1..].copy_from_slice(&self.0);
        k256::PublicKey::from_sec1_bytes(&sec1).map_err(|e| anyhow!("not a public key: {e}"))
    }
}

impl Serialize for Pubkey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.hex())
    }
}

impl<'de> Deserialize<'de> for Pubkey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        Pubkey::parse(&hex).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fiatjaf's key, from a note that has been on the network since 2022.
    const HEX: &str = "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d";

    #[test]
    fn a_key_reads_the_same_in_hex_and_as_an_npub() {
        let key = Pubkey::parse(HEX).unwrap();
        assert_eq!(key.hex(), HEX);
        assert_eq!(
            key.npub(),
            "npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6",
            "the npub other clients show for this key"
        );
        assert_eq!(Pubkey::parse_npub(&key.npub()).unwrap(), key);
    }

    #[test]
    fn a_key_that_is_not_on_the_curve_is_refused() {
        assert!(Pubkey::new([0u8; 32]).is_err());
        assert!(Pubkey::parse("not hex").is_err());
        assert!(Pubkey::parse(&HEX[..62]).is_err(), "too short");
        assert!(Pubkey::parse_npub(
            "nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5"
        )
        .is_err());
    }

    #[test]
    fn a_key_survives_a_json_round_trip_as_hex() {
        let key = Pubkey::parse(HEX).unwrap();
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, format!("\"{HEX}\""));
        assert_eq!(serde_json::from_str::<Pubkey>(&json).unwrap(), key);
    }
}
