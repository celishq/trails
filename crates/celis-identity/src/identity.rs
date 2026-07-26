//! The **identity** — the account root: the P-256 signing key and the content key, both derived from one
//! BIP39 recovery phrase, so the phrase is the single backup that restores who you are *and* the ability to
//! decrypt. The root never leaves the owner's control; it exists to derive the [`Principal`], sign device
//! [`Delegation`]s, and hold the [`Cek`]. Devices get their own Secure-Enclave keys via delegation, never
//! this key.

use crate::delegation::Delegation;
use crate::envelope::{Cek, Envelope};
use crate::keyslot::Keyslot;
use crate::principal::Principal;
use crate::sealed::{Keyset, Sealed};
use anyhow::{anyhow, bail, Result};
use bip39::Mnemonic;
use data_encoding::HEXLOWER;
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use sha2::Sha256;

/// The HKDF salt binding every derivation to this application, so the same phrase used elsewhere never
/// yields these keys.
const SALT: &[u8] = b"celis-identity-v1";
/// The HKDF info label for the root signing key.
const ROOT_INFO: &[u8] = b"root-p256";
/// The HKDF info label for the content-encryption key.
const CEK_INFO: &[u8] = b"cek-aes256";
/// The HKDF info label for the vault key that keys sync-tier content ids — a MAC key distinct from the
/// content-encryption key ([`CEK_INFO`]).
const VAULT_ID_INFO: &[u8] = b"vault-id-hmac";
/// The HKDF info label for the sealing key — the ECDH keypair whose public half the account publishes in
/// its profile, kept separate from the signing root ([`ROOT_INFO`]) so a compromise of one role never
/// touches the other.
const SEAL_INFO: &[u8] = b"seal-p256";

/// An account root derived from a recovery phrase: the root signing key, its recovery `mnemonic` (to show
/// the user for backup), and the vault content key. Deterministic — the same phrase always rebuilds the
/// same identity.
pub struct Identity {
    mnemonic: Mnemonic,
    root: SigningKey,
    cek: Cek,
    vault_key: [u8; 32],
    seal: p256::SecretKey,
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

    /// The recovery phrase — the one backup the user must record. Restores identity and content key alike.
    pub fn phrase(&self) -> String {
        self.mnemonic.to_string()
    }

    /// This account's principal.
    pub fn principal(&self) -> Principal {
        Principal::of(self.public().to_encoded_point(true).as_bytes())
    }

    /// The root public key — the principal's basis and every delegation's issuer.
    pub fn public(&self) -> VerifyingKey {
        *self.root.verifying_key()
    }

