//! CIP-05, "Patterns".

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
/// (`spec/05-rules.md`, "Patterns").
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

/// `start` prints a pattern its ask uses with its good step.
#[test]
fn start_prints_the_pair_of_a_rule_its_ask_uses() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the logo"]);
    let a = add(&repo, &["--under", &trail[..8], "--", "a sharp logo"]);
    repo.write("logo.svg", "<svg/>\n");
    done(&repo, &a, "logo.svg", "sharp");
    let o = cli(
        &repo,
        &[
            "decide",
            "--good",
            &a[..8],
            "--",
            "When a logo is drawn: keep sharp corners, because the person picked them.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rule = out(&o).lines().next().unwrap().to_string();
    let ask = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &rule[..8],
            "--",
            "the icon",
        ],
    );
    let said = out(&cli(&repo, &["step", "start", &ask[..8]]));
    assert!(
        said.contains(&format!("  good    {} a sharp logo · sharp", &a[..8])),
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

/// `decide` takes `--good` and `--bad` again and again, as a file it keeps as a blob or a URL, and
/// `recall` prints each (`spec/05-rules.md`, "Patterns").
#[test]
fn a_pattern_keeps_files_and_urls_as_examples() {
    let repo = Repo::new();
    repo.write("a.svg", "<svg id=\"a\"/>\n");
    repo.write("b.svg", "<svg id=\"b\"/>\n");
    let url = "https://example.com/round-logo.png";
    let o = cli(
        &repo,
        &[
            "decide",
            "--path",
            "*.svg",
            "--good",
            "a.svg",
            "--good",
            "b.svg",
            "--bad",
            url,
            "--",
            "When a logo is drawn: keep sharp corners, because the person picked them.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rule = support::labelled(&repo, "decision").remove(0);
    let values: Vec<&Vec<String>> = rule
        .tags
        .iter()
        .filter(|t| t[0] == "good" || t[0] == "bad")
        .collect();
    assert_eq!(values.len(), 3, "{values:?}");
    assert!(values[0][1].starts_with("blossom:"), "{values:?}");
    assert_eq!(values[0][2], "a.svg");
    assert_eq!(values[2][1], url);
    let o = cli(&repo, &["recall", "--", "draw a logo"]);
    let said = out(&o);
    let blob = |text: &str| {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(text.as_bytes()))
    };
    assert!(said.contains(&blob("<svg id=\"a\"/>\n")), "{said}");
    assert!(said.contains(&blob("<svg id=\"b\"/>\n")), "{said}");
    assert!(said.contains(&format!("  bad     {url}")), "{said}");
    assert!(!said.contains("missing"), "{said}");
}

/// A pattern that replaces another keeps the examples of the one it replaces (`spec/05-rules.md`,
/// "Patterns").
#[test]
fn a_replacing_pattern_keeps_the_old_examples() {
    let repo = Repo::new();
    let first = "https://example.com/one.png";
    let second = "https://example.com/two.png";
    let o = cli(
        &repo,
        &[
            "decide",
            "--good",
            first,
            "--",
            "When a logo is drawn: keep it flat.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let old = out(&o).lines().next().unwrap().to_string();
    let o = cli(
        &repo,
        &[
            "decide",
            "--replaces",
            &old[..8],
            "--good",
            second,
            "--",
            "When a logo is drawn: keep it flat and sharp.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rule = support::labelled(&repo, "decision")
        .into_iter()
        .find(|n| n.content.contains("sharp"))
        .unwrap();
    let good: Vec<&str> = rule
        .tags
        .iter()
        .filter(|t| t[0] == "good")
        .map(|t| t[1].as_str())
        .collect();
    assert_eq!(good, [second, first]);
}

/// `patterns` lists each live pattern in scope with its examples, newest first, and no rule without
/// examples (`spec/02-cli.md`, "Patterns").
#[test]
fn patterns_lists_each_pattern_with_its_examples() {
    let repo = Repo::new();
    let url = "https://example.com/flat.png";
    let decide = |args: &[&str]| {
        let o = cli(&repo, args);
        assert!(o.status.success(), "{}", err(&o));
        out(&o).lines().next().unwrap().to_string()
    };
    let rule = decide(&["decide", "--", "When you test: set A=1."]);
    let old = decide(&[
        "decide",
        "--good",
        url,
        "--",
        "When a logo is drawn: keep it flat.",
    ]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let new = decide(&[
        "decide",
        "--bad",
        url,
        "--",
        "When an icon is drawn: keep it bold.",
    ]);
    let said = out(&cli(&repo, &["patterns"]));
    let lines: Vec<&str> = said.lines().collect();
    assert_eq!(
        lines,
        [
            format!("{} project When an icon is drawn: keep it bold.", &new[..8]),
            format!("  bad     {url}"),
            format!("{} project When a logo is drawn: keep it flat.", &old[..8]),
            format!("  good    {url}"),
        ],
        "{said}"
    );
    assert!(!said.contains(&rule[..8]), "{said}");
}

/// `patterns` says when the store holds none.
#[test]
fn patterns_says_when_there_is_none() {
    let repo = Repo::new();
    assert_eq!(out(&cli(&repo, &["patterns"])).trim(), "no pattern yet");
}
