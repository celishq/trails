//! CIP-10, "The store".

mod support;

use celis_cairns::Store;
use celis_wire::Note;
use support::{account, err, out, Repo};

/// A command opens the store in `$CELIS_HOME`, so a cairn written in one project lists in another folder,
/// inside git or not (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn a_command_must_open_the_store_in_celis_home_whatever_the_working_folder() {
    let repo = Repo::new();
    let text = "Before you run make, set CI=1.";
    let wrote = repo.run(&repo.path(), &["add", "--task", "t", "--", text], &[], "");
    assert!(wrote.status.success(), "{}", err(&wrote));
    let id = out(&wrote).trim().to_string();
    let rated = repo.run(&repo.path(), &["rate", &id, "+"], &[], "");
    assert!(rated.status.success(), "{}", err(&rated));
    let tree = repo.worktree("feat-a");
    let outside = tempfile::tempdir().unwrap();
    for folder in [tree.as_path(), outside.path()] {
        let listed = repo.run(folder, &["list"], &[], "");
        assert!(
            out(&listed).contains(text),
            "{}: {}",
            folder.display(),
            out(&listed)
        );
        assert!(!folder.join(".celis").exists(), "{}", folder.display());
    }
    assert!(repo.store().join("shared").is_dir());
    assert!(!repo.path().join(".celis").exists());
}

/// A record is written to `<created_at>-<id>.json` holding the record as NIP-01 JSON
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn a_record_must_be_written_to_a_file_named_by_its_time_and_id() {
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let record = Note::write(&account(), 1111u16, 1759400060, vec![], "first").unwrap();
    let path = store.write(&record).unwrap();
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        format!("{}-{}.json", record.created_at, record.id)
    );
    let read: Note = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(read, record);
}

/// A record file does not change once written, and a file still being written is not seen
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn a_record_file_must_not_change_once_written() {
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let record = Note::write(&account(), 1111u16, 1759400060, vec![], "first").unwrap();
    let shared = repo.store().join("shared");
    let path = shared.join(format!("{}-{}.json", record.created_at, record.id));
    std::fs::write(&path, "already here").unwrap();
    store.write(&record).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "already here");

    let other = Note::write(&account(), 1111u16, 1759400070, vec![], "second").unwrap();
    std::fs::write(
        shared.join(format!(".{}-{}.json.1", other.created_at, other.id)),
        serde_json::to_string(&other).unwrap(),
    )
    .unwrap();
    assert!(!store.records().contains(&other));
}

/// A reader never sees a record file half written, however large the record
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn a_record_file_must_never_be_seen_half_written() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let shared = repo.store().join("shared");
    let content = "x".repeat(32 * 1024 * 1024);
    let record = Note::write(&account(), 1111u16, 1759400060, vec![], &content).unwrap();
    let done = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let watch = scope.spawn(|| {
            while !done.load(Ordering::SeqCst) {
                for entry in std::fs::read_dir(&shared).unwrap().flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|x| x == "json") {
                        let text = std::fs::read_to_string(&path).unwrap_or_default();
                        let note: Note = serde_json::from_str(&text)
                            .unwrap_or_else(|e| panic!("a named file is half written: {e}"));
                        assert_eq!(note.id, record.id);
                    }
                }
            }
        });
        store.write(&record).unwrap();
        done.store(true, Ordering::SeqCst);
        watch.join().unwrap()
    });
    assert!(store.records().contains(&record));
}

/// A reader skips a file that does not parse or whose record does not verify
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn a_reader_must_skip_a_file_that_does_not_parse_or_verify() {
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let good = Note::write(&account(), 1111u16, 1759400060, vec![], "good").unwrap();
    store.write(&good).unwrap();

    let shared = repo.store().join("shared");
    std::fs::write(shared.join("1759400061-garbage.json"), "{ not json").unwrap();
    let mut forged = Note::write(&account(), 1111u16, 1759400062, vec![], "forged").unwrap();
    forged.content = "changed after signing".into();
    std::fs::write(
        shared.join(format!("1759400062-{}.json", forged.id)),
        serde_json::to_string(&forged).unwrap(),
    )
    .unwrap();

    assert_eq!(store.records(), vec![good]);
}

/// `local/` holds only the log, and no record file carries the account's secret
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn local_must_not_leave_the_machine() {
    let repo = Repo::new();
    let wrote = repo.run(
        &repo.path(),
        &["add", "--task", "t", "--", "Before you run make, set CI=1."],
        &[],
        "",
    );
    assert!(wrote.status.success(), "{}", err(&wrote));
    let secret = std::fs::read_to_string(repo.home().join("key")).unwrap();
    let names: Vec<_> = std::fs::read_dir(repo.store().join("local"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("log")]);
    for entry in std::fs::read_dir(repo.store().join("shared")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!text.contains(secret.trim()));
    }
}

/// With no store, `list --all`, `show` and `log` answer as an empty store and create nothing
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn a_read_with_no_store_must_answer_empty_and_create_nothing() {
    let repo = Repo::new();
    for args in [&["list", "--all"][..], &["show", "abcdef12"], &["log"]] {
        let o = repo.run(&repo.path(), args, &[], "");
        assert!(out(&o).is_empty(), "{args:?}: {}", out(&o));
        assert!(
            args[0] == "show" || o.status.success(),
            "{args:?}: {}",
            err(&o)
        );
        assert!(!repo.store().exists(), "{args:?}");
    }
}

/// With no store, `stats` answers as an empty store, in text and under `--json`, and creates nothing
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn stats_with_no_store_must_answer_empty_and_create_nothing() {
    let repo = Repo::new();
    for args in [&["stats"][..], &["stats", "--json"]] {
        let o = repo.run(&repo.path(), args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
        if args.contains(&"--json") {
            let object: serde_json::Value = serde_json::from_str(out(&o).trim()).unwrap();
            assert_eq!(
                object,
                serde_json::json!({"days": 7, "sessions": 0, "reported": 0, "done": 0}),
                "{args:?}"
            );
            assert_eq!(out(&o).trim().lines().count(), 1, "{args:?}");
        } else {
            assert_eq!(
                out(&o).trim(),
                "0 sessions, 0 of 0 tasks done, in 7 days",
                "{args:?}"
            );
        }
        assert!(!repo.store().exists(), "{args:?}");
    }
}

/// With no store, `show` refuses with `no-cairn` and creates nothing
/// (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn show_with_no_store_must_refuse_with_no_cairn() {
    let repo = Repo::new();
    let o = repo.run(&repo.path(), &["show", "--json", "abcdef12"], &[], "");
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let object: serde_json::Value = serde_json::from_str(err(&o).trim()).unwrap();
    assert_eq!(object["error"], "no-cairn");
    assert!(!repo.store().exists());
}

/// With no store, a command that writes creates it (`spec/cairns/10-cairns.md`, "The store").
#[test]
fn with_no_store_every_other_command_must_create_it() {
    let repo = Repo::new();
    let o = repo.run(&repo.path(), &["list"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(repo.store().join("shared").is_dir());
}
