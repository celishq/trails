//! The `Authorization` header a celis route is called with (NIP-98): a kind 27235 note, base64, signed by
//! the account making the call and naming the url, the method and the sha256 of the body it carries.
//!
//! No account record is consulted at either end — the signature is the whole credential, so there is no
//! session to steal and nothing to expire but the note's own timestamp. Every route the network serves that
//! is ours rather than somebody's spec is called this way.

use anyhow::Result;
use celis_nostr::{Account, Kind, Note, Tags};
use data_encoding::{BASE64, HEXLOWER};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// A signed note authorizing one request — the method, the url it goes to, and the bytes it carries — as the
/// `Authorization` header wants it. `body` is empty for a request that sends none.
pub fn authorization(account: &Account, method: &str, url: &str, body: &str) -> Result<String> {
    let note = Note::write(
        account,
        Kind::HttpAuth,
        seconds(),
        Tags::new()
            .tag("u", url)
            .tag("method", method)
            .tag("payload", HEXLOWER.encode(&Sha256::digest(body.as_bytes()))),
        String::new(),
    )?;
    Ok(format!(
        "Nostr {}",
        BASE64.encode(serde_json::to_string(&note)?.as_bytes())
    ))
}

/// Unix seconds now — the note's `created_at`, which the worker checks against its ±300s window.
fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
