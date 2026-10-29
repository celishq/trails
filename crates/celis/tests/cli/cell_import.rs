//! CIP-09, "Export": `celis cell import`.

use crate::support;

use support::cell::{decide, records, serve, their_folder, theirs};
use support::{cli, err, now, out, refusal, stored_pattern, Repo};

/// `cell import` shows the cell and keeps nothing; with `--yes` it keeps the records and the blob, and
/// the cell stays off (`spec/02-cli.md`, "Cells").
#[test]
fn cell_import_shows_the_cell_and_keeps_it_off_with_yes() {
    let repo = Repo::new();
    let (_, pattern) = their_folder(&repo, "east");
    let before = records(&repo);
    let o = cli(&repo, &["cell", "import", "from-them"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        out(&o).contains(&format!("{} Rice first", &pattern.id[..8])),
        "{}",
        out(&o)
    );
    assert_eq!(records(&repo), before);
    let o = cli(&repo, &["cell", "import", "from-them", "--yes"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(records(&repo), before + 2);
    assert!(out(&cli(&repo, &["cells"]))
        .trim()
        .ends_with("1 pattern off"));
    assert!(!out(&cli(&repo, &["patterns"])).contains(&pattern.id[..8]));
    let sha = {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(b"a bowl"))
    };
    assert!(support::blob(&repo.store().join("blobs"), &sha).exists());
    assert!(cli(&repo, &["cell", "on", "east"]).status.success());
    assert!(out(&cli(&repo, &["patterns"])).contains(&pattern.id[..8]));
}

/// `cell import` refuses a folder whose record was edited, or whose pattern the membership does not
/// name, with `invalid`, and keeps nothing (`spec/09-cells.md`, "Export").
#[test]
fn cell_import_refuses_an_edited_record_and_a_pattern_out_of_the_cell() {
    let repo = Repo::new();
    let (folder, pattern) = their_folder(&repo, "east");
    let lines = std::fs::read_to_string(folder.join("cell.jsonl")).unwrap();
    let edited = lines.replace("serve the rice first", "serve the soup first");
    std::fs::write(folder.join("cell.jsonl"), &edited).unwrap();
    let before = records(&repo);
    let o = cli(&repo, &["cell", "import", "from-them", "--yes", "--json"]);
    refusal(&o, 1, "invalid");
    let stray = stored_pattern(
        &repo,
        ("q", "kitchen"),
        &theirs(),
        (None, None),
        "When soup is served\nDo: serve it hot",
        now(),
    );
    let mut lines: Vec<String> = lines.lines().map(str::to_string).collect();
    lines.insert(1, serde_json::to_string(&stray).unwrap());
    std::fs::write(folder.join("cell.jsonl"), lines.join("\n")).unwrap();
    let o = cli(&repo, &["cell", "import", "from-them", "--yes", "--json"]);
    refusal(&o, 1, "invalid");
    assert_eq!(records(&repo), before + 1);
    assert!(!out(&cli(&repo, &["cells"])).contains(&pattern.id[..8]));
}

/// An edit to `cell.md` changes nothing, and a cell whose name a cell of another account holds is
/// refused with `cell-taken` (`spec/09-cells.md`, "Export").
#[test]
fn cell_import_ignores_cell_md_and_refuses_a_taken_name() {
    let repo = Repo::new();
    let (folder, _) = their_folder(&repo, "east");
    std::fs::write(folder.join("cell.md"), "# something else\n").unwrap();
    assert!(cli(&repo, &["cell", "import", "from-them", "--yes"])
        .status
        .success());
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    assert!(cli(&repo, &["cell", "add", "east", &a]).status.success());
    their_folder(&repo, "east");
    let o = cli(&repo, &["cell", "import", "from-them", "--yes", "--json"]);
    refusal(&o, 1, "cell-taken");
}

/// `cell import <url>` fetches `cell.jsonl` and the blobs beside it, and takes a blob only when its
/// bytes hash to its name (`spec/09-cells.md`, "Export").
#[test]
fn cell_import_from_a_url_takes_only_blobs_that_match_their_hash() {
    let repo = Repo::new();
    let (folder, pattern) = their_folder(&repo, "east");
    let url = serve(folder.clone());
    let o = cli(&repo, &["cell", "import", &format!("{url}/"), "--yes"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains(&pattern.id[..8]), "{}", out(&o));
    let sha = {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(b"a bowl"))
    };
    assert!(support::blob(&repo.store().join("blobs"), &sha).exists());
    let repo = Repo::new();
    let (folder, _) = their_folder(&repo, "east");
    std::fs::write(folder.join(format!("blobs/{sha}.png")), "not a bowl").unwrap();
    let url = serve(folder);
    let o = cli(
        &repo,
        &["cell", "import", &format!("{url}/cell.jsonl"), "--yes"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&cli(&repo, &["cells"])).contains("east"));
    let blobs = std::fs::read_dir(repo.store().join("blobs")).map_or(0, Iterator::count);
    assert_eq!(blobs, 0);
}
