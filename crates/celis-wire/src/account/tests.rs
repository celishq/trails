use super::*;

fn account(byte: u8) -> Account {
    Account::new(&[byte; 32]).unwrap()
}

#[test]
fn a_secret_reads_the_same_as_an_nsec() {
    let mut secret = [0u8; 32];
    secret[31] = 1;
    let account = Account::new(&secret).unwrap();
    assert_eq!(
        account.nsec(),
        "nsec1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsmhltgl",
        "the nsec other clients show for the scalar 1"
    );
    assert_eq!(
        Account::parse_nsec(&account.nsec()).unwrap().pubkey(),
        account.pubkey()
    );
}

#[test]
fn a_signature_verifies_against_the_account_that_made_it_and_no_other() {
    let mine = account(3);
    let theirs = account(4);
    let message = [7u8; 32];
    let signature = mine.sign(&message).unwrap();

    assert!(mine.pubkey().verifies(&message, &signature));
    assert!(!theirs.pubkey().verifies(&message, &signature));
    assert!(!mine.pubkey().verifies(&[8u8; 32], &signature));
}

#[test]
fn two_accounts_derive_the_same_conversation_key_from_opposite_sides() {
    let mine = account(3);
    let theirs = account(4);
    assert_eq!(
        mine.conversation_key(&theirs.pubkey()).unwrap(),
        theirs.conversation_key(&mine.pubkey()).unwrap()
    );
}

#[test]
fn a_reader_opens_what_was_encrypted_to_them_and_a_stranger_does_not() {
    let mine = account(3);
    let theirs = account(4);
    let stranger = account(5);

    let payload = mine.encrypt(&theirs.pubkey(), "the entry").unwrap();
    assert_eq!(
        theirs.decrypt(&mine.pubkey(), &payload).unwrap(),
        "the entry"
    );
    assert!(stranger.decrypt(&mine.pubkey(), &payload).is_err());
}

#[test]
fn encrypting_to_yourself_is_the_ordinary_case_with_no_extra_key() {
    let mine = account(3);
    let payload = mine.encrypt(&mine.pubkey(), "a fleet entry").unwrap();
    assert_eq!(
        mine.decrypt(&mine.pubkey(), &payload).unwrap(),
        "a fleet entry"
    );
}

#[test]
fn the_same_plaintext_encrypts_differently_every_time() {
    let mine = account(3);
    let reader = account(4).pubkey();
    assert_ne!(
        mine.encrypt(&reader, "same words").unwrap(),
        mine.encrypt(&reader, "same words").unwrap(),
        "a fresh nonce each call"
    );
}

#[test]
fn the_same_plaintext_always_encrypts_the_same_way_deterministically() {
    let mine = account(3);
    let elsewhere = account(3);
    let reader = account(4).pubkey();

    let once = mine.encrypt_deterministic(&reader, "same words").unwrap();
    assert_eq!(
        once,
        mine.encrypt_deterministic(&reader, "same words").unwrap()
    );
    assert_eq!(
        once,
        elsewhere
            .encrypt_deterministic(&reader, "same words")
            .unwrap(),
        "another device holding the same key derives the same ciphertext"
    );
    assert_ne!(
        once,
        mine.encrypt_deterministic(&reader, "other words").unwrap(),
        "a different plaintext takes a different nonce"
    );
    assert_eq!(
        account(4).decrypt(&mine.pubkey(), &once).unwrap(),
        "same words",
        "and it is still an ordinary NIP-44 payload the reader opens"
    );
}

#[test]
fn signing_the_same_id_twice_gives_the_same_signature() {
    let mine = account(3);
    assert_eq!(
        mine.sign(&[7u8; 32]).unwrap(),
        mine.sign(&[7u8; 32]).unwrap()
    );
}
