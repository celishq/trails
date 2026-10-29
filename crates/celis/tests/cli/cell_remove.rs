//! `celis cell remove`.

use crate::support;

use support::cell::{decide, records, their_folder};
use support::{cli, err, out, refusal, Repo};

/// `cell remove` removes the records of an imported cell and the blobs only they name, keeps a blob that
/// your pattern names too, and refuses a cell of yours and an unknown name (`spec/02-cli.md`, "Cells").
#[test]
fn cell_remove_removes_an_imported_cell_and_keeps_a_shared_blob() {
    use sha2::Digest;
    let repo = Repo::new();
    let (_, theirs) = their_folder(&repo, "east");
    assert!(cli(&repo, &["cell", "import", "from-them", "--yes"])
        .status
        .success());
    let sha = format!("{:x}", sha2::Sha256::digest(b"a bowl"));
    let blob = support::blob(&repo.store().join("blobs"), &sha);
    let before = records(&repo);
    let o = cli(&repo, &["cell", "remove", "east"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(records(&repo), before - 2);
    assert!(!blob.exists());
    assert_eq!(out(&cli(&repo, &["cells"])).trim(), "no cell yet");
    assert!(!out(&cli(&repo, &["grep", "Rice"])).contains(&theirs.id[..8]));
    assert!(cli(&repo, &["cell", "import", "from-them", "--yes"])
        .status
        .success());
    repo.write("bowl.png", "a bowl");
    let a = decide(&repo, "When a bowl is set\nDo: set it left");
    let o = cli(
        &repo,
        &[
            "pattern",
            "--good",
            "bowl.png",
            "--replaces",
            &a[..8],
            "--",
            "When a bowl is set\nDo: set it right",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(cli(&repo, &["cell", "remove", "east"]).status.success());
    assert!(blob.exists());
    let mine = out(&o).trim().to_string();
    assert!(cli(&repo, &["cell", "add", "west", &mine]).status.success());
    refusal(
        &cli(&repo, &["cell", "remove", "west", "--json"]),
        1,
        "cell-yours",
    );
    refusal(
        &cli(&repo, &["cell", "remove", "north", "--json"]),
        1,
        "no-cell",
    );
}
