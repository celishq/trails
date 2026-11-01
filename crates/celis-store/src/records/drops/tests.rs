use super::Drops;
use celis_nostr::{Account, Note};

/// The drop of `id` that `account` signs at `at` with the reason `reason`.
fn drop(account: &Account, id: &str, at: u64, reason: &str) -> Note {
    let tags = vec![
        vec!["L".to_string(), "celis.step".to_string()],
        vec![
            "l".to_string(),
            "drop".to_string(),
            "celis.step".to_string(),
        ],
        vec!["e".to_string(), id.to_string()],
    ];
    Note::write(account, 1985u16, at, tags, reason).unwrap()
}

/// The first drop with a reason counts, and a drop with no content is ignored (`spec/03-steps.md`,
/// "Dropping").
#[test]
fn the_first_drop_with_a_reason_counts() {
    let account = Account::generate().unwrap();
    let records = [
        drop(&account, "a", 3, "later"),
        drop(&account, "a", 1, " "),
        drop(&account, "a", 2, "first"),
    ];
    let drops = Drops::of(&records);
    assert_eq!(drops.get("a").map(|n| n.content.as_str()), Some("first"));
    assert!(drops.get("b").is_none());
}
