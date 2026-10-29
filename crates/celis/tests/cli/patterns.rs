//! CIP-05, "Patterns".

use crate::support;

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
    let id = out(&o).lines().next().unwrap().to_string();
    let said = out(&cli(&repo, &["grep", &id[..8]]));
    let lines: Vec<&str> = said.lines().skip(2).take(2).collect();
    assert_eq!(
        lines,
        [
            format!(
                "  [repo] good {} {} a sharp logo · sharp",
                &id[..8],
                &b[..8]
            ),
            format!("  [repo] bad {} {} a round logo · round", &id[..8], &a[..8]),
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

/// `pattern --good -` keeps the bytes of stdin, and `--bad data:...;base64,...` the decoded bytes, each
/// as a blob named by its Blossom URI with no path (`spec/05-patterns.md`, "Examples").
#[test]
fn a_pattern_keeps_stdin_and_data_uris_as_examples() {
    let repo = Repo::new();
    let png: &[u8] = b"\x89PNG\r\n\x1a\n a picture";
    let args = [
        "pattern",
        "--good",
        "-",
        "--bad",
        "data:image/svg+xml;base64,PHN2Zy8+Cg==",
        "--",
        "When a logo is drawn: keep sharp corners, because the person picked them.",
    ];
    let o = repo.run(&repo.path(), &args, &[], png);
    assert!(o.status.success(), "{}", err(&o));
    let pattern = support::labelled(&repo, "pattern").remove(0);
    let sha = |bytes: &[u8]| {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(bytes))
    };
    let tag = |side: &str| pattern.tags.iter().find(|t| t[0] == side).unwrap().clone();
    let (good, bad) = (tag("good"), tag("bad"));
    assert_eq!(good.len(), 2, "{good:?}");
    assert!(
        good[1].starts_with(&format!("blossom:{}.png?", sha(png))),
        "{good:?}"
    );
    assert!(
        bad[1].starts_with(&format!("blossom:{}.svg?", sha(b"<svg/>\n"))),
        "{bad:?}"
    );
    let blobs = repo.home().join("trails/blobs");
    assert_eq!(
        std::fs::read(support::blob(&blobs, &sha(png))).unwrap(),
        png
    );
    assert!(support::blob(&blobs, &sha(b"<svg/>\n")).is_file());
}

/// `pattern` takes `--good` and `--bad` again and again, as a file it keeps as a blob or a URL, and
/// `grep` prints each (`spec/05-patterns.md`, "Examples").
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
    let said = out(&cli(&repo, &["grep", &pattern.id[..8]]));
    let blob = |text: &str| {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(text.as_bytes()))
    };
    assert!(said.contains(&blob("<svg id=\"a\"/>\n")), "{said}");
    assert!(said.contains(&blob("<svg id=\"b\"/>\n")), "{said}");
    assert!(
        said.contains(&format!(" bad {} {url}", &pattern.id[..8])),
        "{said}"
    );
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

/// `patterns` says when the store holds none.
#[test]
fn patterns_says_when_there_is_none() {
    let repo = Repo::new();
    assert_eq!(out(&cli(&repo, &["patterns"])).trim(), "no pattern yet");
}

/// `tree --full` prints each pattern a step uses as a `§` line under it, and `stats` counts each pattern's
/// uses and examples.
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
    let trail = add(&repo, &["--", "the logo"]);
    let step = add(
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
    for next in [false, true] {
        let mut args = vec!["tree", &trail[..8], "--full"];
        if next {
            args.push("--next");
        }
        let tree = out(&cli(&repo, &args));
        let shown = format!(
            "  ○   {} draw it flat\n    §   {} When a logo is drawn: keep it flat.\n",
            &step[..8],
            &pattern[..8]
        );
        assert!(tree.contains(&shown), "{tree}");
        assert!(!tree.contains("pattern  "), "{tree}");
    }
    let stats = out(&cli(&repo, &["stats"]));
    let line = stats
        .lines()
        .find(|l| l.starts_with(&pattern[..8]))
        .unwrap();
    assert!(
        line.starts_with(&format!("{} uses:1 ", &pattern[..8])),
        "{stats}"
    );
    assert!(line.contains(" examples:2 "), "{stats}");
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
    assert!(support::blob(&repo.path().join(".celis/trails/blobs"), &sha).is_file());
    let other = tempfile::tempdir().unwrap();
    let home = other.path().display().to_string();
    let env = [("CELIS_HOME", home.as_str())];
    assert!(repo.run(&repo.path(), &["link"], &env, "").status.success());
    let said = out(&repo.run(&repo.path(), &["patterns", "--markdown"], &env, ""));
    assert!(said.contains("keep it flat"), "{said}");
    assert!(!said.contains("keep it bold"), "{said}");
    let blob = support::blob(&other.path().join("trails/blobs"), &sha);
    assert!(said.contains(&blob.display().to_string()), "{said}");
}

/// `patterns --markdown` prints one page: a heading for each pattern, then a line for each example, an
/// image as a picture of its blob, and a URL as a link (`spec/05-patterns.md`, "Examples").
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
        support::blob(&repo.home().join("trails/blobs"), &sha)
    };
    assert_eq!(blob.extension().unwrap(), "svg");
    let blob = blob.display().to_string();
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
            format!("- good: ![logo.svg]({blob})"),
            format!("- bad: <{url}>"),
        ],
        "{said}"
    );
}

