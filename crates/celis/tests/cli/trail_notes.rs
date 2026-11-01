//! CIP-04, "Notes".

use crate::support;

use celis_nostr::Note;
use support::{account, add, cli, err, mark_note, now, objects, out, refusal, Repo};

/// Every note mark in the store.
fn notes(repo: &Repo) -> Vec<Note> {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .filter(|n| n.kind == 1985 && n.tag("l") == Some("note"))
        .collect()
}

/// `celis step note <id> -- <text>`, and the id it prints.
fn note(repo: &Repo, id: &str, text: &str) -> String {
    let o = cli(repo, &["step", "note", id, "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// `step note` writes the text as a note mark on the whole id of the step and prints its id; it refuses
/// empty text as a usage error and an unknown step with `no-step` (`spec/02-cli.md`, "Step";
/// `spec/03-steps.md`, "Notes").
#[test]
fn step_note_writes_a_note_on_the_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let id = note(&repo, &trail[..8], "keep the old table");
    let written = notes(&repo);
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].id, id);
    assert_eq!(written[0].kind, 1985);
    assert_eq!(written[0].tag("e"), Some(trail.as_str()));
    assert_eq!(written[0].content, "keep the old table");
    let o = cli(&repo, &["step", "note", "--json", &trail[..8], "--", " "]);
    refusal(&o, 2, "usage");
    let o = cli(&repo, &["step", "note", "--json", "ffffffff", "--", "x"]);
    refusal(&o, 1, "no-step");
    assert_eq!(notes(&repo).len(), 1);
}

/// A trail's notes are the notes on the steps of its tree, and `start` prints them; not one of another
/// trail (`spec/04-trails.md`, "Notes"; `spec/02-cli.md`, "Step").
#[test]
fn start_prints_the_notes_of_the_trail() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let other = add(&repo, &["--new", "--", "the other trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the ask"]);
    note(&repo, &ask[..8], "a note on the ask");
    let t = now() - 60;
    mark_note(&repo, &account(), "note", &trail, "an old note", t);
    mark_note(&repo, &account(), "note", &other, "theirs", t);
    let step = add(&repo, &["--under", &ask[..8], "--", "s"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    for said in ["a note on the ask", "an old note"] {
        assert!(text.contains(&format!(" note {said}")), "{text}");
    }
    assert!(!text.contains("theirs"), "{text}");
    let json = objects(&repo, &["step", "start", &step[..8]]);
    let notes: Vec<_> = json.iter().filter_map(|v| v.get("note")).collect();
    assert_eq!(notes.len(), 2, "{json:?}");
    assert!(notes
        .iter()
        .all(|n| n["id"].is_string() && n["step"].is_string()));
}
