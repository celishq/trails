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
            "--by",
            "person",
            "--file",
            rel,
            "--",
            proof,
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
}

/// `pattern --good --bad` writes the whole ids of the two steps as `good` and `bad`
/// (`spec/05-patterns.md`, "Examples").
#[test]
fn pattern_names_the_good_and_the_bad_step() {
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
            "pattern",
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
    let pattern = support::labelled(&repo, "pattern")
        .into_iter()
        .find(|n| n.content.contains("sharp corners"))
        .unwrap();
    assert_eq!(pattern.tag("good"), Some(b.as_str()));
    assert_eq!(pattern.tag("bad"), Some(a.as_str()));
    let patterns = cli(
        &repo,
        &["search", "logo.svg", "--kind", "pattern", "--", "x"],
    );
    let said = out(&patterns);
    let blob = |text: &str| {
        use sha2::{Digest, Sha256};
        repo.home()
            .join("trails/blobs")
            .join(format!("{:x}", Sha256::digest(text.as_bytes())))
            .display()
            .to_string()
    };
    let lines: Vec<&str> = said.lines().skip(2).collect();
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
fn start_prints_the_pair_of_a_pattern_its_ask_uses() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the logo"]);
    let a = add(&repo, &["--under", &trail[..8], "--", "a sharp logo"]);
    repo.write("logo.svg", "<svg/>\n");
    done(&repo, &a, "logo.svg", "sharp");
    let o = cli(
        &repo,
        &[
            "pattern",
            "--good",
            &a[..8],
            "--",
            "When a logo is drawn: keep sharp corners, because the person picked them.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let pattern = out(&o).lines().next().unwrap().to_string();
    let ask = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &pattern[..8],
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

/// `pattern --good` refuses a step that the store does not hold, with `no-step`.
#[test]
fn pattern_refuses_an_unknown_step() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &[
            "pattern",
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

/// `pattern` takes `--good` and `--bad` again and again, as a file it keeps as a blob or a URL, and
/// `search` prints each (`spec/05-patterns.md`, "Examples").
#[test]
fn a_pattern_keeps_files_and_urls_as_examples() {
    let repo = Repo::new();
    repo.write("a.svg", "<svg id=\"a\"/>\n");
    repo.write("b.svg", "<svg id=\"b\"/>\n");
    let url = "https://example.com/round-logo.png";
    let o = cli(
        &repo,
        &[
            "pattern",
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
    let pattern = support::labelled(&repo, "pattern").remove(0);
    let values: Vec<&Vec<String>> = pattern
        .tags
        .iter()
        .filter(|t| t[0] == "good" || t[0] == "bad")
        .collect();
    assert_eq!(values.len(), 3, "{values:?}");
    assert!(values[0][1].starts_with("blossom:"), "{values:?}");
    assert_eq!(values[0][2], "a.svg");
    assert_eq!(values[2][1], url);
    let o = cli(&repo, &["search", "--", "draw a logo"]);
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

/// A pattern that replaces another keeps the examples of the one it replaces (`spec/05-patterns.md`,
/// "Examples").
#[test]
fn a_replacing_pattern_keeps_the_old_examples() {
    let repo = Repo::new();
    let first = "https://example.com/one.png";
    let second = "https://example.com/two.png";
    let o = cli(
        &repo,
        &[
            "pattern",
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
            "pattern",
            "--replaces",
            &old[..8],
            "--good",
            second,
            "--",
            "When a logo is drawn: keep it flat and sharp.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let pattern = support::labelled(&repo, "pattern")
        .into_iter()
        .find(|n| n.content.contains("sharp"))
        .unwrap();
    let good: Vec<&str> = pattern
        .tags
        .iter()
        .filter(|t| t[0] == "good")
        .map(|t| t[1].as_str())
        .collect();
    assert_eq!(good, [second, first]);
}

/// `patterns` lists each live pattern in scope with its examples, the rules first, newest first within
/// each (`spec/02-cli.md`, "Patterns").
#[test]
fn patterns_lists_each_pattern_with_its_examples() {
    let repo = Repo::new();
    let url = "https://example.com/flat.png";
    let decide = |args: &[&str]| {
        let o = cli(&repo, args);
        assert!(o.status.success(), "{}", err(&o));
        out(&o).lines().next().unwrap().to_string()
    };
    let rule = decide(&["pattern", "--binds", "--", "When you test: set A=1."]);
    let old = decide(&[
        "pattern",
        "--good",
        url,
        "--",
        "When a logo is drawn: keep it flat.",
    ]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let new = decide(&[
        "pattern",
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
            format!("{} project When you test: set A=1.", &rule[..8]),
            format!("{} project When an icon is drawn: keep it bold.", &new[..8]),
            format!("  bad     {url}"),
            format!("{} project When a logo is drawn: keep it flat.", &old[..8]),
            format!("  good    {url}"),
        ],
        "{said}"
    );
}

/// `patterns` says when the store holds none.
#[test]
fn patterns_says_when_there_is_none() {
    let repo = Repo::new();
    assert_eq!(out(&cli(&repo, &["patterns"])).trim(), "no pattern yet");
}

/// `tree --full` names each pattern a step uses, and `stats` counts each pattern's examples and how often
/// `search` brought it.
#[test]
fn tree_and_stats_show_the_patterns() {
    let repo = Repo::new();
    let url = "https://example.com/flat.png";
    let o = cli(
        &repo,
        &[
            "pattern",
            "--path",
            "*.svg",
            "--good",
            url,
            "--bad",
            url,
            "--",
            "When a logo is drawn: keep it flat.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let pattern = out(&o).lines().next().unwrap().to_string();
    for _ in 0..2 {
        assert!(cli(&repo, &["search", "logo.svg", "--", "draw a logo"])
            .status
            .success());
    }
    let trail = add(&repo, &["--", "the logo"]);
    add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &pattern[..8],
            "--",
            "draw it flat",
        ],
    );
    let tree = out(&cli(&repo, &["tree", &trail[..8], "--full"]));
    assert!(
        tree.contains(&format!(
            "pattern  {} When a logo is drawn: keep it flat.",
            &pattern[..8]
        )),
        "{tree}"
    );
    let stats = out(&cli(&repo, &["stats"]));
    assert!(
        stats.contains(&format!(
            "pattern {} When a logo is drawn: keep it flat. · 2 examples · recalled 2",
            &pattern[..8]
        )),
        "{stats}"
    );
}

/// A linked folder carries a pattern and the blob of each file example to a second store, and keeps a
/// `me` pattern out (`spec/08-files.md`, "Blobs").
#[test]
fn a_link_carries_a_pattern_and_its_files() {
    let repo = Repo::new();
    repo.write("logo.svg", "<svg id=\"flat\"/>\n");
    let decide = |scope: &str, text: &str| {
        let o = cli(
            &repo,
            &[
                "pattern", "--scope", scope, "--good", "logo.svg", "--", text,
            ],
        );
        assert!(o.status.success(), "{}", err(&o));
    };
    decide("project", "When a logo is drawn: keep it flat.");
    decide("me", "When an icon is drawn: keep it bold.");
    assert!(cli(&repo, &["link"]).status.success());
    let sha = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(b"<svg id=\"flat\"/>\n"))
    };
    assert!(repo.path().join(".celis/trails/blobs").join(&sha).is_file());
    let other = tempfile::tempdir().unwrap();
    let home = other.path().display().to_string();
    let env = [("CELIS_HOME", home.as_str())];
    assert!(repo.run(&repo.path(), &["link"], &env, "").status.success());
    let said = out(&repo.run(&repo.path(), &["patterns"], &env, ""));
    assert!(said.contains("keep it flat"), "{said}");
    assert!(!said.contains("keep it bold"), "{said}");
    let blob = other.path().join("trails/blobs").join(&sha);
    assert!(said.contains(&blob.display().to_string()), "{said}");
}

/// `patterns --markdown` prints one page: a heading for each pattern, then a line for each example,
/// a file as a link to its blob and a URL as a link.
#[test]
fn patterns_print_as_markdown() {
    let repo = Repo::new();
    repo.write("logo.svg", "<svg id=\"flat\"/>\n");
    let url = "https://example.com/round.png";
    let o = cli(
        &repo,
        &[
            "pattern",
            "--good",
            "logo.svg",
            "--bad",
            url,
            "--",
            "When a logo is drawn: keep it flat.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).lines().next().unwrap().to_string();
    let blob = {
        use sha2::{Digest, Sha256};
        let sha = format!("{:x}", Sha256::digest(b"<svg id=\"flat\"/>\n"));
        repo.home()
            .join("trails/blobs")
            .join(sha)
            .display()
            .to_string()
    };
    let said = out(&cli(&repo, &["patterns", "--markdown"]));
    assert_eq!(
        said.lines().collect::<Vec<_>>(),
        [
            "# Patterns".to_string(),
            String::new(),
            "## When a logo is drawn: keep it flat.".to_string(),
            String::new(),
            format!("`{}` · project", &id[..8]),
            String::new(),
            format!("- good: [logo.svg]({blob})"),
            format!("- bad: <{url}>"),
        ],
        "{said}"
    );
}

/// The forms of `spec/fixtures/pattern-labels.json`.
fn forms() -> Vec<serde_json::Value> {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/fixtures/pattern-labels.json")).unwrap();
    fixture["forms"].as_array().unwrap().clone()
}

/// A new pattern carries the labels in `celis.pattern` (`spec/05-patterns.md`, "Patterns").
#[test]
fn a_new_rule_carries_the_labels_in_celis_pattern() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &["pattern", "--binds", "--", "When you test: set A=1."],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rule = support::labelled(&repo, "pattern").remove(0);
    let labels: Vec<Vec<String>> = rule
        .tags
        .iter()
        .filter(|t| t[0] == "L" || t[0] == "l")
        .cloned()
        .collect();
    let written = forms().into_iter().find(|f| f["written"] == true).unwrap();
    let want: Vec<Vec<String>> = serde_json::from_value(written["labels"].clone()).unwrap();
    assert_eq!(labels, want);
}

/// A record in each form of the fixture reads as a pattern, and as a rule with `binds`, in
/// `celis.decision` too (`spec/05-patterns.md`, "Reading").
#[test]
fn each_form_of_the_labels_reads_as_a_pattern_and_a_rule() {
    let repo = Repo::new();
    let by = support::account();
    let mut want = Vec::new();
    for (n, form) in forms().iter().enumerate() {
        let mut tags = support::pattern_tags((&repo.project_id(), "repo"));
        tags.truncate(5);
        tags.extend(serde_json::from_value::<Vec<Vec<String>>>(form["labels"].clone()).unwrap());
        let content = format!("When {}", form["name"].as_str().unwrap());
        let note = celis_nostr::Note::write(&by, 1111u16, 1_700_000_000 + n as u64, tags, content)
            .unwrap();
        support::put(&repo, &note);
        let head = match form["binds"] == true {
            true => format!("`{}` · project · rule", &note.id[..8]),
            false => format!("`{}` · project", &note.id[..8]),
        };
        want.push(head);
    }
    let said = out(&cli(&repo, &["patterns", "--markdown"]));
    for head in &want {
        assert!(said.lines().any(|l| l == head), "{head}\n{said}");
    }
}

/// `pattern` prints each live pattern whose "When" shares words with the new one's, and names
/// `--replaces` as the next move; it prints none for an unlike pattern.
#[test]
fn pattern_prints_the_live_patterns_like_it() {
    let repo = Repo::new();
    let old = cli(
        &repo,
        &[
            "pattern",
            "--",
            "When a release tags a version: push the tag last.",
        ],
    );
    let old = out(&old).lines().next().unwrap().to_string();
    let like = cli(
        &repo,
        &[
            "pattern",
            "--",
            "When a release tags its version: sign the tag.",
        ],
    );
    assert!(like.status.success(), "{}", err(&like));
    let line = format!(
        "like    {} When a release tags a version: push the tag last.",
        &old[..8]
    );
    assert!(out(&like).lines().any(|l| l == line), "{}", out(&like));
    assert!(
        err(&like).contains(&format!("celis pattern --replaces {}", &old[..8])),
        "{}",
        err(&like)
    );
    let unlike = cli(
        &repo,
        &["pattern", "--", "When a logo is drawn: keep sharp corners."],
    );
    assert!(unlike.status.success(), "{}", err(&unlike));
    assert!(!out(&unlike).contains("like "), "{}", out(&unlike));
}
