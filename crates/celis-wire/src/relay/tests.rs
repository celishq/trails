use super::*;
use crate::mailbox::Mailbox;
use crate::Account;

use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn note() -> Note {
    entry("ciphertext")
}

/// The account a test client speaks as.
fn speaking() -> Arc<Account> {
    Arc::new(Account::new(&[3u8; 32]).unwrap())
}

fn entry(content: &str) -> Note {
    Note::write(
        &Account::new(&[3u8; 32]).unwrap(),
        35_000u16,
        1_700_000_000,
        vec![vec!["d".into(), "entry-1".into()]],
        content,
    )
    .unwrap()
}

/// A relay on a local port, for what is about the socket rather than about a note. It answers `OK` to
/// every `EVENT` and an empty `EOSE` to every `REQ`, counts the connections it accepted, and hangs up once
/// it has answered `answers` of them.
struct LocalRelay {
    url: String,
    accepted: Arc<AtomicUsize>,
}

impl LocalRelay {
    fn new(answers: usize) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let accepted = Arc::new(AtomicUsize::new(0));
        let counted = accepted.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                counted.fetch_add(1, Ordering::SeqCst);
                let Ok(mut socket) = tungstenite::accept(stream) else {
                    continue;
                };
                let mut answered = 0;
                while let Ok(Message::Text(text)) = socket.read() {
                    let frame: Vec<Value> = serde_json::from_str(&text).unwrap();
                    let answer = match frame[0].as_str() {
                        Some("EVENT") => json!(["OK", frame[1]["id"], true, ""]),
                        Some("REQ") => json!(["EOSE", frame[1], 0]),
                        _ => continue,
                    };
                    if socket.send(Message::Text(answer.to_string())).is_err() {
                        break;
                    }
                    answered += 1;
                    if answered == answers {
                        break;
                    }
                }
            }
        });
        Self { url, accepted }
    }

    fn accepted(&self) -> usize {
        self.accepted.load(Ordering::SeqCst)
    }
}

/// A relay that answers a `REQ` with `EOSE` and then pushes one note on it, the way a held subscription
/// receives what lands after it opened.
struct PushingRelay {
    url: String,
}

impl PushingRelay {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let Ok(mut socket) = tungstenite::accept(stream) else {
                    continue;
                };
                std::thread::spawn(move || {
                    while let Ok(Message::Text(text)) = socket.read() {
                        let Ok(frame) = serde_json::from_str::<Vec<Value>>(&text) else {
                            continue;
                        };
                        if frame[0].as_str() != Some("REQ") {
                            continue;
                        }
                        let id = &frame[1];
                        for answer in [json!(["EOSE", id]), json!(["EVENT", id, note()])] {
                            if socket.send(Message::Text(answer.to_string())).is_err() {
                                return;
                            }
                        }
                    }
                });
            }
        });
        Self { url }
    }
}

/// A relay that answers every frame with remarks, one after another, for as long as the socket stands.
struct TalkativeRelay {
    url: String,
}

impl TalkativeRelay {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let Ok(mut socket) = tungstenite::accept(stream) else {
                    continue;
                };
                std::thread::spawn(move || {
                    if socket.read().is_err() {
                        return;
                    }
                    loop {
                        let remark = json!(["NOTICE", "still here"]);
                        if socket.send(Message::Text(remark.to_string())).is_err() {
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(5));
                    }
                });
            }
        });
        Self { url }
    }
}

/// A relay that opens every subscription and closes it again at once, counting the connections it took.
struct ClosingRelay {
    url: String,
    accepted: Arc<AtomicUsize>,
}

impl ClosingRelay {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let accepted = Arc::new(AtomicUsize::new(0));
        let counted = accepted.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                counted.fetch_add(1, Ordering::SeqCst);
                let Ok(mut socket) = tungstenite::accept(stream) else {
                    continue;
                };
                std::thread::spawn(move || {
                    while let Ok(Message::Text(text)) = socket.read() {
                        let Ok(frame) = serde_json::from_str::<Vec<Value>>(&text) else {
                            continue;
                        };
                        if frame[0].as_str() != Some("REQ") {
                            continue;
                        }
                        let id = &frame[1];
                        for answer in [
                            json!(["EOSE", id]),
                            json!(["CLOSED", id, "restricted: not any more"]),
                        ] {
                            if socket.send(Message::Text(answer.to_string())).is_err() {
                                return;
                            }
                        }
                    }
                });
            }
        });
        Self { url, accepted }
    }
}