/// Off a terminal, `patterns <id>` prints the text of a pattern with an image example and no escape
/// sequence, so a pipe reads the text alone.
#[test]
fn patterns_with_an_id_draw_no_image_off_a_terminal() {
    let repo = Repo::new();
    repo.write("logo.png", "png bytes\n");
    let o = cli(
        &repo,
        &[
            "pattern",
            "--good",
            "logo.png",
            "--",
            "When a logo is drawn: keep it flat.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).lines().next().unwrap().to_string();
    let said = out(&cli(&repo, &["patterns", &id[..8]]));
    assert!(said.contains("When a logo is drawn"), "{said}");
    assert!(!said.contains('\x1b'), "{said}");
}

/// `patterns <id>` prints that pattern whole, as `--markdown` does, and refuses an id that names no live
/// pattern with `no-pattern`; a word that names a file stays a file (`spec/02-cli.md`, "Patterns").
#[test]
fn patterns_with_an_id_print_that_pattern_whole() {
    let repo = Repo::new();
    let decide = |text: &str| {
        let o = cli(&repo, &["pattern", "--", text]);
        assert!(o.status.success(), "{}", err(&o));
        out(&o).lines().next().unwrap().to_string()
    };
    let id = decide("When a logo is drawn\nDo: keep it flat.");
    decide("When a price rounds: round down.");
    let said = out(&cli(&repo, &["patterns", &id[..8]]));
    assert_eq!(
        said.lines().collect::<Vec<_>>(),
        [
            "## When a logo is drawn".to_string(),
            String::new(),
            format!("`{}` · project", &id[..8]),
            String::new(),
            "Do: keep it flat.  ".to_string(),
        ],
        "{said}"
    );
    support::refusal(
        &cli(&repo, &["patterns", "deadbeef", "--json"]),
        1,
        "no-pattern",
    );
    repo.write("cafe", "");
    let index = out(&cli(&repo, &["patterns", "cafe"]));
    assert!(index.contains("When a price rounds"), "{index}");
}

/// The forms of `spec/fixtures/pattern-labels.json`.
fn forms() -> Vec<serde_json::Value> {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../spec/fixtures/pattern-labels.json"
    ))
    .unwrap();
    fixture["forms"].as_array().unwrap().clone()
}

/// A new pattern carries the labels in `celis.pattern`, and no `binds` label with `--binds`
/// (`spec/05-patterns.md`, "Patterns").
#[test]
fn a_new_pattern_carries_the_labels_in_celis_pattern() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &["pattern", "--binds", "--", "When you test: set A=1."],
    );
    assert!(o.status.success(), "{}", err(&o));
    let pattern = support::labelled(&repo, "pattern").remove(0);
    let labels: Vec<Vec<String>> = pattern
        .tags
        .iter()
        .filter(|t| t[0] == "L" || t[0] == "l")
        .cloned()
        .collect();
    let written = forms().into_iter().find(|f| f["written"] == true).unwrap();
    let want: Vec<Vec<String>> = serde_json::from_value(written["labels"].clone()).unwrap();
    assert_eq!(labels, want);
}

/// A record in each form of the fixture reads as a plain pattern, and the `grep` view marks it
/// `old-rule` when it has `binds` (`spec/05-patterns.md`, "Reading").
#[test]
fn each_form_of_the_labels_reads_as_a_plain_pattern() {
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
        want.push((note.id[..8].to_string(), form["old_rule"] == true));
    }
    let said = out(&cli(&repo, &["patterns", "--markdown"]));
    let grep = out(&cli(&repo, &["grep", " pattern "]));
    for (id, old_rule) in &want {
        let head = format!("`{id}` · project");
        assert!(said.lines().any(|l| l == head), "{head}\n{said}");
        let line = grep.lines().find(|l| l.contains(id.as_str())).unwrap();
        assert_eq!(line.contains(" old-rule"), *old_rule, "{line}");
    }
}

/// `pattern` warns for each example a live pattern marks the other way, a file by its content; it
/// does not warn for one marked the same way.
#[test]
fn pattern_warns_of_an_example_a_live_pattern_marks_the_other_way() {
    let repo = Repo::new();
    repo.write("1.svg", "<svg id=\"green\"/>\n");
    let old = cli(
        &repo,
        &[
            "pattern",
            "--good",
            "1.svg",
            "--",
            "When a ground is chosen: use green.",
        ],
    );
    let old = out(&old).lines().next().unwrap().to_string();
    repo.write("copy.svg", "<svg id=\"green\"/>\n");
    let other = cli(
        &repo,
        &[
            "pattern",
            "--bad",
            "copy.svg",
            "--",
            "When a photo is placed: grade it.",
        ],
    );
    assert!(other.status.success(), "{}", err(&other));
    let line = format!(
        "warn    contradicts: {} marks copy.svg good; say in the Forces which part of it each pattern is about, or give another example",
        &old[..8]
    );
    assert!(err(&other).lines().any(|l| l == line), "{}", err(&other));
    repo.write("logo.svg", "<svg id=\"flat\"/>\n");
    cli(
        &repo,
        &[
            "pattern",
            "--good",
            "logo.svg",
            "--",
            "When a logo is drawn: keep it flat.",
        ],
    );
    let same = cli(
        &repo,
        &[
            "pattern",
            "--good",
            "logo.svg",
            "--",
            "When an icon is drawn: keep it flat.",
        ],
    );
    assert!(!err(&same).contains("contradicts"), "{}", err(&same));
}

/// `pattern` prints each live pattern whose "When" shares words with the new one's, and names
/// `--replaces` and a narrower When as the next moves; it prints none for an unlike pattern.
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
    assert!(
        err(&like).contains("narrow the When of one"),
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
