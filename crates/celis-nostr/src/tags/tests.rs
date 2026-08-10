use super::*;
use crate::{Account, Kind, Note};

#[test]
fn a_tag_is_its_name_then_its_value() {
    assert_eq!(
        Vec::from(Tags::new().tag("d", "entry-1").tag("h", "a-server")),
        vec![
            vec!["d".to_string(), "entry-1".to_string()],
            vec!["h".to_string(), "a-server".to_string()],
        ]
    );
}

#[test]
fn a_flag_states_nothing_beside_its_name() {
    assert_eq!(
        Vec::from(Tags::new().tag("h", "a-server").flag("-")),
        vec![
            vec!["h".to_string(), "a-server".to_string()],
            vec!["-".to_string()],
        ]
    );
}

#[test]
/// A note's id covers its tags as they were written, so two names spelled alike both stay and the order
/// they were stated in is the order they keep.
fn nothing_is_reordered_or_dropped() {
    let tags = Tags::new()
        .tag("p", "one")
        .tag("d", "between")
        .tag("p", "two");
    assert_eq!(
        Vec::from(tags),
        vec![
            vec!["p".to_string(), "one".to_string()],
            vec!["d".to_string(), "between".to_string()],
            vec!["p".to_string(), "two".to_string()],
        ]
    );
}

#[test]
fn a_note_built_from_tags_reads_them_back() {
    let account = Account::new(&[9u8; 32]).unwrap();
    let note = Note::write(
        &account,
        Kind::Epoch,
        1_700_000_000,
        Tags::new()
            .tag("d", "the-address")
            .tag("h", "a-server")
            .flag("-"),
        "",
    )
    .unwrap();

    assert!(note.verifies());
    assert_eq!(note.tag("d"), Some("the-address"));
    assert_eq!(note.tag("h"), Some("a-server"));
    assert_eq!(note.tag("-"), None, "a flag states no value to read");
}
