//! CIP-08, "Files".

mod support;

use support::{add, cli, err, Repo};

/// The stderr of `celis step done <id> --as-planned --file <rel> -- the logo`, which must succeed.
fn done_with(repo: &Repo, id: &str, rel: &str) -> String {
    let o = cli(
        repo,
        &[
            "step",
            "done",
            &id[..8],
            "--as-planned",
            "--file",
            rel,
            "--",
            "the logo",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    err(&o)
}

/// `done --file` keeps the file as a blob of the store and names it on the done, by its place, sha256
/// and size; the review gives the blob, local only with no link (`spec/08-files.md`, "Files of a done";
/// "Blobs").
#[test]
fn done_with_a_file_keeps_it_as_a_blob() {
    let repo = Repo::new();
    repo.write("art/logo.svg", "<svg/>\n");
    let trail = add(&repo, &["--", "the trail"]);
    let said = done_with(&repo, &trail, "art/logo.svg");
    let sha = sha_of("<svg/>\n");
    let blob = repo.home().join("trails/blobs").join(&sha);
    assert_eq!(std::fs::read_to_string(&blob).unwrap(), "<svg/>\n");
    let shared = repo.home().join("trails/shared");
    let done: celis_nostr::Note = std::fs::read_dir(shared)
        .unwrap()
        .filter_map(|e| {
            serde_json::from_str(&std::fs::read_to_string(e.unwrap().path()).ok()?).ok()
        })
        .find(|n: &celis_nostr::Note| n.kind == 1985 && n.tags.iter().any(|t| t[0] == "file"))
        .unwrap();
    let tags = done.tags;
    assert!(tags.contains(&vec![
        "file".to_string(),
        "art/logo.svg".to_string(),
        sha.clone(),
        "7".to_string()
    ]));
    assert!(
        said.contains(&format!(
            "  file    art/logo.svg · {} · local only",
            blob.display()
        )),
        "{said}"
    );
}

/// `done --file` refuses a file that is not there, as a usage error.
#[test]
fn done_with_a_missing_file_is_a_usage_error() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &trail[..8],
            "--as-planned",
            "--file",
            "none.png",
            "--",
            "p",
        ],
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// While a link is on, a kept file goes to the folder's `blobs/` too, and a blob of the folder that the
/// store lacks is copied in (`spec/08-files.md`, "Blobs").
#[test]
fn a_link_carries_the_blobs_both_ways() {
    let repo = Repo::new();
    assert!(cli(&repo, &["link"]).status.success());
    repo.write("logo.svg", "<svg/>\n");
    let trail = add(&repo, &["--", "the trail"]);
    let said = done_with(&repo, &trail, "logo.svg");
    let sha = sha_of("<svg/>\n");
    let folder = repo.path().join(".celis/trails/blobs");
    assert!(folder.join(&sha).is_file());
    assert!(!said.contains("local only"), "{said}");
    repo.write("other.svg", "<svg id=\"b\"/>\n");
    let other = sha_of("<svg id=\"b\"/>\n");
    std::fs::copy(repo.path().join("other.svg"), folder.join(&other)).unwrap();
    std::fs::write(folder.join("0bad"), "not its hash").unwrap();
    assert!(cli(&repo, &["tree"]).status.success());
    let store = repo.home().join("trails/blobs");
    assert!(store.join(&other).is_file());
    assert!(!store.join("0bad").exists());
}

/// When the last step beside it closes, the review gives the blobs of the done steps beside it, so the
/// agent can show them side by side.
#[test]
fn the_last_step_beside_gives_the_blobs_of_the_others() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let layer = add(&repo, &["--under", &trail[..8], "--", "the greybox"]);
    let a = add(&repo, &["--under", &layer[..8], "--", "variant a"]);
    let b = add(&repo, &["--under", &layer[..8], "--", "variant b"]);
    repo.write("a.svg", "<svg id=\"a\"/>\n");
    done_with(&repo, &a, "a.svg");
    repo.write("a.svg", "<svg id=\"b\"/>\n");
    let said = done_with(&repo, &b, "a.svg");
    let first = repo
        .home()
        .join("trails/blobs")
        .join(sha_of("<svg id=\"a\"/>\n"));
    assert!(
        said.contains(&format!(
            "  beside  {} variant a · {}",
            &a[..8],
            first.display()
        )),
        "{said}"
    );
}

/// The sha256 of `text`, in lower-case hex.
fn sha_of(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
