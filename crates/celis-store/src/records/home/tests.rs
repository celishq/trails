use super::Home;
use celis_nostr::{Account, Note};

/// A store keeps each record it verified in `$CELIS_HOME/cache/pack`, and the next store takes it from
/// there while its file is in `shared/`, whatever the file holds now; a file that does not verify is
/// neither read nor packed.
#[test]
fn a_read_takes_the_verified_records_from_the_pack() {
    let home = tempfile::tempdir().unwrap();
    let account = Account::generate().unwrap();
    let note =
        |text: &str| Note::write(&account, 1u16, 1, Vec::<Vec<String>>::new(), text).unwrap();
    let first = Home::open(home.path()).unwrap();
    let kept = first.write(&note("kept"), &[]).unwrap().0;
    let gone = first.write(&note("gone"), &[]).unwrap().0;
    let mut forged = note("forged");
    forged.content = "forged!".into();
    let forged_file = kept.with_file_name(format!("{}-{}.json", forged.created_at, forged.id));
    std::fs::write(&forged_file, serde_json::to_string(&forged).unwrap()).unwrap();
    assert_eq!(Home::open(home.path()).unwrap().records().len(), 2);
    let pack = std::fs::read_to_string(home.path().join("cache/pack")).unwrap();
    assert_eq!(pack.lines().count(), 2, "{pack}");
    let text = std::fs::read_to_string(&kept).unwrap();
    std::fs::write(&kept, text.replace("\"kept\"", "\"kelp\"")).unwrap();
    std::fs::remove_file(&gone).unwrap();
    let texts: Vec<String> = Home::open(home.path())
        .unwrap()
        .records()
        .into_iter()
        .map(|n| n.content)
        .collect();
    assert_eq!(texts, ["kept"]);
}

/// A kept file is a blob in `blobs/`, named by the sha256 of its bytes and its extension, once; a blob
/// whose bytes do not hash to its name reads as missing (`spec/08-files.md`, "Blobs").
#[test]
fn a_kept_file_is_a_blob_named_by_its_sha256_and_extension() {
    let home = tempfile::tempdir().unwrap();
    let store = Home::open(home.path()).unwrap();
    let sha = store.keep(b"abc", "txt").unwrap();
    assert_eq!(
        sha,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(store.keep(b"abc", "md").unwrap(), sha);
    let path = store.blob(&sha).unwrap();
    assert_eq!(
        path,
        home.path().join("trails/blobs").join(format!("{sha}.txt"))
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"abc");
    std::fs::write(&path, b"abd").unwrap();
    assert_eq!(store.blob(&sha), None);
}

/// A blob named by its sha256 alone, as older versions wrote it, reads as the blob of that sha256
/// (`spec/08-files.md`, "Blobs").
#[test]
fn a_blob_named_by_its_sha256_alone_still_reads() {
    let home = tempfile::tempdir().unwrap();
    let store = Home::open(home.path()).unwrap();
    let sha = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let folder = home.path().join("trails/blobs");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join(sha), b"abc").unwrap();
    assert_eq!(store.blob(sha), Some(folder.join(sha)));
}
