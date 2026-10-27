//! CIP-09, "Export": `celis cell export`.

use crate::support;

use celis_nostr::Note;
use support::cell::{decide, membership, records, theirs};
use support::{account, add, cli, err, finish, now, out, refusal, stored_pattern, Repo};

/// `cell export` writes the cell's live patterns and one new membership that names them to
/// `cell.jsonl`, the blob of each file example to `blobs/`, and `cell.md` with its links and credits,
/// and writes no record (`spec/09-cells.md`, "Export").
#[test]
fn cell_export_writes_the_records_the_blobs_and_a_page_and_no_record() {
    use sha2::Digest;
    let repo = Repo::new();
    repo.write("plate.png", "a plate");
    let a = decide(
        &repo,
        "Name: Soups first\nWhen a menu is laid out\nDo: put soups first",
    );
    let step = add(&repo, &["--", "Lay out the menu"]);
    finish(&repo, &step);
    let o = cli(
        &repo,
        &[
            "pattern",
            "--good",
            "plate.png",
            "--bad",
            "https://example.com/menu",
            "--good",
            &step,
            "--",
            "Name: Hot dishes\nWhen a dish is served\nDo: serve it hot",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let b = out(&o).trim().to_string();
    assert!(cli(&repo, &["cell", "add", "east", &a, &b])
        .status
        .success());
    let before = records(&repo);
    let o = cli(&repo, &["cell", "export", "east", "--to", "shared/east"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(records(&repo), before);
    let folder = repo.path().join("shared/east");
    let lines = std::fs::read_to_string(folder.join("cell.jsonl")).unwrap();
    let notes: Vec<Note> = lines
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(notes.len(), 3);
    assert!(notes.iter().all(Note::verifies));
    let mut exported = [&notes[0].id, &notes[1].id];
    exported.sort();
    let mut made = [&a, &b];
    made.sort();
    assert_eq!(exported, made);
    let membership = &notes[2];
    assert_eq!(membership.tag("L"), Some("celis.cell"));
    let named: Vec<&str> = membership
        .tags
        .iter()
        .filter(|t| t[0] == "e")
        .map(|t| t[1].as_str())
        .collect();
    assert_eq!(named, [notes[0].id.as_str(), notes[1].id.as_str()]);
    let sha = format!("{:x}", sha2::Sha256::digest(b"a plate"));
    let blob = format!("blobs/{sha}.png");
    assert_eq!(std::fs::read(folder.join(&blob)).unwrap(), b"a plate");
    let page = std::fs::read_to_string(folder.join("cell.md")).unwrap();
    for part in [
        "# east",
        "## Soups first",
        "## Hot dishes",
        &format!("- good: ![plate.png]({blob})"),
        "- bad: <https://example.com/menu>",
        "https://celis.ai",
        &account().pubkey().npub(),
    ] {
        assert!(page.contains(part), "{part}:\n{page}");
    }
    assert!(!page.contains(&step[..8]), "{page}");
}

/// `cell export` refuses a cell of another account and a name no cell holds with `no-cell`
/// (`spec/02-cli.md`, "Cells").
#[test]
fn cell_export_refuses_a_cell_not_yours() {
    let repo = Repo::new();
    let other = stored_pattern(
        &repo,
        (&repo.project_id(), "repo"),
        &theirs(),
        (None, None),
        "When rice is cooked\nDo: rinse it",
        now(),
    );
    membership(&repo, &theirs(), "west", &[&other.id]);
    for name in ["west", "east"] {
        let o = cli(&repo, &["cell", "export", name, "--json"]);
        refusal(&o, 1, "no-cell");
    }
    assert!(!repo.path().join("west").exists());
}
