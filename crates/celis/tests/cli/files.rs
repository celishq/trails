//! CIP-08, "Files".

use crate::support;

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

/// `done --file` keeps the file as a blob of the store and names it on the done by an `imeta` and its
/// Blossom URI; the review gives the blob, local only with no link (`spec/08-files.md`, "Files of a done";
/// "Blobs").
#[test]
fn done_with_a_file_keeps_it_as_a_blob() {
    let repo = Repo::new();
    repo.write("art/logo.svg", "<svg/>\n");
    let ask = add(&repo, &["--", "the ask"]);
    let step = add(&repo, &["--under", &ask[..8], "--", "the logo"]);
    let said = done_with(&repo, &step, "art/logo.svg");
    let sha = sha_of("<svg/>\n");
    let blob = repo.home().join("trails/blobs").join(&sha);
    assert_eq!(std::fs::read_to_string(&blob).unwrap(), "<svg/>\n");
    let shared = repo.home().join("trails/shared");
    let done: celis_nostr::Note = std::fs::read_dir(shared)
        .unwrap()
        .filter_map(|e| {
            serde_json::from_str(&std::fs::read_to_string(e.unwrap().path()).ok()?).ok()
        })
        .find(|n: &celis_nostr::Note| n.kind == 1985 && n.tags.iter().any(|t| t[0] == "imeta"))
        .unwrap();
    let url = format!(
        "blossom:{sha}.svg?as={}&sz=7",
        support::account().pubkey().hex()
    );
    assert!(done.tags.contains(&vec![
        "imeta".to_string(),
        format!("url {url}"),
        format!("x {sha}"),
        "m image/svg+xml".to_string(),
        "size 7".to_string(),
        "path art/logo.svg".to_string(),
    ]));
    assert_eq!(done.content, format!("the logo\n{url}"));
    assert!(
        said.contains(&format!(
            "  file    art/logo.svg · {} · local only",
            blob.display()
        )),
        "{said}"
    );
}

/// The first bytes of a PNG, as a file of a proof gives them on stdin.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n a picture";

/// The `imeta` tag of the newest done in the store.
fn imeta(repo: &Repo) -> Vec<String> {
    let shared = repo.home().join("trails/shared");
    let done: celis_nostr::Note = std::fs::read_dir(shared)
        .unwrap()
        .filter_map(|e| {
            serde_json::from_str(&std::fs::read_to_string(e.unwrap().path()).ok()?).ok()
        })
        .find(|n: &celis_nostr::Note| n.kind == 1985 && n.tags.iter().any(|t| t[0] == "imeta"))
        .unwrap();
    done.tags.into_iter().find(|t| t[0] == "imeta").unwrap()
}

/// `done --file -` keeps the bytes of stdin as a blob, with the media type of its first bytes and no
/// path (`spec/08-files.md`, "Files of a done").
#[test]
fn done_with_stdin_keeps_its_bytes_as_a_blob() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the logo"]);
    let args = [
        "step",
        "done",
        &trail[..8],
        "--as-planned",
        "--file",
        "-",
        "--",
        "p",
    ];
    let o = repo.run(&repo.path(), &args, &[], PNG);
    assert!(o.status.success(), "{}", err(&o));
    let sha = sha_of_bytes(PNG);
    assert_eq!(
        std::fs::read(repo.home().join("trails/blobs").join(&sha)).unwrap(),
        PNG
    );
    let pubkey = support::account().pubkey().hex();
    assert_eq!(
        imeta(&repo),
        vec![
            "imeta".to_string(),
            format!("url blossom:{sha}.png?as={pubkey}&sz={}", PNG.len()),
            format!("x {sha}"),
            "m image/png".to_string(),
            format!("size {}", PNG.len()),
        ]
    );
}

/// `done --file data:...;base64,...` keeps the decoded bytes as a blob, with the media type the URI
/// gives (`spec/08-files.md`, "Files of a done").
#[test]
fn done_with_a_data_uri_keeps_its_bytes_as_a_blob() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the logo"]);
    let uri = "data:image/svg+xml;base64,PHN2Zy8+Cg==";
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &trail[..8],
            "--as-planned",
            "--file",
            uri,
            "--",
            "p",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let sha = sha_of("<svg/>\n");
    assert!(repo.home().join("trails/blobs").join(&sha).is_file());
    let tag = imeta(&repo);
    assert!(tag.contains(&"m image/svg+xml".to_string()), "{tag:?}");
    assert!(
        tag[1].starts_with(&format!("url blossom:{sha}.svg?")),
        "{tag:?}"
    );
}

/// `done --file` refuses empty stdin, a `data:` URI with no `;base64`, and base64 that does not decode,
/// as usage errors (`spec/02-cli.md`, "Commands").
#[test]
fn done_with_no_bytes_is_a_usage_error() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the logo"]);
    for file in ["-", "data:text/plain,hello", "data:image/png;base64,***"] {
        let args = [
            "step",
            "done",
            &trail[..8],
            "--as-planned",
            "--file",
            file,
            "--",
            "p",
        ];
        let o = cli(&repo, &args);
        assert_eq!(o.status.code(), Some(2), "{file}: {}", err(&o));
    }
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
    let ask = add(&repo, &["--", "the ask"]);
    let step = add(&repo, &["--under", &ask[..8], "--", "the logo"]);
    let said = done_with(&repo, &step, "logo.svg");
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
    sha_of_bytes(text.as_bytes())
}

/// The sha256 of `bytes`, in lower-case hex.
fn sha_of_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
