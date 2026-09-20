//! **A relay** — where notes go and where they come back from. One websocket, four frames: `EVENT`
//! publishes, `REQ` asks, `EOSE` says the relay has sent everything it holds, `CLOSE` ends the
//! subscription. Nothing here decides anything about a note's meaning; a relay is storage that speaks.
//!
//! A relay holds its connection for as long as it is held itself: every exchange with the same relay
//! travels over one socket, opened the first time it is needed and opened again if it goes away. Who says
//! when to let go is the host (`spec/network.md`).
//!
//! **Authentication is answered, never offered** (NIP-42). A relay that wants to know who is asking sends a
//! challenge on connect and turns a frame down with `auth-required:`; only then does this client sign the
//! challenge and try the frame again.

use crate::mailbox::Mailbox;
use crate::seconds;
use crate::{Account, Answer, Filter, Information, Kind, Limitation, Note, Tags};
use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tungstenite::client::IntoClientRequest;
use tungstenite::http::Uri;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

/// How long one exchange waits on a relay that has stopped answering. A sync holds the binding's write
/// guard for its whole length, so this is what a stalled relay costs the person at the screen — well past
/// any answer a relay owes, and short enough that a dead one is a pause rather than a freeze. It is also
/// how long dialling one may take, and how long a write may wait on a socket that has stopped reading.
const TIMEOUT: Duration = Duration::from_secs(15);

/// How long one whole exchange may take, however much the relay keeps saying. A relay that answers with
/// anything but what was asked — a challenge over and over, a remark, another subscription's notes — would
/// otherwise hold the connection for as long as it liked.
const EXCHANGE: Duration = Duration::from_secs(60);

/// How many pings in a row may go unanswered before the held socket is judged dead and dialled again. A
/// socket a router has forgotten looks healthy from here: reads time out into nothing, which is what a quiet
/// relay looks like too, and only the missing answers tell the two apart.
const LOST: u32 = 3;

/// What a relay prefixes a refusal with when the frame would have been taken from an authenticated client.
const AUTH_REQUIRED: &str = "auth-required:";

/// How long a held subscription waits on a quiet socket before it looks up. Short, because it is also how
/// often the thread asks whether it has been told to stop, and how long a burst of notes is gathered into
/// one ring.
const TICK: Duration = Duration::from_secs(1);

/// How many quiet [`TICK`]s pass before the client says it is still there. The relay answers this without
/// waking, so it costs the host nothing (`spec/network.md`).
const PING_EVERY: u32 = 30;

/// What a client says on a quiet socket, and what a relay answers. Bare words rather than NIP-01 frames,
/// which all start with `[` — so nothing can read one as the other (`spec/network.md`).
const PING: &str = "ping";
const PONG: &str = "pong";

/// Where a redial starts, and the longest it ever waits. The host stops the dialling altogether while there
/// is no network, so this is the rate for a relay that is down while the network is not.
const REDIAL: Duration = Duration::from_secs(1);
const LONGEST: Duration = Duration::from_secs(60);

/// How long a socket must have been held for its breaking to be ordinary rather than a flap. The longest
/// wait, because a socket that lived past that broke for a reason no backoff was going to outlast, and one
/// that did not would ring at the rate it flaps (`spec/network.md`).
const STEADY: Duration = LONGEST;

/// The subscription a held socket asks under. One per relay, so it never collides with an exchange's.
const WATCHING: &str = "celis-watch";

/// How long an id this device published is held against an echo of it. A relay hands a publisher its own
/// note back within a breath of the publish, and past this an id that matches is somebody else's
/// (`spec/network.md`).
const ECHOING: Duration = Duration::from_secs(60);

/// How far ahead of this device's clock a note may be stamped and still be taken as having just landed. The
/// same allowance a relay holds a publisher to, so a device whose clock trails the sender's still hears what
/// it is sent (`spec/network.md`).
pub const SKEW: u64 = 300;

/// How long what a relay said about itself stands before it is asked again. A relay is held for as long as
/// the vault is open, so a document read once would let one failed read stand for the whole session
/// (`spec/network.md`).
const STATED: Duration = Duration::from_secs(3600);

