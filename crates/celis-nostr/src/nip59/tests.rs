use super::*;

/// What this system wraps: a record, which states its own kind. The wrap layer never reads it, so any
/// number would pass — this one is the one it actually sees.
const KIND: u16 = Kind::Revision as u16;
const AT: u64 = 1_785_326_400;

fn account(byte: u8) -> Account {
    Account::new(&[byte; 32]).unwrap()
}

#[test]
fn a_message_opens_for_its_reader_and_names_the_author_who_sealed_it() {
    let (author, reader) = (account(3), account(4));
    let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

    let rumor = Rumor::unwrap(&reader, &note).unwrap();
    assert_eq!(rumor.content, "the invite");
    assert_eq!(rumor.pubkey, author.pubkey());
    assert_eq!((rumor.kind, rumor.created_at), (KIND, AT));
}

#[test]
fn nobody_else_opens_it_and_the_relay_sees_neither_party() {
    let (author, reader, stranger) = (account(3), account(4), account(5));
    let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

    assert!(Rumor::unwrap(&stranger, &note).is_err());
    assert!(
        Rumor::unwrap(&author, &note).is_err(),
        "not even the author reads it back — the carrier key is thrown away"
    );
    assert_ne!(
        note.pubkey,
        author.pubkey(),
        "a key minted for this message"
    );
    assert_eq!(
        note.tags,
        vec![vec!["p".to_string(), reader.pubkey().hex()]],
        "one tag, and no expiry — this message stands until it is answered"
    );
    assert!(note.verifies());
    assert!(!note.content.contains("the invite"));
}

#[test]
fn the_same_message_wraps_differently_every_time() {
    let (author, reader) = (account(3), account(4));
    let once = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "same words").unwrap();
    let twice = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "same words").unwrap();

    assert_ne!(once.pubkey, twice.pubkey, "a fresh carrier each send");
    assert_ne!(once.id, twice.id);
    assert_eq!(
        Rumor::unwrap(&reader, &once).unwrap(),
        Rumor::unwrap(&reader, &twice).unwrap()
    );
}

/// The wrap `author` sends `reader` carrying `rumor` as it stands, sealed by `sealer` — what a test
/// building a message by hand needs, since [`wrap`] mints its own rumor.
fn wrapped(sealer: &Account, reader: &Account, rumor: &Rumor) -> Note {
    let seal = Note::write(
        sealer,
        Kind::Seal,
        AT,
        Vec::new(),
        sealer
            .encrypt(&reader.pubkey(), &serde_json::to_string(rumor).unwrap())
            .unwrap(),
    )
    .unwrap();
    let carrier = Account::generate().unwrap();
    Note::write(
        &carrier,
        Kind::Wrap,
        AT,
        vec![vec!["p".to_string(), reader.pubkey().hex()]],
        carrier
            .encrypt(&reader.pubkey(), &serde_json::to_string(&seal).unwrap())
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn a_rumor_carries_every_field_a_nostr_event_does() {
    let (author, reader) = (account(3), account(4));
    let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

    let rumor = Rumor::unwrap(&reader, &note).unwrap();
    assert_eq!(rumor.id, rumor.digest(), "and its id is its own hash");
    assert_eq!(rumor.tags, Vec::<Vec<String>>::new());
    assert_eq!(
        serde_json::to_value(&rumor)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<String>>(),
        ["content", "created_at", "id", "kind", "pubkey", "tags"]
            .iter()
            .map(|it| it.to_string())
            .collect()
    );
}

#[test]
fn a_rumor_whose_id_does_not_describe_it_is_refused() {
    let (author, reader) = (account(3), account(4));
    let mut rumor = Rumor {
        id: String::new(),
        pubkey: author.pubkey(),
        created_at: AT,
        kind: KIND,
        tags: Vec::new(),
        content: "the invite".to_string(),
    };
    rumor.id = rumor.digest();
    assert!(Rumor::unwrap(&reader, &wrapped(&author, &reader, &rumor)).is_ok());

    rumor.content.push('!');
    assert!(Rumor::unwrap(&reader, &wrapped(&author, &reader, &rumor)).is_err());
}

#[test]
fn the_seal_is_pushed_back_and_the_wrap_carries_the_send_second() {
    let (author, reader) = (account(3), account(4));
    let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

    assert_eq!(note.created_at, AT, "the wrap says when it was sent");
    assert_eq!(
        Rumor::unwrap(&reader, &note).unwrap().created_at,
        AT,
        "and the rumor says when the message is from"
    );

    let seals: std::collections::BTreeSet<u64> = (0..24)
        .map(|_| {
            let note =
                Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();
            let seal: Note =
                serde_json::from_str(&reader.decrypt(&note.pubkey, &note.content).unwrap())
                    .unwrap();
            seal.created_at
        })
        .collect();
    assert!(seals.len() > 1, "drawn fresh each time: {seals:?}");
    assert!(
        seals.iter().all(|at| (AT - TWEAK..=AT).contains(at)),
        "and never ahead, nor further back than the NIP suggests: {seals:?}"
    );
}

#[test]
fn a_message_claiming_an_author_that_did_not_seal_it_is_refused() {
    let (liar, victim, reader) = (account(3), account(4), account(5));
    let mut rumor = Rumor {
        id: String::new(),
        pubkey: victim.pubkey(),
        created_at: AT,
        kind: KIND,
        tags: Vec::new(),
        content: "not mine".to_string(),
    };
    rumor.id = rumor.digest();
    assert!(Rumor::unwrap(&reader, &wrapped(&liar, &reader, &rumor)).is_err());
}

#[test]
fn an_expiring_message_says_so_where_the_relay_can_read_it() {
    let (author, reader) = (account(3), account(4));
    let note = Rumor::wrap(
        &author,
        &reader.pubkey(),
        KIND,
        AT,
        Some(AT + 60),
        "the receipt",
    )
    .unwrap();

    assert_eq!(note.expires_at(), Some(AT + 60));
    assert!(note.verifies(), "the expiry is signed, not appended after");
    assert_eq!(
        Rumor::unwrap(&reader, &note).unwrap().content,
        "the receipt",
        "and it still opens the same way"
    );
}

#[test]
fn a_tampered_wrap_and_a_note_of_another_kind_are_refused() {
    let (author, reader) = (account(3), account(4));
    let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

    let tampered = Note {
        content: "not the ciphertext it was".to_string(),
        ..note.clone()
    };
    assert!(Rumor::unwrap(&reader, &tampered).is_err());

    let plain = Note::write(&author, 1u16, AT, Vec::new(), "hello").unwrap();
    assert!(Rumor::unwrap(&reader, &plain).is_err());
}
