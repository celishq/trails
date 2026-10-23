use super::*;
use crate::step::proof_file::ProofFile;

/// A `done` carries its proof, what changed from the plan and who gave the proof, and reads back the
/// same (`spec/03-steps.md`, "Closing").
#[test]
fn a_done_carries_its_proof() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let done = Mark::Done {
        proof: "proof".into(),
        changed: Some("changed".into()),
        by: Some("judge".into()),
        files: vec![],
    };
    let note = done.record(&account, "p", 10).unwrap();
    assert_eq!(Mark::read(&note), Some(("p".to_string(), done)));
}

/// A `done` carries each file of its proof as its place, sha256 and size, and reads back the same
/// (`spec/08-files.md`, "Files of a done").
#[test]
fn a_done_carries_the_files_of_its_proof() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let file = ProofFile {
        place: "art/logo.svg".into(),
        sha256: "ba7816bf".into(),
        size: 3,
    };
    let done = Mark::Done {
        proof: "the logo".into(),
        changed: None,
        by: None,
        files: vec![file],
    };
    let note = done.record(&account, "p", 10).unwrap();
    assert!(note.tags.contains(&vec![
        "file".to_string(),
        "art/logo.svg".to_string(),
        "ba7816bf".to_string(),
        "3".to_string()
    ]));
    assert_eq!(Mark::read(&note), Some(("p".to_string(), done)));
}

/// A `done` with no proof and a `drop` with no reason read as no mark (`spec/03-steps.md`, "Closing";
/// "Dropping").
#[test]
fn an_empty_done_or_drop_is_no_mark() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let done = Mark::Done {
        proof: " ".into(),
        changed: None,
        by: None,
        files: vec![],
    };
    let drop = Mark::Drop { reason: "".into() };
    for mark in [done, drop] {
        let note = mark.record(&account, "p", 10).unwrap();
        assert_eq!(Mark::read(&note), None);
    }
}

/// A name carries the name on its trail, and reads back the same (`spec/07-names.md`, "Names").
#[test]
fn a_name_carries_its_name() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let name = Mark::Name {
        name: "review".into(),
    };
    let note = name.record(&account, "t", 10).unwrap();
    assert_eq!(Mark::read(&note), Some(("t".to_string(), name)));
}
