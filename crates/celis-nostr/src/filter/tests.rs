use super::*;
use crate::Account;
use serde_json::json;

fn key(seed: u8) -> Pubkey {
    Account::new(&[seed; 32]).unwrap().pubkey()
}

#[test]
fn an_absent_condition_is_absent_from_the_frame_rather_than_null() {
    assert_eq!(serde_json::to_value(Filter::default()).unwrap(), json!({}));
    assert_eq!(
        serde_json::to_value(Filter {
            kinds: Some(vec![Kind::Record]),
            since: Some(42),
            ..Filter::default()
        })
        .unwrap(),
        json!({"kinds": [35000], "since": 42})
    );
}

#[test]
fn a_kind_and_an_author_travel_as_the_numbers_and_hex_a_relay_reads() {
    let mine = key(3);
    assert_eq!(
        serde_json::to_value(Filter {
            authors: Some(vec![mine]),
            ..Filter::of(Kind::Server)
        })
        .unwrap(),
        json!({"authors": [mine.hex()], "kinds": [35010]})
    );
}

#[test]
fn a_tag_travels_under_its_own_name_behind_a_hash() {
    let reader = key(4);
    assert_eq!(
        serde_json::to_value(Filter::of(Kind::Wrap).tagged('p', reader.hex())).unwrap(),
        json!({"kinds": [1059], "#p": [reader.hex()]})
    );
    assert_eq!(
        serde_json::to_value(
            Filter::of(Kind::Membership)
                .tagged('h', "a-server")
                .tagged('p', "someone")
        )
        .unwrap(),
        json!({"kinds": [35011], "#h": ["a-server"], "#p": ["someone"]})
    );
}

#[test]
/// A relay answers a note matching any value under one tag, so asking twice widens the subscription rather
/// than narrowing it to notes carrying both.
fn asking_twice_under_one_name_keeps_both_values() {
    assert_eq!(
        serde_json::to_value(Filter::default().tagged('h', "one").tagged('h', "two")).unwrap(),
        json!({"#h": ["one", "two"]})
    );
}

#[test]
fn a_page_states_its_own_window() {
    assert_eq!(
        serde_json::to_value(Filter {
            until: Some(412),
            limit: Some(500),
            ..Filter::default()
        })
        .unwrap(),
        json!({"until": 412, "limit": 500})
    );
}
