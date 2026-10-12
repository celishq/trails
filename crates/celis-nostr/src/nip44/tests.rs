//! The padding and the payload bounds, then the specification's own test vectors run against this
//! implementation — `vectors/nip44-v2.json`, taken unmodified from the NIP-44 reference repository. Every
//! key, padding length and payload in it was produced by other people's code, so agreeing with all of them
//! means a note we encrypt is one any client can read.

use super::*;
use crate::account::Account;
use crate::pubkey::Pubkey;
use data_encoding::HEXLOWER;
use serde_json::Value;
use sha2::Digest;

const VECTORS: &str = include_str!("../../vectors/nip44-v2.json");

fn v2() -> Value {
    serde_json::from_str::<Value>(VECTORS).expect("the vectors parse")["v2"].clone()
}

fn hex32(value: &Value, field: &str) -> [u8; 32] {
    HEXLOWER
        .decode(value[field].as_str().expect("a hex field").as_bytes())
        .expect("valid hex")
        .try_into()
        .expect("32 bytes")
}

fn account(value: &Value, field: &str) -> Account {
    Account::new(&hex32(value, field)).expect("a valid secret key")
}

#[test]
fn every_conversation_key_matches() {
    let vectors = v2();
    let cases = vectors["valid"]["get_conversation_key"]
        .as_array()
        .expect("a list");
    for case in cases {
        let mine = account(case, "sec1");
        let theirs = Pubkey::parse(case["pub2"].as_str().expect("a hex key")).expect("a valid key");
        assert_eq!(
            mine.conversation_key(&theirs).expect("a shared secret"),
            hex32(case, "conversation_key"),
        );
    }
    assert_eq!(cases.len(), 35, "every case ran");
}

#[test]
fn every_message_key_matches() {
    let vectors = v2();
    let group = &vectors["valid"]["get_message_keys"];
    let conversation_key = hex32(group, "conversation_key");
    let cases = group["keys"].as_array().expect("a list");
    for case in cases {
        let keys = keys(&conversation_key, &hex32(case, "nonce"));
        assert_eq!(keys.cipher, hex32(case, "chacha_key"));
        assert_eq!(
            keys.nonce.to_vec(),
            HEXLOWER
                .decode(case["chacha_nonce"].as_str().expect("hex").as_bytes())
                .expect("valid hex"),
        );
        assert_eq!(keys.mac, hex32(case, "hmac_key"));
    }
    assert_eq!(cases.len(), 32, "every case ran");
}

#[test]
fn every_padded_length_matches() {
    let vectors = v2();
    let cases = vectors["valid"]["calc_padded_len"]
        .as_array()
        .expect("a list");
    for case in cases {
        let unpadded = case[0].as_u64().expect("a length") as usize;
        let expected = case[1].as_u64().expect("a length") as usize;
        assert_eq!(padded_len(unpadded), expected, "padding {unpadded}");
    }
    assert_eq!(cases.len(), 24, "every case ran");
}

#[test]
fn every_payload_encrypts_and_decrypts_to_the_stated_bytes() {
    let vectors = v2();
    let cases = vectors["valid"]["encrypt_decrypt"]
        .as_array()
        .expect("a list");
    for case in cases {
        let writer = account(case, "sec1");
        let reader = account(case, "sec2");
        let conversation_key = hex32(case, "conversation_key");
        let plaintext = case["plaintext"].as_str().expect("a plaintext");
        let payload = case["payload"].as_str().expect("a payload");

        assert_eq!(
            writer
                .conversation_key(&reader.pubkey())
                .expect("a shared secret"),
            conversation_key,
        );
        assert_eq!(
            encrypt(
                &conversation_key,
                &hex32(case, "nonce"),
                plaintext.as_bytes()
            )
            .expect("it encrypts"),
            payload,
        );
        assert_eq!(
            reader
                .decrypt(&writer.pubkey(), payload)
                .expect("it decrypts"),
            plaintext,
        );
    }
    assert_eq!(cases.len(), 10, "every case ran");
}

#[test]
fn a_long_message_encrypts_to_the_stated_payload() {
    let vectors = v2();
    let cases = vectors["valid"]["encrypt_decrypt_long_msg"]
        .as_array()
        .expect("a list");
    for case in cases {
        let pattern = case["pattern"].as_str().expect("a pattern");
        let repeat = case["repeat"].as_u64().expect("a count") as usize;
        let plaintext = pattern.repeat(repeat);
        assert_eq!(
            HEXLOWER.encode(&Sha256::digest(plaintext.as_bytes())),
            case["plaintext_sha256"].as_str().expect("a digest"),
        );

        let conversation_key = hex32(case, "conversation_key");
        let payload = encrypt(
            &conversation_key,
            &hex32(case, "nonce"),
            plaintext.as_bytes(),
        )
        .expect("it encrypts");
        assert_eq!(
            HEXLOWER.encode(&Sha256::digest(payload.as_bytes())),
            case["payload_sha256"].as_str().expect("a digest"),
        );
        assert_eq!(
            decrypt(&conversation_key, &payload).expect("it decrypts"),
            plaintext.as_bytes(),
        );
    }
    assert_eq!(cases.len(), 3, "every case ran");
}

#[test]
fn every_bad_payload_is_refused() {
    let vectors = v2();
    let cases = vectors["invalid"]["decrypt"].as_array().expect("a list");
    for case in cases {
        let conversation_key = hex32(case, "conversation_key");
        let payload = case["payload"].as_str().expect("a payload");
        assert!(
            decrypt(&conversation_key, payload).is_err(),
            "{}",
            case["note"].as_str().unwrap_or("should be refused"),
        );
    }
    assert_eq!(cases.len(), 12, "every case ran");
}

#[test]
fn every_bad_key_pair_is_refused() {
    let vectors = v2();
    let cases = vectors["invalid"]["get_conversation_key"]
        .as_array()
        .expect("a list");
    for case in cases {
        let secret = HEXLOWER
            .decode(case["sec1"].as_str().expect("hex").as_bytes())
            .ok()
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .and_then(|bytes| Account::new(&bytes).ok());
        let public = Pubkey::parse(case["pub2"].as_str().expect("hex")).ok();
        let shared = secret
            .zip(public)
            .and_then(|(secret, public)| secret.conversation_key(&public).ok());
        assert!(
            shared.is_none(),
            "{}",
            case["note"].as_str().unwrap_or("should be refused"),
        );
    }
    assert_eq!(cases.len(), 8, "every case ran");
}

#[test]
fn every_bad_plaintext_length_is_refused() {
    let vectors = v2();
    let cases = vectors["invalid"]["encrypt_msg_lengths"]
        .as_array()
        .expect("a list");
    for case in cases {
        let length = case.as_u64().expect("a length") as usize;
        assert!(
            pad(&vec![b'x'; length]).is_err(),
            "a {length}-byte plaintext"
        );
    }
    assert_eq!(cases.len(), 4, "every case ran");
}

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
