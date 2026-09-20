use super::*;

/// Two notes that have been on the network since 2022, signed by other clients. Nothing here was
/// produced by this crate, so agreeing with them is a real check and not a round trip.
const WILD_KIND_1: &str = r#"{"kind":1,"id":"dc90c95f09947507c1044e8f48bcf6350aa6bff1507dd4acfc755b9239b5c962","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1644271588,"tags":[],"content":"now that https://blueskyweb.org/blog/2-7-2022-overview was announced we can stop working on nostr?","sig":"230e9d8f0ddaf7eb70b5f7741ccfa37e87a455c9a469282e3464e2052d3192cd63a167e196e381ef9d7e69e9ea43af2443b839974dc85d8aaab9efe1d9296524"}"#;

/// Tags, and a content string full of quotes and slashes — the escaping the id depends on.
const WILD_KIND_3: &str = r#"{"kind":3,"id":"9e662bdd7d8abc40b5b15ee1ff5e9320efc87e9274d8d440c58e6eed2dddfbe2","pubkey":"373ebe3d45ec91977296a178d9f19f326c70631d2a1b0bbba5c5ecc2eb53b9e7","created_at":1644844224,"tags":[["p","3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d"],["p","75fc5ac2487363293bd27fb0d14fb966477d0f1dbc6361d37806a6a740eda91e"],["p","46d0dfd3a724a302ca9175163bdf788f3606b3fd1bb12d5fe055d1e418cb60ea"]],"content":"{\"wss://nostr-pub.wellorder.net\":{\"read\":true,\"write\":true},\"wss://nostr.bitcoiner.social\":{\"read\":false,\"write\":true},\"wss://expensive-relay.fiatjaf.com\":{\"read\":true,\"write\":true},\"wss://relayer.fiatjaf.com\":{\"read\":true,\"write\":true},\"wss://relay.bitid.nz\":{\"read\":true,\"write\":true},\"wss://nostr.rocks\":{\"read\":true,\"write\":true}}","sig":"811355d3484d375df47581cb5d66bed05002c2978894098304f20b595e571b7e01b2efd906c5650080ffe49cf1c62b36715698e9d88b9e8be43029a2f3fa66be"}"#;

fn wild(json: &str) -> Note {
    serde_json::from_str(json).unwrap()
}

#[test]
fn our_id_reproduces_the_id_another_client_gave_a_note() {
    for json in [WILD_KIND_1, WILD_KIND_3] {
        let note = wild(json);
        assert_eq!(note.digest(), note.id);
    }
}

#[test]
fn a_note_from_the_wild_verifies() {
    for json in [WILD_KIND_1, WILD_KIND_3] {
        assert!(wild(json).verifies());
    }
}

#[test]
fn editing_a_note_after_it_was_signed_breaks_it() {
    let note = wild(WILD_KIND_1);
    let edited = Note {
        content: "something else".into(),
        ..note.clone()
    };
    assert_ne!(edited.digest(), edited.id, "the id no longer describes it");
    assert!(!edited.verifies());

    let relabelled = Note {
        id: edited.digest(),
        ..edited
    };
    assert!(
        !relabelled.verifies(),
        "recomputing the id does not repair the signature"
    );
}

#[test]
fn a_note_survives_a_json_round_trip_unchanged() {
    let note = wild(WILD_KIND_3);
    let restored: Note = serde_json::from_str(&serde_json::to_string(&note).unwrap()).unwrap();
    assert_eq!(restored, note);
    assert!(restored.verifies());
}

#[test]
fn a_note_we_write_verifies_and_carries_its_tags() {
    let account = Account::new(&[9u8; 32]).unwrap();
    let note = Note::write(
        &account,
        35_000u16,
        1_700_000_000,
        vec![
            vec!["h".into(), "group-1".into()],
            vec!["d".into(), "entry-1".into()],
        ],
        "ciphertext",
    )
    .unwrap();

    assert!(note.verifies());
    assert_eq!(note.pubkey, account.pubkey());
    assert_eq!(note.tag("h"), Some("group-1"));
    assert_eq!(note.tag("d"), Some("entry-1"));
    assert_eq!(note.tag("p"), None);
}

#[test]
fn the_id_covers_every_signed_field() {
    let account = Account::new(&[9u8; 32]).unwrap();
    let note = Note::write(&account, 1u16, 1_700_000_000, vec![], "hello").unwrap();
    let vary = |other: Note| assert_ne!(other.digest(), note.id);

    vary(Note {
        content: "hello ".into(),
        ..note.clone()
    });
    vary(Note {
        created_at: note.created_at + 1,
        ..note.clone()
    });
    vary(Note {
        kind: 2,
        ..note.clone()
    });
    vary(Note {
        tags: vec![vec!["d".into(), "x".into()]],
        ..note.clone()
    });
}
