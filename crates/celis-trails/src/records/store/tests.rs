use super::Store;
use celis_nostr::{Account, Note};

/// A read of what is new gives each record another store wrote since the last read once, and adds it to
/// the records.
#[test]
fn fresh_gives_each_record_written_since_the_last_read_once() {
    let home = tempfile::tempdir().unwrap();
    let ours = Store::open(home.path()).unwrap();
    let theirs = Store::open(home.path()).unwrap();
    let account = Account::generate().unwrap();
    let note =
        |text: &str| Note::write(&account, 1u16, 1, Vec::<Vec<String>>::new(), text).unwrap();
    theirs.write(&note("first")).unwrap();
    assert_eq!(ours.records().len(), 1);
    assert!(ours.fresh().is_empty());
    let second = note("second");
    theirs.write(&second).unwrap();
    let fresh: Vec<String> = ours.fresh().into_iter().map(|n| n.id).collect();
    assert_eq!(fresh, [second.id]);
    assert!(ours.fresh().is_empty());
    assert_eq!(ours.records().len(), 2);
}
