use super::{name, Pack};
use celis_nostr::{Account, Note};

/// A pack gives the record of each line by its file's name, skips a line that does not parse, and gives
/// a record added and saved to the next pack of its file.
#[test]
fn a_pack_gives_each_record_it_kept_by_its_files_name() {
    let home = tempfile::tempdir().unwrap();
    let file = home.path().join("pack");
    let account = Account::generate().unwrap();
    let note = Note::write(&account, 1u16, 1, Vec::<Vec<String>>::new(), "kept").unwrap();
    std::fs::write(&file, "broken\n").unwrap();
    let pack = Pack::open(file.clone());
    assert!(pack.get(&name(&note)).is_none());
    pack.add(&note);
    pack.save();
    let again = Pack::open(file);
    assert_eq!(
        again.get(&name(&note)).map(|n| n.content.as_str()),
        Some("kept")
    );
}