    /// Sign `msg` with the root key (delegation issuance and root-authenticated registration).
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.root.sign(msg)
    }

    /// The vault content key.
    pub fn cek(&self) -> &Cek {
        &self.cek
    }

    /// Seal `plaintext` to the private vault ([`Keyset::SelfVault`]) — the seed content key wraps the
    /// body, no key-slots, no signature (the vault has no foreign reader). The reconciliation set-union
    /// key is the orthogonal [`content_id`](Self::content_id); the envelope is fresh each call (random
    /// nonce).
    pub fn seal_self(&self, plaintext: &[u8]) -> Result<Sealed> {
        Sealed::seal(plaintext, Keyset::SelfVault(&self.cek), None)
    }

    /// Open a vault [`Sealed`] back to plaintext; errors on a wrong key or tampered bytes (the GCM tag).
    pub fn open_self(&self, sealed: &Sealed) -> Result<Vec<u8>> {
        if sealed.body_enc.len() < 12 {
            bail!("truncated sealed body");
        }
        let (nonce, ciphertext) = sealed.body_enc.split_at(12);
        self.cek.open(&Envelope {
            nonce: nonce.to_vec(),
            ciphertext: ciphertext.to_vec(),
        })
    }

    /// Recover a shared cell's body from its `body_enc` and keyring `slots` (`spec/cell.md`)
    /// using this account's sealing key: `Some(body)` when a slot seals to this account, `None` when
    /// it is not a recipient, `Err` on a tampered body.
    pub fn open_shared(&self, body_enc: &[u8], slots: &[Keyslot]) -> Result<Option<Vec<u8>>> {
        crate::keyslot::open_body(body_enc, slots, &self.seal)
    }

    /// The per-vault content id of `plaintext` — HMAC-SHA256 under the vault key, lower-hex. Deterministic
    /// across this vault's devices (shared key) yet uncorrelatable and unconfirmable by the keyless network:
    /// not a bare `sha256(plaintext)` fingerprint.
    pub fn content_id(&self, plaintext: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.vault_key)
            .expect("HMAC accepts a key of any length");
        mac.update(plaintext);
        HEXLOWER.encode(&mac.finalize().into_bytes())
    }

    /// Delegate authority to a device key — the root grants the device the given capabilities until `expiry`
    /// (Unix seconds; `0` = no expiry).
    pub fn delegate(
        &self,
        device: &VerifyingKey,
        capabilities: Vec<String>,
        expiry: i64,
    ) -> Delegation {
        Delegation::issue(&self.root, device, capabilities, expiry)
    }

    /// The root public key as SEC1-compressed bytes — for callers that speak bytes, not `p256` types
    /// (the FFI seam, registration, peer delegation checks).
    pub fn public_sec1(&self) -> Vec<u8> {
        self.public().to_encoded_point(true).as_bytes().to_vec()
    }

    /// The sealing public key as SEC1-compressed bytes — published in the account's public profile; peers
    /// ECDH against it so a contribution is openable only by this account. Not the signing root.
    pub fn sealing_public_sec1(&self) -> Vec<u8> {
        self.seal
            .public_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }

    /// A root signature over `msg` as its fixed 64-byte form.
    pub fn sign_bytes(&self, msg: &[u8]) -> Vec<u8> {
        self.sign(msg).to_bytes().to_vec()
    }

    /// Delegate to a device key given its SEC1-compressed bytes; errors on a malformed key.
    pub fn delegate_sec1(
        &self,
        device_sec1: &[u8],
        capabilities: Vec<String>,
        expiry: i64,
    ) -> Result<Delegation> {
        let device = VerifyingKey::from_sec1_bytes(device_sec1)
            .map_err(|e| anyhow!("bad device key: {e}"))?;
        Ok(self.delegate(&device, capabilities, expiry))
    }

    /// Derive the root key and content key from a mnemonic's seed via HKDF-SHA256 under distinct labels.
    fn from_mnemonic(mnemonic: Mnemonic) -> Result<Self> {
        let seed = mnemonic.to_seed("");
        let hk = Hkdf::<Sha256>::new(Some(SALT), &seed);

        let mut root = [0u8; 32];
        hk.expand(ROOT_INFO, &mut root)
            .map_err(|e| anyhow!("root derive: {e}"))?;
        let root = SigningKey::from_slice(&root).map_err(|e| anyhow!("root scalar: {e}"))?;

        let mut cek = [0u8; 32];
        hk.expand(CEK_INFO, &mut cek)
            .map_err(|e| anyhow!("cek derive: {e}"))?;

        let mut vault_key = [0u8; 32];
        hk.expand(VAULT_ID_INFO, &mut vault_key)
            .map_err(|e| anyhow!("vault-id derive: {e}"))?;

        let mut seal = [0u8; 32];
        hk.expand(SEAL_INFO, &mut seal)
            .map_err(|e| anyhow!("seal derive: {e}"))?;
        let seal = p256::SecretKey::from_slice(&seal).map_err(|e| anyhow!("seal scalar: {e}"))?;

        Ok(Identity {
            mnemonic,
            root,
            cek: Cek::new(cek),
            vault_key,
            seal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delegation::authorize;
    use p256::ecdsa::signature::Signer;

    #[test]
    fn a_phrase_deterministically_rebuilds_the_same_sealing_key_distinct_from_the_root() {
        let original = Identity::generate().unwrap();
        let restored = Identity::from_phrase(&original.phrase()).unwrap();
        assert_eq!(
            original.sealing_public_sec1(),
            restored.sealing_public_sec1(),
            "sealing key is stable — the phrase restores the ability to be sealed to"
        );
        assert_ne!(
            original.sealing_public_sec1(),
            original.public_sec1(),
            "key separation — the sealing key is not the signing root"
        );
    }

    #[test]
    fn a_phrase_deterministically_rebuilds_the_same_principal_and_cek() {
        let original = Identity::generate().unwrap();
        let phrase = original.phrase();

        let restored = Identity::from_phrase(&phrase).unwrap();
        assert_eq!(
            original.principal(),
            restored.principal(),
            "principal is stable"
        );
        assert_eq!(
            original.cek().bytes(),
            restored.cek().bytes(),
            "content key is stable — the phrase restores decryption too"
        );
    }

    #[test]
    fn seal_open_round_trips() {
        let id = Identity::generate().unwrap();
        let sealed = id.seal_self(b"a log entry").unwrap();
        assert_eq!(id.open_self(&sealed).unwrap(), b"a log entry");
    }

    #[test]
    fn the_same_plaintext_seals_to_the_same_id_across_devices_and_reseals() {
        let phrase = Identity::generate().unwrap().phrase();
        let a = Identity::from_phrase(&phrase).unwrap();
        let b = Identity::from_phrase(&phrase).unwrap();
        let pt = b"cells/a.md revision";

        assert_eq!(
            a.content_id(pt),
            b.content_id(pt),
            "same vault key → same id across devices (union stays idempotent)"
        );
        let s1 = a.seal_self(pt).unwrap();
        let s2 = a.seal_self(pt).unwrap();
        assert_ne!(
            s1.body_enc[..12],
            s2.body_enc[..12],
            "a fresh nonce each seal (the 12-byte prefix)"
        );
        assert_eq!(b.open_self(&s1).unwrap(), pt, "either device opens it");
    }

    #[test]
    fn a_different_vault_yields_a_different_id_that_is_not_a_plaintext_hash() {
        use sha2::Digest;
        let a = Identity::generate().unwrap();
        let b = Identity::generate().unwrap();
        let pt = b"secret";

        assert_ne!(
            a.content_id(pt),
            b.content_id(pt),
            "another vault → an uncorrelatable id"
        );
        let bare = HEXLOWER.encode(&Sha256::digest(pt));
        assert_ne!(
            a.content_id(pt),
            bare,
            "keyed, not a bare sha256(plaintext) the network could confirm"
        );
    }

    #[test]
    fn a_different_phrase_yields_a_different_principal() {
        let a = Identity::generate().unwrap();
        let b = Identity::generate().unwrap();
        assert_ne!(a.principal(), b.principal());
    }

    #[test]
    fn a_bad_phrase_is_rejected() {
        assert!(Identity::from_phrase("not a real mnemonic phrase at all").is_err());
    }

    #[test]
    fn the_principal_string_round_trips_through_parse() {
        let id = Identity::generate().unwrap();
        let principal = id.principal();
        let parsed: Principal = principal.to_string().parse().unwrap();
        assert_eq!(principal, parsed);
    }

    #[test]
    fn a_mistyped_principal_fails_the_checksum() {
        let id = Identity::generate().unwrap();
        let mut text = id.principal().to_string();
        let last = text.pop().unwrap();
        text.push(if last == 'a' { 'b' } else { 'a' });
        assert!(
            text.parse::<Principal>().is_err(),
            "checksum catches the typo"
        );
    }

    #[test]
    /// A different message under the same signature fails authorization.
    fn a_delegated_device_request_authorizes_and_a_tampered_one_fails() {
        let id = Identity::generate().unwrap();
        let device = SigningKey::from_slice(&[3u8; 32]).unwrap();
        let delegation = id.delegate(device.verifying_key(), vec!["sync".into()], 0);

        let msg = b"GET /sync";
        let sig: Signature = device.sign(msg);
        assert!(authorize(
            &id.principal(),
            &id.public(),
            &delegation,
            0,
            msg,
            &sig
        ));

        assert!(!authorize(
            &id.principal(),
            &id.public(),
            &delegation,
            0,
            b"GET /other",
            &sig
        ));
    }

    #[test]
    fn an_expired_delegation_is_refused() {
        let id = Identity::generate().unwrap();
        let device = SigningKey::from_slice(&[5u8; 32]).unwrap();
        let delegation = id.delegate(device.verifying_key(), vec!["sync".into()], 100);
        let msg = b"GET /sync";
        let sig: Signature = device.sign(msg);
        assert!(
            authorize(&id.principal(), &id.public(), &delegation, 50, msg, &sig),
            "before expiry"
        );
        assert!(
            !authorize(&id.principal(), &id.public(), &delegation, 150, msg, &sig),
            "after expiry"
        );
    }

    #[test]
    /// An attacker delegating to a device while presenting the victim's principal is refused.
    fn a_delegation_from_another_root_is_refused() {
        let id = Identity::generate().unwrap();
        let attacker = Identity::generate().unwrap();
        let device = SigningKey::from_slice(&[7u8; 32]).unwrap();
        let forged = attacker.delegate(device.verifying_key(), vec!["sync".into()], 0);
        let msg = b"GET /sync";
        let sig: Signature = device.sign(msg);
        assert!(!authorize(
            &id.principal(),
            &attacker.public(),
            &forged,
            0,
            msg,
            &sig
        ));
    }
}
