//! CIP-05, "Pairs".

mod support;

use support::{add, cli, err, out, Repo};

/// Closes `id` with `proof` and the file `rel`, which must succeed.
fn done(repo: &Repo, id: &str, rel: &str, proof: &str) {
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
            proof,
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
}

/// `decide --good --bad` writes the whole ids of the two steps as `good` and `bad`
/// (`spec/05-rules.md`, "Pairs").
#[test]
fn decide_names_the_good_and_the_bad_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the logo"]);
    let a = add(&repo, &["--under", &trail[..8], "--", "a round logo"]);
    let b = add(&repo, &["--under", &trail[..8], "--", "a sharp logo"]);
    repo.write("logo.svg", "<svg id=\"a\"/>\n");
    done(&repo, &a, "logo.svg", "round");
    repo.write("logo.svg", "<svg id=\"b\"/>\n");
    done(&repo, &b, "logo.svg", "sharp");
    let o = cli(
        &repo,
        &[
            "decide",
            "--good",
            &b[..8],
            "--bad",
            &a[..8],
            "--path",
            "*.svg",
            "--",
            "When a logo is drawn: keep sharp corners, because the person picked them.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rule = support::labelled(&repo, "decision")
        .into_iter()
        .find(|n| n.content.contains("sharp corners"))
        .unwrap();
    assert_eq!(rule.tag("good"), Some(b.as_str()));
    assert_eq!(rule.tag("bad"), Some(a.as_str()));
    let rules = cli(&repo, &["rules", "logo.svg"]);
    let said = out(&rules);
    let blob = |text: &str| {
        use sha2::{Digest, Sha256};
        repo.home()
            .join("trails/blobs")
            .join(format!("{:x}", Sha256::digest(text.as_bytes())))
            .display()
            .to_string()
    };
    let lines: Vec<&str> = said.lines().skip(1).collect();
    assert_eq!(
        lines,
        [
            format!("  good    {} a sharp logo · sharp", &b[..8]),
            format!("          {}", blob("<svg id=\"b\"/>\n")),
            format!("  bad     {} a round logo · round", &a[..8]),
            format!("          {}", blob("<svg id=\"a\"/>\n")),
        ],
        "{said}"
    );
}

/// `decide --good` refuses a step that the store does not hold, with `no-step`.
#[test]
fn decide_refuses_an_unknown_step() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &[
            "decide",
            "--good",
            "0000aaaa",
            "--",
            "When x: y, because z.",
        ],
    );
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(
        err(&o).contains("no step has the id 0000aaaa"),
        "{}",
        err(&o)
    );
}
