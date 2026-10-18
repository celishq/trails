use super::Store;
use celis_nostr::{Account, Note};

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
    assert_eq!(ours.records().len(), 2);
    assert!(home
        .path()
        .join("trails/shared")
        .join(file.file_name().unwrap())
        .exists());
}

/// A store keeps the files whose records it verified in `$CELIS_HOME/cache/verified`, and the next store
/// verifies a file again only when its size or time changed.
#[test]
fn a_read_verifies_again_only_files_that_changed() {
    let home = tempfile::tempdir().unwrap();
    let account = Account::generate().unwrap();
    let note =
        |text: &str| Note::write(&account, 1u16, 1, Vec::<Vec<String>>::new(), text).unwrap();
    let first = Store::open(home.path()).unwrap();
    let kept = first.write(&note("kept")).unwrap();
    let changed = first.write(&note("changed")).unwrap();
    assert_eq!(Store::open(home.path()).unwrap().records().len(), 2);
    let cache = std::fs::read_to_string(home.path().join("cache/verified")).unwrap();
    assert_eq!(cache.lines().count(), 2, "{cache}");
    let tamper = |path: &std::path::Path, from: &str, to: &str| {
        let time = std::fs::metadata(path).unwrap().modified().unwrap();
        let text = std::fs::read_to_string(path).unwrap().replace(from, to);
        std::fs::write(path, text).unwrap();
        let file = std::fs::File::options().write(true).open(path).unwrap();
        file.set_modified(time).unwrap();
    };
    tamper(&kept, "\"kept\"", "\"kelp\"");
    tamper(&changed, "\"changed\"", "\"changes!\"");
    let texts: Vec<String> = Store::open(home.path())
        .unwrap()
        .records()
        .into_iter()
        .map(|n| n.content)
        .collect();
    assert_eq!(texts, ["kelp"]);
}
