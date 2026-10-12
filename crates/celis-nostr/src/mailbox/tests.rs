use super::*;
use crate::Account;
use std::sync::Arc;

fn note(content: &str) -> Note {
    Note::write(
        &Account::new(&[3u8; 32]).unwrap(),
        1059u16,
        1_700_000_000,
        Vec::new(),
        content,
    )
    .unwrap()
}

#[test]
fn a_note_is_held_before_whoever_is_listening_hears_it() {
    let mailbox = Arc::new(Mailbox::default());
    let seen: Arc<Mutex<Vec<BTreeSet<String>>>> = Arc::new(Mutex::new(Vec::new()));
    let (held, told) = (mailbox.clone(), seen.clone());
    mailbox.heard_by(move |_| told.lock().unwrap().push(held.all()));

    mailbox.heard("abc/records", note("one"));

    assert_eq!(
        seen.lock().unwrap().as_slice(),
        [BTreeSet::from(["abc/records".to_string()])],
        "whoever was listening was told before the note was held"
    );
}

#[test]
fn a_take_empties_the_streams_named_and_leaves_the_others() {
    let mailbox = Mailbox::default();
    mailbox.heard("abc/records", note("one"));
    mailbox.heard("abc/records", note("two"));
    mailbox.heard("def/records", note("three"));

    let taken = mailbox.take(&["abc/records".to_string()]);

    let abc = &taken["abc/records"];
    assert_eq!(abc.notes.len(), 2);
    assert!(!abc.walk, "a note off the socket owes no walk");
    assert_eq!(mailbox.all(), BTreeSet::from(["def/records".to_string()]));
}

#[test]
fn a_ring_holds_no_note_and_owes_the_walk_and_a_storm_of_them_is_one_entry() {
    let mailbox = Mailbox::default();
    for _ in 0..5 {
        mailbox.ring("abc/records");
    }

    let taken = mailbox.take(&["abc/records".to_string()]);

    assert_eq!(taken.len(), 1);
    assert!(taken["abc/records"].notes.is_empty());
    assert!(taken["abc/records"].walk);
    assert!(mailbox.all().is_empty());
}

#[test]
fn a_stream_no_longer_read_is_dropped() {
    let mailbox = Mailbox::default();
    mailbox.heard("abc/records", note("one"));
    mailbox.ring("gone/records");

    let kept = mailbox.only(&["abc/records".to_string()]);

    assert_eq!(kept, BTreeSet::from(["abc/records".to_string()]));
    assert_eq!(mailbox.all(), kept, "the one nothing reads is still heard");
}

#[test]
fn a_listener_may_ring_back_in_from_inside_the_call() {
    let mailbox = Arc::new(Mailbox::default());
    let again = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (held, once) = (mailbox.clone(), again.clone());
    mailbox.heard_by(move |stream| {
        if stream == "abc/records" && !once.swap(true, std::sync::atomic::Ordering::SeqCst) {
            held.ring("def/records");
        }
    });

    let rang = std::thread::spawn({
        let mailbox = mailbox.clone();
        move || mailbox.heard("abc/records", note("one"))
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !rang.is_finished() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert!(
        rang.is_finished(),
        "the ring came back rather than waiting on itself"
    );
    assert_eq!(
        mailbox.all(),
        BTreeSet::from(["abc/records".to_string(), "def/records".to_string()])
    );
}
