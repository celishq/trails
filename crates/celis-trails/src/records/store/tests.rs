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

/// A read of named files gives each record of them it lacks once, from `shared/` or another folder, adds
/// it to the records, and copies one from another folder into `shared/`; a file that does not parse is
/// skipped.
#[test]
fn take_gives_each_record_of_the_named_files_once() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let ours = Store::open(home.path()).unwrap();
    let theirs = Store::open(home.path()).unwrap();
    let account = Account::generate().unwrap();
    let note =
        |text: &str| Note::write(&account, 1u16, 1, Vec::<Vec<String>>::new(), text).unwrap();
    assert!(ours.records().is_empty());
    let shared = theirs.write(&note("shared")).unwrap();
    let linked = note("linked");
    let file = elsewhere
        .path()
        .join(format!("{}-{}.json", linked.created_at, linked.id));
    std::fs::write(&file, serde_json::to_string(&linked).unwrap()).unwrap();
    let broken = elsewhere.path().join("1-broken.json");
    std::fs::write(&broken, "{").unwrap();
    let paths = [shared.clone(), file.clone(), broken];
    let taken: Vec<String> = ours.take(&paths).into_iter().map(|n| n.content).collect();
    assert_eq!(taken, ["shared", "linked"]);
    assert!(ours.take(&paths).is_empty());
    assert!(ours.fresh().is_empty());
    assert_eq!(ours.records().len(), 2);
    assert!(home
        .path()
        .join("trails/shared")
        .join(file.file_name().unwrap())
        .exists());
}
