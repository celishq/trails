use super::*;

#[test]
fn a_phrase_deterministically_rebuilds_the_same_nostr_account() {
    let original = Identity::generate().unwrap();
    let restored = Identity::from_phrase(&original.phrase()).unwrap();
    assert_eq!(
        original.account().pubkey(),
        restored.account().pubkey(),
        "the phrase restores the account every note is signed by"
    );

    let other = Identity::generate().unwrap();
    assert_ne!(
        original.account().pubkey(),
        other.account().pubkey(),
        "another phrase is another account"
    );
}

#[test]
fn a_note_this_account_signs_verifies_as_this_account() {
    let id = Identity::generate().unwrap();
    let note = crate::Note::write(id.account(), 1u16, 1_700_000_000, vec![], "hello").unwrap();
    assert!(note.verifies());
    assert_eq!(note.pubkey, id.account().pubkey());
}

#[test]
fn a_bad_phrase_is_rejected() {
    assert!(Identity::from_phrase("not a real mnemonic phrase at all").is_err());
}

/// A phrase gives the same account on every host and in the app.
#[test]
fn the_fixture_phrase_gives_the_fixture_npub() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../spec/fixtures/account-phrase.json"
    ))
    .unwrap();
    let identity = Identity::from_phrase(fixture["phrase"].as_str().unwrap()).unwrap();
    assert_eq!(
        identity.account().pubkey().npub(),
        fixture["npub"].as_str().unwrap()
    );
}
