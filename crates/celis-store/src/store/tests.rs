use super::Store;
use crate::Home;
use celis_nostr::{Account, Note};

/// A take of named files gives each record of them it lacks once, from `shared/` or another folder, adds
/// it to the records, and copies one from another folder into `shared/`; a file that does not parse is
/// skipped.
#[test]
fn take_gives_each_record_of_the_named_files_once() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let mut ours = Store::open(Home::open(home.path()).unwrap(), 1, 600);
    let theirs = Home::open(home.path()).unwrap();
    let account = Account::generate().unwrap();
    let note =
        |text: &str| Note::write(&account, 1u16, 1, Vec::<Vec<String>>::new(), text).unwrap();
    assert!(ours.records().is_empty());
    let shared = theirs.write(&note("shared"), &[]).unwrap().0;
    let linked = note("linked");
    let file = elsewhere
        .path()
        .join(format!("{}-{}.json", linked.created_at, linked.id));
    std::fs::write(&file, serde_json::to_string(&linked).unwrap()).unwrap();
    let broken = elsewhere.path().join("1-broken.json");
    std::fs::write(&broken, "{").unwrap();
    let paths = [shared, file.clone(), broken];
    let taken: Vec<String> = ours.take(&paths).into_iter().map(|n| n.content).collect();
    assert_eq!(taken, ["shared", "linked"]);
    assert!(ours.take(&paths).is_empty());
    assert_eq!(ours.records().len(), 2);
    assert!(home
        .path()
        .join("trails/shared")
        .join(file.file_name().unwrap())
        .exists());
}

/// A record it writes joins its records at the next moment, once.
#[test]
fn a_written_record_joins_at_the_next_moment() {
    let home = tempfile::tempdir().unwrap();
    let account = Account::generate().unwrap();
    let mut store = Store::open(Home::open(home.path()).unwrap(), 1, 600);
    let note = Note::write(&account, 1u16, 1, Vec::<Vec<String>>::new(), "new").unwrap();
    store.put(&note).unwrap();
    store.put(&note).unwrap();
    assert!(store.records().is_empty());
    store.at(2);
    assert_eq!(store.records().len(), 1);
}