/// What a ring in these tests names. A stream is named by the sync layer; from here it is a label the
/// relay carries through to the ring.
const STREAM: &str = "abc/records";

#[test]
/// **An exchange ends at its deadline however much the relay keeps saying.** A relay answering with
/// anything but what was asked would otherwise hold the connection, and every exchange queued behind it,
/// for as long as it liked.
fn an_exchange_over_a_relay_that_keeps_talking_ends_at_its_deadline() {
    let served = TalkativeRelay::new();
    let mut line = Line::dial(&served.url, TIMEOUT).unwrap();
    line.within(Duration::from_millis(300));
    line.send(json!(["EVENT", note()])).unwrap();

    let began = std::time::Instant::now();
    let why = line.answer(&note().id).unwrap_err().to_string();

    assert!(why.contains("deadline"), "{why}");
    assert!(
        began.elapsed() < Duration::from_secs(5),
        "{:?}",
        began.elapsed()
    );
}

#[test]
/// **A subscription the relay closed is opened again.** A `CLOSED` on the held socket is the relay letting
/// go — an expired session, say — and a thread that read past it would hold a dead subscription for the
/// whole session, hearing nothing and saying so to nobody.
fn a_subscription_the_relay_closed_is_opened_again() {
    let served = ClosingRelay::new();
    let relay = Relay::new(&served.url, speaking(), Arc::new(Mailbox::default()));

    relay.watch(&Filter::of(Kind::Wrap), STREAM);

    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while served.accepted.load(Ordering::SeqCst) < 2 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        served.accepted.load(Ordering::SeqCst) >= 2,
        "the closed subscription was dialled again"
    );
    relay.close();
}

#[test]
/// **A relay that stops answering pings is let go of.** A socket a router has forgotten reads like a quiet
/// relay, and only the missing answers say it is not one.
fn a_relay_that_stops_answering_pings_is_let_go_of() {
    let served = LocalRelay::new(usize::MAX);
    let mut line = Line::dial(&served.url, Duration::from_millis(2)).unwrap();
    let stopped = AtomicBool::new(false);

    let began = std::time::Instant::now();
    read(
        &mut line,
        &Mailbox::default(),
        &Sent::default(),
        STREAM,
        &stopped,
    );

    assert!(
        began.elapsed() < Duration::from_secs(10),
        "three pings unanswered at two milliseconds a tick: {:?}",
        began.elapsed()
    );
}

#[test]
/// A panic inside one exchange does not cost the relay every exchange after it.
fn a_connection_a_panic_left_behind_is_used_again() {
    let served = LocalRelay::new(usize::MAX);
    let relay = Relay::new(&served.url, speaking(), Arc::new(Mailbox::default()));
    relay.publish(&note()).unwrap();

    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        relay.exchange(|_, _, _| -> Result<()> { panic!("inside the exchange") })
    }));
    assert!(panicked.is_err());

    assert!(relay.publish(&note()).unwrap().accepted);
}

#[test]
/// A note landing on a held subscription is left in the mailbox under the stream it landed on, unopened,
/// and the connect before it rang with none — the walk is owed, the note is not read twice
/// (`spec/network.md`).
fn a_note_landing_on_a_held_subscription_is_held_under_its_stream() {
    let served = PushingRelay::new();
    let mailbox = Arc::new(Mailbox::default());
    let relay = Relay::new(&served.url, speaking(), mailbox.clone());

    relay.watch(&Filter::of(Kind::Wrap), STREAM);

    let held = wait_for_note(&mailbox);
    assert!(held.walk, "the connect rang, so the walk is owed");
    assert_eq!(
        held.notes
            .iter()
            .map(|it| it.id.clone())
            .collect::<Vec<_>>(),
        vec![note().id],
        "and the note that landed after it is held as it came"
    );
    relay.close();
}

