//! **A relay** — where notes go and where they come back from. One websocket, four frames: `EVENT`
//! publishes, `REQ` asks, `EOSE` says the relay has sent everything it holds, `CLOSE` ends the
//! subscription. Nothing here decides anything about a note's meaning; a relay is storage that speaks.
//!
//! A relay holds its connection: every exchange with the same relay travels over one socket, opened the
//! first time it is needed and opened again if it goes away.
//!
//! **Authentication is answered, never offered** (NIP-42). A relay that wants to know who is asking sends a
//! challenge on connect and turns a frame down with `auth-required:`; only then does this client sign the
//! challenge and try the frame again.

use anyhow::{anyhow, bail, Result};
use celis_nostr::{Account, Answer, Filter, Information, Kind, Limitation, Note, Tags};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

/// How long one exchange waits on a relay that has stopped answering. A sync holds the binding's write
/// guard for its whole length, so this is what a stalled relay costs the person at the screen — well past
/// any answer a relay owes, and short enough that a dead one is a pause rather than a freeze.
const TIMEOUT: Duration = Duration::from_secs(15);

/// What a relay prefixes a refusal with when the frame would have been taken from an authenticated client.
const AUTH_REQUIRED: &str = "auth-required:";

type Socket = WebSocket<MaybeTlsStream<std::net::TcpStream>>;

/// A relay's socket, and how many subscriptions have been asked over it. An id is never reused, so a frame
/// left over from a subscription that has ended is recognisably not the current one's.
#[derive(Default)]
struct Connection {
    socket: Option<Socket>,
    asked: u64,
}

/// One relay, at a websocket URL, spoken to as one `account`.
pub struct Relay {
    url: String,
    account: Arc<Account>,
    connection: Mutex<Connection>,
    /// The challenge the relay offered on the socket now open, when it offered one. Replaced whenever a
    /// fresh socket brings a fresh challenge, and meaningless to a relay that sends none.
    challenge: Mutex<Option<String>>,
    /// What this relay says it keeps, once it has been asked.
    information: OnceLock<Option<Information>>,
}