/// How long a relay that answered **no** document stands before it is asked again.
///
/// Far shorter than [`STATED`], because the two are not the same fact. What a relay said is about the relay
/// and keeps for the hour; what it did not say is nothing, and a caller acts on that nothing — a note
/// refused for a kind this relay does not keep is offered again until the document settles it
/// (`spec/network.md`). An hour of remembered silence is an hour of offering it.
const UNANSWERED: Duration = Duration::from_secs(60);

/// How long the document itself waits. Shorter than an exchange: it is one small `GET`, it is asked while
/// the relay's own lock is held, and one that cannot answer it quickly is asked again in a minute anyway.
const ASKING: Duration = Duration::from_secs(5);

type Socket = WebSocket<MaybeTlsStream<std::net::TcpStream>>;

/// One open socket to a relay, and the challenge that relay offered on it.
///
/// A challenge belongs to the socket it arrived on, so a relay reached over two sockets never answers one
/// with the other's.
struct Line {
    url: String,
    socket: Socket,
    challenge: Option<String>,
    /// When the exchange under way must be over, or `None` on a socket held open for what lands.
    deadline: Option<Instant>,
    /// How many of this client's pings the relay has answered.
    pongs: u32,
}

impl Line {
    /// Dial `url`, giving up on a read that goes unanswered for `patience`. Reaching the host, the
    /// handshake and every write are bounded by [`TIMEOUT`].
    fn dial(url: &str, patience: Duration) -> Result<Line> {
        let request = url
            .into_client_request()
            .map_err(|e| anyhow!("relay {url}: {e}"))?;
        let stream = reach(request.uri()).map_err(|e| anyhow!("relay {url}: {e}"))?;
        stream.set_read_timeout(Some(TIMEOUT))?;
        stream.set_write_timeout(Some(TIMEOUT))?;
        stream.set_nodelay(true)?;
        let (socket, _) =
            tungstenite::client_tls(request, stream).map_err(|e| anyhow!("relay {url}: {e}"))?;
        let line = Line {
            url: url.to_string(),
            socket,
            challenge: None,
            deadline: None,
            pongs: 0,
        };
        line.patience(patience)?;
        Ok(line)
    }

    /// Bound the exchange about to run over this socket to `how_long` from now.
    fn within(&mut self, how_long: Duration) {
        self.deadline = Some(Instant::now() + how_long);
    }

    /// How long a read waits on a relay that is saying nothing, from now on.
    fn patience(&self, how_long: Duration) -> Result<()> {
        match self.socket.get_ref() {
            MaybeTlsStream::Plain(stream) => stream.set_read_timeout(Some(how_long))?,
            MaybeTlsStream::Rustls(stream) => stream.get_ref().set_read_timeout(Some(how_long))?,
            _ => {}
        }
        Ok(())
    }

    fn send(&mut self, frame: Value) -> Result<()> {
        self.socket.send(Message::Text(frame.to_string()))?;
        Ok(())
    }

    /// The next frame the relay sends that this crate understands, where saying nothing in time is a relay
    /// that did not answer.
    fn next(&mut self) -> Result<Frame> {
        self.waited()?
            .ok_or_else(|| anyhow!("relay {} said nothing in time", self.url))
    }

