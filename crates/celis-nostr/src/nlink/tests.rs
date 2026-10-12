use super::*;

#[derive(serde::Deserialize)]
struct Case {
    secret: String,
    relay: String,
    nlink: String,
}

fn case() -> Case {
    serde_json::from_str(include_str!("../../../../spec/fixtures/nlink.json")).unwrap()
}

fn key(secret: &str) -> Account {
    let bytes: [u8; 32] = data_encoding::HEXLOWER
        .decode(secret.as_bytes())
        .unwrap()
        .try_into()
        .unwrap();
    Account::new(&bytes).unwrap()
}

/// Every end that writes or reads a guest link's URL matches `spec/fixtures/nlink.json`
#[test]
fn a_guest_link_encodes_as_the_fixture_states() {
    let case = case();
    let link = Nlink {
        key: key(&case.secret),
        relay: case.relay.clone(),
    };
    assert_eq!(link.text().unwrap(), case.nlink);
}

#[test]
fn a_guest_link_reads_back_its_key_and_relay() {
    let case = case();
    let link = Nlink::parse(&case.nlink).unwrap();
    assert_eq!(link.key.pubkey(), key(&case.secret).pubkey());
    assert_eq!(link.relay, case.relay);
}

#[test]
fn a_string_that_is_no_guest_link_is_refused() {
    let nsec = key(&case().secret).nsec();
    assert!(Nlink::parse(&nsec).is_err());
    assert!(Nlink::parse("nlink1").is_err());
}

#[test]
fn a_relay_too_long_for_one_entry_is_refused() {
    let link = Nlink {
        key: key(&case().secret),
        relay: format!("wss://{}", "a".repeat(250)),
    };
    assert!(link.text().is_err());
}