impl Relay {
    pub fn new(url: impl Into<String>, account: Arc<Account>) -> Self {
        Self {
            url: url.into(),
            account,
            connection: Mutex::new(Connection::default()),
            challenge: Mutex::new(None),
            information: OnceLock::new(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Publish `note` and wait for the relay's [`Answer`], reason and all, as it wrote it.
    ///
    /// A refusal is not an error, and nothing on the way out reads the reason — which of them this caller
    /// can act on is the caller's to decide. The error is the socket. `auth-required:` is answered rather
    /// than returned: the client signs the challenge and offers the same note again.
    pub fn publish(&self, note: &Note) -> Result<Answer> {
        self.exchange(|relay, socket, _| {
            let mut answered = false;
            loop {
                send(socket, json!(["EVENT", note]))?;
                let said = relay.answer(socket, &note.id)?;
                if said.accepted || answered || !said.reason.starts_with(AUTH_REQUIRED) {
                    return Ok(said);
                }
                relay.authenticate(socket)?;
                answered = true;
            }
        })
    }

    /// Every note the relay holds matching any of `filters` — `REQ`, collect until `EOSE`, then `CLOSE`.
    /// Notes sent after `EOSE` are not waited for: this asks what is stored, not what is next. Newest first,
    /// the order NIP-01 answers in and the order a walk backwards through `until` reads.
    pub fn fetch(&self, filters: &[Filter]) -> Result<Vec<Note>> {
        self.exchange(|relay, socket, id| {
            let mut request = vec![json!("REQ"), json!(id)];
            for filter in filters {
                request.push(serde_json::to_value(filter)?);
            }
            let request = Value::Array(request);

            let mut answered = false;
            loop {
                send(socket, request.clone())?;
                let refusal = match relay.page(socket, id)? {
                    Ok(page) => return Ok(page),
                    Err(message) => message,
                };
                if answered || !refusal.starts_with(AUTH_REQUIRED) {
                    bail!("the relay refused the subscription: {refusal}");
                }
                relay.authenticate(socket)?;
                answered = true;
            }
        })
    }

    /// The relay's answer about the note `id` it was just offered, read off the `OK` frame naming that note.
    fn answer(&self, socket: &mut Socket, id: &str) -> Result<Answer> {
        loop {
            if let Frame::Ok {
                id: answered,
                accepted,
                message,
            } = self.next(socket)?
            {
                if answered == id {
                    return Ok(Answer {
                        accepted,
                        reason: message,
                    });
                }
            }
        }
    }

    /// One subscription's stored answer — everything up to `EOSE`, then `CLOSE` — or the relay's own reason
    /// for refusing it.
    fn page(&self, socket: &mut Socket, id: &str) -> Result<Result<Vec<Note>, String>> {
        let mut found = Vec::new();
        loop {
            match self.next(socket)? {
                Frame::Event { subscription, note } if subscription == id => found.push(note),
                Frame::Eose { subscription } if subscription == id => {
                    send(socket, json!(["CLOSE", id]))?;
                    return Ok(Ok(found));
                }
                Frame::Closed {
                    subscription,
                    message,
                } if subscription == id => return Ok(Err(message)),
                _ => continue,
            }
        }
    }

    /// Answer the challenge this relay offered, so it knows which account is asking (NIP-42). The note names
    /// the relay by the URL this client dialed and carries the challenge exactly as it arrived. A relay that
    /// refused a frame without ever sending one cannot be satisfied, and saying so beats retrying forever.
    fn authenticate(&self, socket: &mut Socket) -> Result<()> {
        let challenge = self
            .challenge
            .lock()
            .map_err(|_| anyhow!("relay {}: the challenge is poisoned", self.url))?
            .clone()
            .ok_or_else(|| {
                anyhow!(
                    "relay {} asked for authentication and offered no challenge",
                    self.url
                )
            })?;
        let note = Note::write(
            &self.account,
            Kind::RelayAuth,
            seconds(),
            Tags::new()
                .tag("relay", self.url.clone())
                .tag("challenge", challenge),
            String::new(),
        )?;
        send(socket, json!(["AUTH", note]))?;
        let said = self.answer(socket, &note.id)?;
        match said.accepted {
            true => Ok(()),
            false => bail!(
                "relay {} refused the authentication: {}",
                self.url,
                said.reason
            ),
        }
    }

    /// What the relay says about itself (NIP-11): the same URL over plain HTTP, asked with
    /// `Accept: application/nostr+json`. Asked once and kept for as long as this relay lives.
    ///
    /// **`None` is a relay that answered nothing**, which is not the same as one whose document left every
    /// field out: a caller reading silence as a statement would be quoting a relay that never spoke.
    pub fn information(&self) -> Option<Information> {
        self.information.get_or_init(|| self.ask().ok()).clone()
    }

    fn ask(&self) -> Result<Information> {
        let answer = ureq::get(&http(&self.url))
            .timeout(TIMEOUT)
            .set("Accept", "application/nostr+json")
            .call()
            .map_err(|e| anyhow!("asking relay {} what it keeps: {e}", self.url))?
            .into_string()?;
        Ok(serde_json::from_str(&answer)?)
    }

    /// The limits this relay states it holds a client to (NIP-11), or `None` when it answered no document.
    pub fn limits(&self) -> Option<Limitation> {
        Some(self.information()?.limitation)
    }

    /// What this relay says against keeping notes of `kind`, or `None` when it says nothing against them. A
    /// relay that answers no document, or one whose document names no kinds, refuses nothing here — NIP-11
    /// is optional, and a publish is a better question than a guess.
    pub fn refusal(&self, kind: u16) -> Option<String> {
        let information = self.information()?;
        match information.keeps(kind) {
            true => None,
            false => Some(format!(
                "{} keeps kinds {}, not kind {kind}",
                self.url,
                information.kinds()
            )),
        }
    }

    /// Run one exchange over this relay's connection, opening one if it holds none, and give the exchange a
    /// subscription id no earlier one has used.
    ///
    /// A held socket may have gone away with nothing said over it, and nothing finds out until the next
    /// frame. So a failure over an already-open socket is tried once more over a fresh one. Nothing is
    /// published twice: a refused note was never stored.
    fn exchange<T>(&self, run: impl Fn(&Self, &mut Socket, &str) -> Result<T>) -> Result<T> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("relay {}: the connection is poisoned", self.url))?;
        connection.asked += 1;
        let id = format!("celis-{}", connection.asked);

        if let Some(socket) = connection.socket.as_mut() {
            match run(self, socket, &id) {
                Ok(answer) => return Ok(answer),
                Err(_) => connection.socket = None,
            }
        }
        let mut socket = self.open()?;
        let answer = run(self, &mut socket, &id)?;
        connection.socket = Some(socket);
        Ok(answer)
    }

    /// Close the connection to this relay, if one is open. A round says so when it ends, and a host says so
    /// when the app is going away, so the relay is told rather than left to time the socket out.
    pub fn close(&self) {
        let Ok(mut connection) = self.connection.lock() else {
            return;
        };
        if let Some(mut socket) = connection.socket.take() {
            let _ = socket.close(None);
        }
    }

    fn open(&self) -> Result<Socket> {
        let (socket, _) =
            tungstenite::connect(&self.url).map_err(|e| anyhow!("relay {}: {e}", self.url))?;
        match socket.get_ref() {
            MaybeTlsStream::Plain(stream) => stream.set_read_timeout(Some(TIMEOUT))?,
            MaybeTlsStream::Rustls(stream) => stream.get_ref().set_read_timeout(Some(TIMEOUT))?,
            _ => {}
        }
        Ok(socket)
    }

    /// The next frame the relay sends that this crate understands. A challenge is taken as it passes rather
    /// than waited for: it arrives unasked, before whatever this exchange was after, and is only wanted if
    /// the relay later turns something down.
    fn next(&self, socket: &mut Socket) -> Result<Frame> {
        loop {
            let frame = match socket
                .read()
                .map_err(|e| anyhow!("relay {}: {e}", self.url))?
            {
                Message::Text(text) => Frame::parse(&text)?,
                Message::Close(_) => bail!("relay {} hung up", self.url),
                _ => continue,
            };
            if let Frame::Auth { challenge } = frame {
                if let Ok(mut held) = self.challenge.lock() {
                    *held = Some(challenge);
                }
                continue;
            }
            return Ok(frame);
        }
    }
}

/// A relay's websocket URL as the HTTP one its information document is served from — the same address, the
/// scheme a plain request uses. Anything else is left as it is.
fn http(url: &str) -> String {
    match url.split_once("://") {
        Some(("wss", rest)) => format!("https://{rest}"),
        Some(("ws", rest)) => format!("http://{rest}"),
        _ => url.to_string(),
    }
}

/// Unix seconds now — a note's `created_at`, which a relay checks against its own clock.
fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn send(socket: &mut Socket, frame: Value) -> Result<()> {
    socket.send(Message::Text(frame.to_string()))?;
    Ok(())
}

/// One frame a relay sends back (NIP-01). `Other` is anything this client has no use for, including a
/// `NOTICE` — a relay's remarks are not a protocol.
#[derive(Debug, PartialEq)]
enum Frame {
    Event {
        subscription: String,
        note: Note,
    },
    Eose {
        subscription: String,
    },
    Ok {
        id: String,
        accepted: bool,
        message: String,
    },
    Closed {
        subscription: String,
        message: String,
    },
    /// The challenge a relay offers on connect, for a client that wants to be known (NIP-42).
    Auth {
        challenge: String,
    },
    Other,
}

impl Frame {
    fn parse(text: &str) -> Result<Frame> {
        let frame: Vec<Value> = serde_json::from_str(text)
            .map_err(|e| anyhow!("the relay sent something that is not a frame: {e}"))?;
        let name = frame
            .first()
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("the relay sent an unnamed frame"))?;
        let text_at = |i: usize| {
            frame
                .get(i)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        Ok(match name {
            "EVENT" => Frame::Event {
                subscription: text_at(1),
                note: serde_json::from_value(
                    frame
                        .get(2)
                        .cloned()
                        .ok_or_else(|| anyhow!("an EVENT frame with no note"))?,
                )?,
            },
            "EOSE" => Frame::Eose {
                subscription: text_at(1),
            },
            "OK" => Frame::Ok {
                id: text_at(1),
                accepted: frame.get(2).and_then(Value::as_bool).unwrap_or(false),
                message: text_at(3),
            },
            "CLOSED" => Frame::Closed {
                subscription: text_at(1),
                message: text_at(2),
            },
            "AUTH" if frame.get(1).and_then(Value::as_str).is_some() => Frame::Auth {
                challenge: text_at(1),
            },
            _ => Frame::Other,
        })
    }
}

#[cfg(test)]
mod tests;