/// Wait until `mailbox` holds a note on [`STREAM`], and take what it holds.
fn wait_for_note(mailbox: &Mailbox) -> crate::mailbox::Held {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let mut taken = mailbox.take(&[STREAM.to_string()]);
        if let Some(held) = taken.remove(STREAM) {
            if !held.notes.is_empty() {
                return held;
            }
            // Only the connect so far: put the walk back and keep waiting for the note.
            mailbox.ring(STREAM);
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no note reached the mailbox"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Wait until `served` has taken `many` connections, or fail saying how far it got.
fn accepted(served: &LocalRelay, many: usize) {
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while served.accepted() < many {
        assert!(
            std::time::Instant::now() < deadline,
            "took {} connections, waiting for {many}",
            served.accepted()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
/// One watch per relay: a round says what to watch every time it reaches one, and saying the same thing
/// again must not open a second socket.
fn watching_the_same_filter_again_holds_the_one_subscription() {
    let served = LocalRelay::new(usize::MAX);
    let relay = Relay::new(&served.url, speaking(), Arc::new(Mailbox::default()));

    for _ in 0..5 {
        relay.watch(&Filter::of(Kind::Wrap), STREAM);
    }
    accepted(&served, 1);

    assert_eq!(
        served.accepted(),
        1,
        "one socket, however often it is asked for"
    );
    relay.close();
}

#[test]
/// A thread that is gone leaves a watch behind naming the right filter, so nothing else would notice it had
/// died. The thread flags itself on the way out and the next round dials again — otherwise the vault falls
/// back to its backstop for the rest of the session, silently.
fn a_watch_whose_thread_is_gone_is_asked_for_again() {
    let served = LocalRelay::new(usize::MAX);
    let relay = Relay::new(&served.url, speaking(), Arc::new(Mailbox::default()));
    let filter = Filter::of(Kind::Wrap);

    relay.watch(&filter, STREAM);
    accepted(&served, 1);

    // What the guard the thread holds does when that thread is gone, however it went.
    relay
        .watch
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .stopped
        .store(true, Ordering::SeqCst);
    relay.watch(&filter, STREAM);

    accepted(&served, 2);
    relay.close();
}

/// A relay that turns every note down with one reason of its own — for what a client is handed back.
struct RefusingRelay {
    url: String,
}

impl RefusingRelay {
    fn saying(reason: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let reason = reason.to_string();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let Ok(mut socket) = tungstenite::accept(stream) else {
                    continue;
                };
                while let Ok(Message::Text(text)) = socket.read() {
                    let frame: Vec<Value> = serde_json::from_str(&text).unwrap();
                    let answer = json!(["OK", frame[1]["id"], false, reason]);
                    if socket.send(Message::Text(answer.to_string())).is_err() {
                        break;
                    }
                }
            }
        });
        Self { url }
    }
}

#[test]
fn a_relays_reason_reaches_the_caller_unparsed() {
    let said = "error: could not save: invalid: is what it said about the last one";
    let relay = RefusingRelay::saying(said);
    let client = Relay::new(&relay.url, speaking(), Arc::new(Mailbox::default()));

    let answer = client.publish(&entry("an entry")).unwrap();

    assert!(!answer.accepted, "a refusal is an answer, not an error");
    assert_eq!(answer.reason, said, "word for word");
}

/// A relay that offers a challenge and turns down anything from a client that has not answered it.
struct GuardedRelay {
    url: String,
    /// Every auth note it was given, so a test can read what the client signed.
    answered: Arc<Mutex<Vec<Note>>>,
}

impl GuardedRelay {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let answered = Arc::new(Mutex::new(Vec::new()));
        let kept = answered.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let Ok(mut socket) = tungstenite::accept(stream) else {
                    continue;
                };
                let challenge = "the-challenge";
                let _ = socket.send(Message::Text(json!(["AUTH", challenge]).to_string()));
                let mut known = false;
                while let Ok(Message::Text(text)) = socket.read() {
                    let frame: Vec<Value> = serde_json::from_str(&text).unwrap();
                    let answer = match frame[0].as_str() {
                        Some("AUTH") => {
                            let note: Note = serde_json::from_value(frame[1].clone()).unwrap();
                            let id = note.id.clone();
                            kept.lock().unwrap().push(note);
                            known = true;
                            json!(["OK", id, true, ""])
                        }
                        Some("EVENT") if !known => {
                            json!(["OK", frame[1]["id"], false, "auth-required: who are you"])
                        }
                        Some("EVENT") => json!(["OK", frame[1]["id"], true, ""]),
                        Some("REQ") if !known => {
                            json!(["CLOSED", frame[1], "auth-required: who are you"])
                        }
                        Some("REQ") => json!(["EOSE", frame[1], 0]),
                        _ => continue,
                    };
                    if socket.send(Message::Text(answer.to_string())).is_err() {
                        break;
                    }
                }
            }
        });
        Self { url, answered }
    }
}

