//! The `Authorization` header a celis route is called with (NIP-98): a kind 27235 note, base64, signed by
//! the account making the call and naming the url, the method and the sha256 of the body it carries.
//!
//! No account record is consulted at either end — the signature is the whole credential, so there is no
//! session to steal and nothing to expire but the note's own timestamp. Every route the network serves that
//! is ours rather than somebody's spec is called this way.
//!
//! [`header`] is how a signed note is spelled in that field. BUD-01 spells it the same way, so the blob
//! store's own authorization is the same encoding over a different note.

use crate::seconds;
use anyhow::Result;
use celis_nostr::{Account, Kind, Note, Tags};
use data_encoding::{BASE64, HEXLOWER};
use sha2::{Digest, Sha256};

/// The tag naming the call a note is one attempt of, so a host answers a second attempt from the first.
pub const IDEMPOTENCY: &str = "idempotency";

/// One request as the note names it: the method, the url it goes to, the bytes it carries, and the call it
/// is an attempt of where it is one that must not be served twice. `body` is empty for a request that
/// sends none.
pub struct Request<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub body: &'a str,
    pub key: Option<&'a str>,
}

/// A signed note authorizing `request`, as the `Authorization` header wants it.
pub fn authorization(account: &Account, request: &Request<'_>) -> Result<String> {
    let mut tags = Tags::new()
        .tag("u", request.url)
        .tag("method", request.method)
        .tag(
            "payload",
            HEXLOWER.encode(&Sha256::digest(request.body.as_bytes())),
        );
    if let Some(key) = request.key {
        tags = tags.tag(IDEMPOTENCY, key);
    }
    let note = Note::write(account, Kind::HttpAuth, seconds(), tags, String::new())?;
    header(&note)
}

/// `note` as an `Authorization` field spells it — the `Nostr` scheme over the note's own JSON, base64.
pub fn header(note: &Note) -> Result<String> {
    Ok(format!(
        "Nostr {}",
        BASE64.encode(serde_json::to_string(note)?.as_bytes())
    ))
}
