use super::*;
use crate::{By, Done, ProofFile};

/// A `done` carries its proof, what changed from the plan and who gave the proof, and reads back the
/// same (`spec/03-steps.md`, "Closing").
#[test]
fn a_done_carries_its_proof() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let done = Mark::Done(Done {
        proof: "proof".into(),
        changed: Some("changed".into()),
        by: Some(By::Judge),
        files: vec![],
    });
    let note = done.record(&account, "p", 10).unwrap();
    assert_eq!(Mark::read(&note), Some(("p".to_string(), done)));
}

/// A `done` names each file of its proof by an `imeta` tag and its Blossom URI in the content, and reads
/// back the same (`spec/08-files.md`, "Files of a done").
#[test]
fn a_done_names_the_files_of_its_proof() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let file = ProofFile::new("art/Logo.SVG".into(), "ba7816bf".into(), 3);
    let done = Mark::Done(Done {
        proof: "the logo".into(),
        changed: None,
        by: None,
        files: vec![file],
    });
    let note = done.record(&account, "p", 10).unwrap();
    let url = format!("blossom:ba7816bf.svg?as={}&sz=3", account.pubkey().hex());
    assert!(note.tags.contains(&vec![
        "imeta".to_string(),
        format!("url {url}"),
        "x ba7816bf".to_string(),
        "m image/svg+xml".to_string(),
        "size 3".to_string(),
        "path art/Logo.SVG".to_string(),
    ]));
    assert_eq!(note.content, format!("the logo\n{url}"));
    assert_eq!(Mark::read(&note), Some(("p".to_string(), done)));
}

/// A file with no extension it knows has the media type `application/octet-stream` and a `bin` URI
/// (`spec/08-files.md`, "Files of a done").
#[test]
fn an_unknown_file_is_octet_stream_and_bin() {
    let file = ProofFile::new("build/out".into(), "ab".into(), 1);
    assert_eq!(file.media, "application/octet-stream");
    assert_eq!(file.url("pk"), "blossom:ab.bin?as=pk&sz=1");
}

/// A `done` with no proof and a `drop` with no reason read as no mark (`spec/03-steps.md`, "Closing";
/// "Dropping").
#[test]
fn an_empty_done_or_drop_is_no_mark() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let done = Mark::Done(Done {
        proof: " ".into(),
        changed: None,
        by: None,
        files: vec![],
    });
    let drop = Mark::Drop { reason: "".into() };
    for mark in [done, drop] {
        let note = mark.record(&account, "p", 10).unwrap();
        assert_eq!(Mark::read(&note), None);
    }
}

/// A reader ignores a name, which older versions wrote on a trail (`spec/README.md`, "Names").
#[test]
fn a_name_reads_as_no_mark() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let prompt = Mark::Prompt {
        text: "review".into(),
    };
    let mut note = prompt.record(&account, "t", 10).unwrap();
    for tag in note.tags.iter_mut().filter(|t| t[0] == "l") {
        tag[1] = "name".into();
    }
    assert_eq!(Mark::read(&note), None);
}