#[test]
/// The whole of NIP-42 from this side, without the caller knowing any of it happened.
fn a_relay_that_asks_who_is_publishing_is_answered_and_the_note_goes_again() {
    let guarded = GuardedRelay::new();
    let client = Relay::new(&guarded.url, speaking(), Arc::new(Mailbox::default()));

    client.publish(&entry("a first entry")).unwrap();

    let answered = guarded.answered.lock().unwrap();
    assert_eq!(answered.len(), 1, "answered once, not once per note");
    assert_eq!(answered[0].kind, Kind::RelayAuth);
    assert_eq!(answered[0].tag("challenge"), Some("the-challenge"));
    assert_eq!(answered[0].tag("relay"), Some(guarded.url.as_str()));
    assert_eq!(answered[0].pubkey, speaking().pubkey());
    assert!(answered[0].verifies());
}

#[test]
fn a_relay_that_asks_who_is_subscribing_is_answered_and_the_request_goes_again() {
    let guarded = GuardedRelay::new();
    let client = Relay::new(&guarded.url, speaking(), Arc::new(Mailbox::default()));

    assert!(
        client.fetch(&[Filter::default()]).is_ok(),
        "answered once it knows"
    );
    assert_eq!(guarded.answered.lock().unwrap().len(), 1);
}

#[test]
fn a_relay_that_asks_nothing_is_told_nothing() {
    let relay = LocalRelay::new(usize::MAX);
    let client = Relay::new(&relay.url, speaking(), Arc::new(Mailbox::default()));
    client.publish(&entry("an entry")).unwrap();
    assert_eq!(
        relay.accepted(),
        1,
        "one socket, and no round trip for AUTH"
    );
}

/// A relay's NIP-11 document on a local port, counting how many times it was asked for.
struct LocalDocument {
    url: String,
    asked: Arc<AtomicUsize>,
}

impl LocalDocument {
    fn serving(document: &'static str) -> Self {
        Self::silent_then(document, 0)
    }

    /// The same, hanging up on the first `silent` requests before it starts answering.
    fn silent_then(document: &'static str, silent: usize) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let asked = Arc::new(AtomicUsize::new(0));
        let counted = asked.clone();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            for mut stream in listener.incoming().flatten() {
                let seen = counted.fetch_add(1, Ordering::SeqCst);
                let _ = stream.read(&mut [0u8; 1024]);
                if seen < silent {
                    continue;
                }
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/nostr+json\r\n\
                         Content-Length: {}\r\nConnection: close\r\n\r\n{document}",
                        document.len()
                    )
                    .as_bytes(),
                );
            }
        });
        Self { url, asked }
    }

    fn asked(&self) -> usize {
        self.asked.load(Ordering::SeqCst)
    }
}