    /// The relay's answer about the note `id` it was just offered, read off the `OK` frame naming that note.
    fn answer(&mut self, id: &str) -> Result<Answer> {
        loop {
            if let Frame::Ok {
                id: answered,
                accepted,
                message,
            } = self.next()?
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
    fn page(&mut self, id: &str) -> Result<Result<Vec<Note>, String>> {
        let mut found = Vec::new();
        loop {
            match self.next()? {
                Frame::Event { subscription, note } if subscription == id => found.push(note),
                Frame::Eose { subscription } if subscription == id => {
                    self.send(json!(["CLOSE", id]))?;
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

    /// The next frame the relay sends that this crate understands, or `None` when it said nothing before the
    /// read gave up.
    ///
    /// A challenge is taken as it passes rather than waited for: it arrives unasked, before whatever this
    /// exchange was after, and is only wanted if the relay later turns something down. A `pong` is passed
    /// over — it answers this client's own keepalive and is not a frame.
    fn waited(&mut self) -> Result<Option<Frame>> {
        loop {
            if self.deadline.is_some_and(|it| Instant::now() >= it) {
                bail!(
                    "relay {} kept talking past the exchange's deadline",
                    self.url
                );
            }
            let read = match self.socket.read() {
                Ok(message) => message,
                Err(tungstenite::Error::Io(e))
                    if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    return Ok(None)
                }
                Err(e) => bail!("relay {}: {e}", self.url),
            };
            let frame = match read {
                Message::Text(text) if text == PONG => {
                    self.pongs += 1;
                    continue;
                }
                Message::Text(text) => Frame::parse(&text)?,
                Message::Close(_) => bail!("relay {} hung up", self.url),
                _ => continue,
            };
            if let Frame::Auth { challenge } = frame {
                self.challenge = Some(challenge);
                continue;
            }
            return Ok(Some(frame));
        }
    }

    /// Read up to the `EOSE` of the subscription just asked for, or the relay's own reason for refusing it.
    /// Everything the relay already held is passed over: this subscription exists to say what lands next.
    fn opened(&mut self) -> Result<Result<(), String>> {
        loop {
            match self.next()? {
                Frame::Eose { subscription } if subscription == WATCHING => return Ok(Ok(())),
                Frame::Closed {
                    subscription,
                    message,
                } if subscription == WATCHING => return Ok(Err(message)),
                _ => continue,
            }
        }
    }

    /// Answer the challenge this relay offered, so it knows which account is asking (NIP-42). The note names
    /// the relay by the URL this client dialed and carries the challenge exactly as it arrived. A relay that
    /// refused a frame without ever sending one cannot be satisfied, and saying so beats retrying forever.
    fn authenticate(&mut self, account: &Account) -> Result<()> {
        let challenge = self.challenge.clone().ok_or_else(|| {
            anyhow!(
                "relay {} asked for authentication and offered no challenge",
                self.url
            )
        })?;
        let note = Note::write(
            account,
            Kind::RelayAuth,
            seconds(),
            Tags::new()
                .tag("relay", self.url.clone())
                .tag("challenge", challenge),
            String::new(),
        )?;
        self.send(json!(["AUTH", note]))?;
        let said = self.answer(&note.id)?;
        match said.accepted {
            true => Ok(()),
            false => bail!(
                "relay {} refused the authentication: {}",
                self.url,
                said.reason
            ),
        }
    }
}

/// A relay's exchange line, and how many subscriptions have been asked over it. An id is never reused, so a
/// frame left over from a subscription that has ended is recognisably not the current one's.
#[derive(Default)]
struct Connection {
    line: Option<Line>,
    asked: u64,
}

/// One relay, at a websocket URL, spoken to as one `account`.
pub struct Relay {
    url: String,
    account: Arc<Account>,
    mailbox: Arc<Mailbox>,
    connection: Mutex<Connection>,
    /// What this relay said it keeps and the second it was asked, or `None` before it has been. The inner
    /// `None` is a relay that answered nothing, which is not a relay that stated nothing.
    information: Mutex<Option<(u64, Option<Information>)>>,
    /// The subscription being held open, when one is. It is on a socket of its own, read by a thread of its
    /// own: a frame is delivered once, so an exchange and a subscription sharing a socket would take each
    /// other's (`spec/network.md`).
    watch: Mutex<Option<Watch>>,
    /// What this device published here, which the subscription reads back and must not hold.
    sent: Arc<Sent>,
}

/// A subscription held open, and the flag that ends the thread reading it.
struct Watch {
    filter: Filter,
    stopped: Arc<AtomicBool>,
}

impl Drop for Watch {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
}

impl Watch {
    /// Whether the thread reading this subscription is still there. A `Watch` outlives its thread, and one
    /// left behind names the right filter, so this is what tells a subscription from the memory of one.
    fn live(&self) -> bool {
        !self.stopped.load(Ordering::SeqCst)
    }
}

/// The notes this device published here, so its own copy coming back down the subscription is not read as
/// something landing.
///
/// A relay hands a publisher its own note back on any subscription that matches it, and a send would
/// otherwise hold what it just sent for the landing that follows. **Held by when, so nothing
/// has to say a round began**: a count drops the earliest of a first sync's thousands, and a round
/// boundary drops the last publish of the round before it, which is the one still echoing.
#[derive(Default)]
struct Sent(Mutex<BTreeMap<String, u64>>);

impl Sent {
    /// Remember that `id` went out from here, and forget what went out too long ago to still be echoing.
    fn wrote(&self, id: &str) {
        if let Ok(mut held) = self.0.lock() {
            let now = seconds();
            held.retain(|_, at| now.saturating_sub(*at) <= ECHOING.as_secs());
            held.insert(id.to_string(), now);
        }
    }

    /// Whether `id` is a note this device published, rather than one that has landed.
    fn ours(&self, id: &str) -> bool {
        self.0.lock().is_ok_and(|held| held.contains_key(id))
    }
}

/// Held by the thread that follows a subscription, and dropped when that thread is gone — however it went.
/// `Drop` runs while a panic unwinds, so a thread that died says so as surely as one that was told to stop.
struct Gone(Arc<AtomicBool>);

impl Drop for Gone {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl Relay {
    pub fn new(url: impl Into<String>, account: Arc<Account>, mailbox: Arc<Mailbox>) -> Self {
        Self {
            url: url.into(),
            account,
            mailbox,
            connection: Mutex::new(Connection::default()),
            information: Mutex::new(None),
            watch: Mutex::new(None),
            sent: Arc::new(Sent::default()),
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
        // Before the send, not after: the relay may hand this note back on the held subscription while the
        // answer to the publish is still in flight.
        self.sent.wrote(&note.id);
        self.exchange(|account, line, _| {
            let mut answered = false;
            loop {
                line.send(json!(["EVENT", note]))?;
                let said = line.answer(&note.id)?;
                if said.accepted || answered || !said.reason.starts_with(AUTH_REQUIRED) {
                    return Ok(said);
                }
                line.authenticate(account)?;
                answered = true;
            }
        })
    }

    /// Every note the relay holds matching any of `filters` — `REQ`, collect until `EOSE`, then `CLOSE`.
    /// Notes sent after `EOSE` are not waited for: this asks what is stored, not what is next. Newest first,
    /// the order NIP-01 answers in and the order a walk backwards through `until` reads.
    pub fn fetch(&self, filters: &[Filter]) -> Result<Vec<Note>> {
        self.exchange(|account, line, id| {
            let request = request(id, filters)?;
            let mut answered = false;
            loop {
                line.send(request.clone())?;
                let refusal = match line.page(id)? {
                    Ok(page) => return Ok(page),
                    Err(message) => message,
                };
                if answered || !refusal.starts_with(AUTH_REQUIRED) {
                    bail!("the relay refused the subscription: {refusal}");
                }
                line.authenticate(account)?;
                answered = true;
            }
        })
    }

    /// What the relay says about itself (NIP-11): the same URL over plain HTTP, asked with
    /// `Accept: application/nostr+json`. An answer is kept for [`STATED`], a silence for [`UNANSWERED`], and
    /// then each is asked again.
    ///
    /// **`None` is a relay that answered nothing**, which is not the same as one whose document left every
    /// field out: a caller reading silence as a statement would be quoting a relay that never spoke. It is
    /// also not worth keeping as long — see [`UNANSWERED`].
    pub fn information(&self) -> Option<Information> {
        let mut held = self.information.lock().ok()?;
        let now = seconds();
        let stale = match held.as_ref() {
            None => true,
            Some((at, stated)) => {
                let window = match stated {
                    Some(_) => STATED,
                    None => UNANSWERED,
                };
                now.saturating_sub(*at) >= window.as_secs()
            }
        };
        if stale {
            *held = Some((now, self.ask().ok()));
        }
        held.as_ref().and_then(|(_, stated)| stated.clone())
    }

    fn ask(&self) -> Result<Information> {
        let answer = ureq::get(&http(&self.url))
            .timeout(ASKING)
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
    fn exchange<T>(&self, run: impl Fn(&Account, &mut Line, &str) -> Result<T>) -> Result<T> {
        // A connection holds nothing a panic can leave half-done, so one is recovered rather than refused.
        let mut connection = self
            .connection
            .lock()
            .unwrap_or_else(|held| held.into_inner());
        connection.asked += 1;
        let id = format!("celis-{}", connection.asked);

        if let Some(line) = connection.line.as_mut() {
            line.within(EXCHANGE);
            match run(&self.account, line, &id) {
                Ok(answer) => return Ok(answer),
                Err(_) => connection.line = None,
            }
        }
        let mut line = Line::dial(&self.url, TIMEOUT)?;
        line.within(EXCHANGE);
        let answer = run(&self.account, &mut line, &id)?;
        connection.line = Some(line);
        Ok(answer)
    }

    /// Hold `filter` open on a socket of its own, so the relay hands a matching note over as it lands rather
    /// than waiting to be asked. A note that lands is left in the mailbox under `stream`, unopened, for the
    /// turn that lands it (`spec/network.md`).
    ///
    /// Asking for the filter already being watched does nothing, so a round may say this every time it
    /// reaches a relay. A different filter replaces the one held, and so does one whose thread is gone —
    /// which is what brings a subscription back rather than leaving the vault on its backstop.
    pub fn watch(&self, filter: &Filter, stream: &str) {
        let Ok(mut held) = self.watch.lock() else {
            return;
        };
        if held
            .as_ref()
            .is_some_and(|it| &it.filter == filter && it.live())
        {
            return;
        }
        let stopped = Arc::new(AtomicBool::new(false));
        *held = Some(Watch {
            filter: filter.clone(),
            stopped: stopped.clone(),
        });
        let (url, account, mailbox, sent, filter, stream) = (
            self.url.clone(),
            self.account.clone(),
            self.mailbox.clone(),
            self.sent.clone(),
            filter.clone(),
            stream.to_string(),
        );
        std::thread::spawn(move || {
            let _gone = Gone(stopped.clone());
            follow(&url, &account, &mailbox, &sent, &filter, &stream, &stopped)
        });
    }

    /// Let go of both sockets: the exchange line, and the subscription being held open. The host says when
    /// — the app going away, or the network going away (`spec/network.md`). Whatever was being watched is
    /// asked for again by the next round that reaches this relay.
    pub fn close(&self) {
        if let Ok(mut watch) = self.watch.lock() {
            watch.take();
        }
        let Ok(mut connection) = self.connection.lock() else {
            return;
        };
        if let Some(mut line) = connection.line.take() {
            let _ = line.socket.close(None);
        }
    }
}

/// A TCP connection to the host `uri` names, tried for [`TIMEOUT`] per address it resolves to.
fn reach(uri: &Uri) -> Result<TcpStream> {
    let host = uri.host().ok_or_else(|| anyhow!("no host in {uri}"))?;
    let port = uri.port_u16().unwrap_or(match uri.scheme_str() {
        Some("wss") => 443,
        _ => 80,
    });
    let mut refused = None;
    for address in (host, port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&address, TIMEOUT) {
            Ok(stream) => return Ok(stream),
            Err(why) => refused = Some(why),
        }
    }
    Err(match refused {
        Some(why) => anyhow!("{why}"),
        None => anyhow!("{host} resolves to no address"),
    })
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

/// Hold one subscription open until `stopped`, leaving each note that lands on it in `mailbox`, and dialling
/// again whenever the socket breaks. Every connect rings: a note may have landed while there was no socket
/// to hear it on, and a ring is one stream's walk under its floor (`spec/runtime.md`).
///
/// **The backoff is reset by how long the socket was held, not by whether the relay spoke.** One that
/// lived longer than the longest wait broke for an ordinary reason and is worth dialling at once. One that
/// died sooner is a flap, and a flap that reset the backoff would ring at the rate it flaps.
fn follow(
    url: &str,
    account: &Account,
    mailbox: &Mailbox,
    sent: &Sent,
    filter: &Filter,
    stream: &str,
    stopped: &AtomicBool,
) {
    let mut waiting = REDIAL;
    while !stopped.load(Ordering::SeqCst) {
        match subscribe(url, account, filter) {
            Ok(mut line) => {
                mailbox.ring(stream);
                let opened = Instant::now();
                read(&mut line, mailbox, sent, stream, stopped);
                if opened.elapsed() >= STEADY {
                    waiting = REDIAL;
                }
            }
            Err(why) => {
                tracing::debug!(target: "client", %url, %why, "the subscription did not open")
            }
        }
        rest(waiting, stopped);
        waiting = (waiting * 2).min(LONGEST);
    }
}

/// A socket of its own, authenticated if the relay asks, holding `filter` open from this moment on.
///
/// **`since` is settled here, not by the caller.** Nothing the relay already holds is wanted — the walk
/// reads that, and this hands over what lands from now — and a filter that carried the second it was made
/// would differ on every round, so the subscription would be torn down and reopened each time.
///
/// Opening waits like an exchange; the held socket does not. Everything up to `EOSE` is a relay answering,
/// and everything after is silence until something lands.
fn subscribe(url: &str, account: &Account, filter: &Filter) -> Result<Line> {
    let from = Filter {
        since: Some(seconds().saturating_sub(SKEW)),
        ..filter.clone()
    };
    let mut line = Line::dial(url, TIMEOUT)?;
    line.within(EXCHANGE);
    let asked = request(WATCHING, std::slice::from_ref(&from))?;
    let mut answered = false;
    loop {
        line.send(asked.clone())?;
        match line.opened()? {
            Ok(()) => {
                line.deadline = None;
                line.patience(TICK)?;
                return Ok(line);
            }
            Err(refusal) if answered || !refusal.starts_with(AUTH_REQUIRED) => {
                bail!("relay {url} refused the subscription: {refusal}")
            }
            Err(_) => {
                line.authenticate(account)?;
                answered = true;
            }
        }
    }
}

/// Read the held subscription until the socket breaks or `stopped` says to let go.
///
/// Each note that lands is handed to the mailbox as it comes, unopened; [`PING_EVERY`] quiet ticks in a
/// row is where the client says it is still there.
///
/// **A note this device published is not one landing.** The relay hands a publisher its own note back, and
/// holding it would land what the send just sent, for as long as the vault is open. Another device on this
/// account is still held: it is the id that says whose note this is, not the author.
///
/// Returns when the socket breaks, the relay closes the subscription, [`LOST`] pings go unanswered, or the
/// thread is told to let go. How long that took is what the caller reads a flap from, so nothing has to be
/// answered here.
fn read(line: &mut Line, mailbox: &Mailbox, sent: &Sent, stream: &str, stopped: &AtomicBool) {
    let (mut quiet, mut pinged) = (0, 0u32);
    while !stopped.load(Ordering::SeqCst) {
        let Ok(heard) = line.waited() else {
            return;
        };
        match heard {
            Some(Frame::Event { subscription, note }) if subscription == WATCHING => {
                if !sent.ours(&note.id) {
                    mailbox.heard(stream, note);
                }
            }
            Some(Frame::Closed {
                subscription,
                message,
            }) if subscription == WATCHING => {
                tracing::debug!(target: "client", url = %line.url, %message, "the relay closed the subscription");
                return;
            }
            Some(_) => continue,
            None => {
                quiet += 1;
                if quiet >= PING_EVERY {
                    quiet = 0;
                    if pinged.saturating_sub(line.pongs) >= LOST {
                        tracing::debug!(target: "client", url = %line.url, "the relay stopped answering");
                        return;
                    }
                    if line.socket.send(Message::Text(PING.into())).is_err() {
                        return;
                    }
                    pinged += 1;
                }
            }
        }
    }
}

/// Wait `how_long`, looking up often enough that being told to stop is not waited out.
fn rest(how_long: Duration, stopped: &AtomicBool) {
    let mut left = how_long;
    while left > Duration::ZERO && !stopped.load(Ordering::SeqCst) {
        let slice = left.min(TICK);
        std::thread::sleep(slice);
        left -= slice;
    }
}

/// One `REQ` frame asking `filters` under the subscription `id`.
fn request(id: &str, filters: &[Filter]) -> Result<Value> {
    let mut frame = vec![json!("REQ"), json!(id)];
    for filter in filters {
        frame.push(serde_json::to_value(filter)?);
    }
    Ok(Value::Array(frame))
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