#[test]
fn a_relays_document_is_asked_for_once_however_often_it_is_read() {
    let served = LocalDocument::serving(r#"{"name":"a mirror","retention":[{"kinds":[1]}]}"#);
    let relay = Relay::new(&served.url, speaking(), Arc::new(Mailbox::default()));

    for _ in 0..10 {
        assert!(
            relay.refusal(30_023).is_some(),
            "it named kind 1 and an article is not kind 1"
        );
    }
    assert!(relay.refusal(1).is_none());

    assert_eq!(
        served.asked(),
        1,
        "a document describes the relay, not the note — one request, not one per publish"
    );
}

#[test]
fn a_relay_that_answers_no_document_refuses_nothing() {
    let relay = Relay::new("ws://127.0.0.1:1", speaking(), Arc::new(Mailbox::default()));
    assert!(relay.refusal(30_023).is_none(), "silence is not a refusal");
    assert!(
        relay.information().is_none(),
        "and it is remembered as having said nothing, not as having said the defaults"
    );
    assert!(relay.limits().is_none());
}

/// Age what `relay` holds of its document by `by` seconds, so a test need not wait the window out.
fn age(relay: &Relay, by: u64) {
    let mut held = relay.information.lock().unwrap();
    if let Some((at, _)) = held.as_mut() {
        *at = at.saturating_sub(by);
    }
}

#[test]
/// A relay is held for as long as the vault is open, so a document read once and never again would let one
/// failed read decide the fitness check for the whole session — the check that keeps this account's stream
/// off a relay anybody may read (`spec/network.md`).
///
/// **A silence goes stale far sooner than a document does.** What a relay said about itself is a fact about
/// the relay and stands for the hour; what it did not say is nothing, and remembering nothing for an hour is
/// an hour of acting on it — a note refused for a kind is offered again until the document settles it.
fn a_silence_is_asked_again_long_before_a_document_would_be() {
    let served = LocalDocument::silent_then(
        r#"{"limitation":{"auth_required":true,"max_limit":500}}"#,
        1,
    );
    let relay = Relay::new(&served.url, speaking(), Arc::new(Mailbox::default()));

    assert!(
        relay.limits().is_none(),
        "the first ask was answered with nothing"
    );
    assert!(
        relay.limits().is_none(),
        "and the silence stands until it goes stale, so a dead relay is not asked on every read"
    );
    assert_eq!(served.asked(), 1);

    age(&relay, UNANSWERED.as_secs());

    assert!(
        relay.limits().is_some(),
        "asked again a minute on, and this time it answered"
    );
    assert_eq!(served.asked(), 2);

    age(&relay, UNANSWERED.as_secs());
    assert!(relay.limits().is_some());
    assert_eq!(
        served.asked(),
        2,
        "and what it did answer stands for the hour, so the document is not read every minute"
    );
}

#[test]
fn a_run_of_publishes_travels_over_one_socket() {
    let relay = LocalRelay::new(usize::MAX);
    let client = Relay::new(&relay.url, speaking(), Arc::new(Mailbox::default()));
    for i in 0..500 {
        client.publish(&entry(&format!("entry {i}"))).unwrap();
    }
    assert_eq!(
        relay.accepted(),
        1,
        "a relay is connected to once, not once per note"
    );
}

#[test]
fn a_socket_the_relay_dropped_is_opened_again_and_the_run_finishes() {
    let relay = LocalRelay::new(3);
    let client = Relay::new(&relay.url, speaking(), Arc::new(Mailbox::default()));
    for i in 0..6 {
        client
            .publish(&entry(&format!("entry {i}")))
            .expect("a relay that hung up is connected to again, not reported as a failure");
    }
    assert_eq!(relay.accepted(), 2, "one reconnection, not six");
}

#[test]
fn every_frame_the_client_acts_on_parses() {
    let note = note();
    assert_eq!(
        Frame::parse(&json!(["EVENT", "celis", note]).to_string()).unwrap(),
        Frame::Event {
            subscription: "celis".into(),
            note
        }
    );
    assert_eq!(
        Frame::parse(r#"["EOSE","celis"]"#).unwrap(),
        Frame::Eose {
            subscription: "celis".into()
        }
    );
    assert_eq!(
        Frame::parse(r#"["OK","abc",true,""]"#).unwrap(),
        Frame::Ok {
            id: "abc".into(),
            accepted: true,
            message: String::new()
        }
    );
    assert_eq!(
        Frame::parse(r#"["OK","abc",false,"invalid: bad signature"]"#).unwrap(),
        Frame::Ok {
            id: "abc".into(),
            accepted: false,
            message: "invalid: bad signature".into()
        }
    );
    assert_eq!(
        Frame::parse(r#"["CLOSED","celis","blocked: too many subscriptions"]"#).unwrap(),
        Frame::Closed {
            subscription: "celis".into(),
            message: "blocked: too many subscriptions".into()
        }
    );
}

#[test]
fn a_challenge_is_read_as_one() {
    assert_eq!(
        Frame::parse(r#"["AUTH","a-challenge"]"#).unwrap(),
        Frame::Auth {
            challenge: "a-challenge".to_string()
        }
    );
}

#[test]
fn a_frame_with_nothing_to_act_on_is_not_an_error() {
    assert_eq!(
        Frame::parse(r#"["NOTICE","the relay would like you to know something"]"#).unwrap(),
        Frame::Other
    );
    assert_eq!(
        Frame::parse(r#"["AUTH"]"#).unwrap(),
        Frame::Other,
        "a challenge with no challenge in it"
    );
}

fn information(document: &str) -> Information {
    serde_json::from_str(document).unwrap()
}

#[test]
fn a_relay_that_names_the_kinds_it_keeps_answers_for_each_one() {
    let relay = information(
        r#"{"name":"a mirror","retention":[{"kinds":[1,5,[10000,19999]],"time":null}]}"#,
    );
    assert_eq!(relay.name, "a mirror");
    assert!(relay.keeps(1));
    assert!(relay.keeps(5));
    assert!(relay.keeps(10_002), "inside the range it named");
    assert!(
        !relay.keeps(30_023),
        "an article, which it never said it keeps"
    );
    assert_eq!(relay.kinds(), "1, 5, [10000,19999]");
}

#[test]
fn a_relay_states_what_it_requires_and_how_much_it_answers_at_once() {
    let stated = information(
        r#"{"name":"ours","limitation":{"auth_required":true,"max_limit":5000,"max_filters":10}}"#,
    );
    assert!(stated.limitation.auth_required);
    assert_eq!(stated.limitation.max_limit, Some(5000));
}

#[test]
fn a_relay_that_states_no_limits_requires_nothing_and_says_nothing() {
    for document in [r#"{"name":"quiet"}"#, r#"{"limitation":{}}"#] {
        let quiet = information(document);
        assert!(!quiet.limitation.auth_required, "{document}");
        assert_eq!(quiet.limitation.max_limit, None, "{document}");
    }
    assert!(!Information::default().limitation.auth_required);
    assert_eq!(Information::default().limitation.max_limit, None);
}

#[test]
fn a_relay_that_says_nothing_about_kinds_refuses_none() {
    assert!(information(r#"{"name":"quiet"}"#).keeps(30_023));
    assert!(information(r#"{"retention":[{"time":3600}]}"#).keeps(30_023));
    assert!(Information::default().keeps(30_023));
}

#[test]
fn a_relays_document_is_asked_for_over_http_at_the_same_address() {
    assert_eq!(
        http("wss://relay.example.com/abc/relay"),
        "https://relay.example.com/abc/relay"
    );
    assert_eq!(
        http("ws://127.0.0.1:8787/abc/relay"),
        "http://127.0.0.1:8787/abc/relay"
    );
    assert_eq!(
        http("https://relay.example.com"),
        "https://relay.example.com"
    );
}

#[test]
fn a_frame_that_is_not_one_is_refused() {
    assert!(Frame::parse("not json").is_err());
    assert!(Frame::parse(r#"{"kind":1}"#).is_err(), "not an array");
    assert!(Frame::parse("[42]").is_err(), "unnamed");
    assert!(Frame::parse(r#"["EVENT","celis"]"#).is_err(), "no note");
    assert!(
        Frame::parse(r#"["EVENT","celis",{"id":"nope"}]"#).is_err(),
        "not a note"
    );
}

#[test]
/// **The echo memory is bounded by when, not by how many.** A first sync publishes thousands at once, and
/// a memory that dropped the earliest of them would ring for this device's own notes exactly where it
/// matters most (`spec/network.md`).
fn a_publish_of_thousands_holds_every_one_of_them_against_its_echo() {
    let relay = Relay::new(
        "ws://nowhere.example",
        speaking(),
        Arc::new(Mailbox::default()),
    );
    let published: Vec<String> = (0..2000)
        .map(|n| entry(&format!("ciphertext {n}")).id)
        .collect();

    for id in &published {
        relay.sent.wrote(id);
    }

    assert!(
        published.iter().all(|id| relay.sent.ours(id)),
        "an id published in this breath was forgotten while its echo was still in flight"
    );
}

#[test]
/// **A note this device published is not one landing.** A relay hands a publisher its own note back on the
/// subscription it holds, and holding it would land what the send just sent, for as long as the vault is
/// open.
fn a_note_this_device_published_is_neither_held_nor_rung() {
    let served = PushingRelay::new();
    let mailbox = Arc::new(Mailbox::default());
    let rung = Arc::new(AtomicUsize::new(0));
    let counted = rung.clone();
    mailbox.heard_by(move |_| {
        counted.fetch_add(1, Ordering::SeqCst);
    });
    let relay = Relay::new(&served.url, speaking(), mailbox.clone());
    relay.sent.wrote(&note().id);

    relay.watch(&Filter::of(Kind::Wrap), STREAM);

    // The connect rings. Past two ticks of silence the note has been read and answered for, so a second
    // ring here is the echo being taken for an arrival.
    let deadline = std::time::Instant::now() + TICK * 3;
    while std::time::Instant::now() < deadline {
        assert!(
            rung.load(Ordering::SeqCst) < 2,
            "this device's own note rang"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(rung.load(Ordering::SeqCst), 1, "and the connect still rang");
    let held = mailbox.take(&[STREAM.to_string()]).remove(STREAM).unwrap();
    assert!(
        held.walk && held.notes.is_empty(),
        "and nothing was held: {held:?}"
    );
    relay.close();
}
