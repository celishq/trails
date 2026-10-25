//! CIP-05, "Triggers", and CIP-02, "Search".

mod support;

use celis_nostr::Note;
use serde_json::json;
use std::path::Path;
use support::{account, add, err, objects, out, pattern_tags, put, stored_pattern, Repo, DAY};

/// The id `celis decide <args> -- <text>` prints in the main checkout.
fn decide(repo: &Repo, args: &[&str], text: &str) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    all.extend(["--", text]);
    let o = repo.run(&repo.path(), &all, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// A pattern of this repo with the extra tags `tags`, saying `content`, made at `at`, put in the store.
fn tagged(repo: &Repo, tags: &[(&str, &str)], content: &str, at: u64) -> Note {
    let mut all = pattern_tags((&repo.project_id(), "repo"));
    all.extend(tags.iter().map(|(n, v)| vec![n.to_string(), v.to_string()]));
    let note = Note::write(&account(), 1111u16, at, all, content).unwrap();
    put(repo, &note);
    note
}

/// The pattern lines `celis search <files> --kind pattern` prints in `folder`, for an ask that shares no word
/// with a pattern.
fn found(repo: &Repo, folder: &Path, files: &[&str]) -> Vec<String> {
    let mut all = vec!["search"];
    all.extend(files);
    all.extend(["--kind", "pattern", "--", "x"]);
    let o = repo.run(folder, &all, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let shown = out(&o);
    let lines = shown.lines().filter(|l| !l.starts_with(' '));
    let patterns = lines.filter(|l| *l != "patterns" && *l != "nothing fits this ask");
    patterns.map(str::to_string).collect()
}

/// The pattern lines `celis search <files> --kind pattern` prints in the main checkout.
fn patterns(repo: &Repo, files: &[&str]) -> Vec<String> {
    found(repo, &repo.path(), files)
}

/// The text of each pattern `celis search <files> --kind pattern` prints, without its id and scope.
fn texts(repo: &Repo, args: &[&str]) -> Vec<String> {
    let lines = patterns(repo, args).into_iter();
    lines
        .map(|l| l.splitn(3, ' ').nth(2).unwrap().to_string())
        .collect()
}

/// A pattern matches a file when one of its `path` globs matches the file's path, whether the file exists
/// or not (`spec/05-patterns.md`, "Triggers").
#[test]
fn a_path_glob_matches_the_file() {
    let repo = Repo::new();
    decide(&repo, &["--path", "**/*.py"], "python");
    decide(&repo, &["--path", "shelf/*.py"], "shelf");
    repo.write("shelf/a.py", "x = 1\n");
    let mut shelf = texts(&repo, &["shelf/a.py"]);
    shelf.sort();
    assert_eq!(shelf, ["python", "shelf"]);
    assert_eq!(texts(&repo, &["new.py"]), ["python"]);
    assert!(texts(&repo, &["c.rs"]).is_empty());
}

/// A folder counts as a read of each file git gives under it, ignored files left out
/// (`spec/02-cli.md`, "Search").
#[test]
fn a_folder_gives_the_patterns_of_its_files() {
    let repo = Repo::new();
    decide(&repo, &["--path", "shelf/*.py"], "floats");
    decide(&repo, &["--path", "**/*.md"], "markdown");
    decide(&repo, &["--path", "*.log"], "logs");
    repo.write("shelf/a.py", "x = float(1)\n");
    repo.write("shelf/docs/b.md", "b\n");
    repo.write("shelf/out.log", "x\n");
    repo.write(".gitignore", "*.log\n");
    let mut shelf = texts(&repo, &["shelf"]);
    shelf.sort();
    assert_eq!(shelf, ["floats", "markdown"]);
    let shown = found(&repo, &repo.path().join("shelf"), &["."]);
    let mut here: Vec<&str> = shown
        .iter()
        .map(|l| l.splitn(3, ' ').nth(2).unwrap())
        .collect();
    here.sort();
    assert_eq!(here, ["floats", "markdown"]);
}

/// A pattern from an older version with only a `tool` tag reads with no trigger, and never comes
/// (`spec/README.md`, "Names").
#[test]
fn an_older_pattern_with_a_tool_tag_has_no_trigger() {
    let repo = Repo::new();
    tagged(&repo, &[("tool", "^Grep$")], "grep", support::now() - DAY);
    let o = repo.run(&repo.path(), &["grep", " pattern "], &[], "");
    assert!(out(&o).contains("live {project} grep"), "{}", out(&o));
    assert!(patterns(&repo, &["."]).is_empty());
}

/// An older pattern `at:<place>` counts as a pattern with that place as its `path`, and holds under a folder
/// it names (`spec/05-patterns.md`, "Triggers").
#[test]
fn an_at_pattern_comes_at_its_place() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    tagged(&repo, &[("scope", "at:shelf")], "in the shelf", at);
    tagged(&repo, &[("scope", "at:a.py:3")], "at a line", at + 1);
    assert_eq!(texts(&repo, &["shelf/b.py"]), ["in the shelf"]);
    assert_eq!(texts(&repo, &["a.py"]), ["at a line"]);
    assert!(patterns(&repo, &["b.py"]).is_empty());
}

/// A pattern with no trigger and no `at:<place>` never comes by a file, and a pattern with a trigger comes at
/// its files (`spec/05-patterns.md`, "Triggers"; `spec/02-cli.md`, "Search").
#[test]
fn a_pattern_with_no_trigger_never_comes() {
    let repo = Repo::new();
    decide(&repo, &[], "no trigger");
    decide(&repo, &["--scope", "me"], "mine");
    assert!(patterns(&repo, &["README.md"]).is_empty());
    decide(&repo, &["--path", "*.md"], "both");
    assert_eq!(texts(&repo, &["README.md"]), ["both"]);
}

/// `decide --situation` is a usage error, and a reader ignores an older `situation` label
/// (`spec/02-cli.md`, "Commands"; `spec/README.md`, "Deprecated").
#[test]
fn situation_is_no_flag_and_its_label_is_ignored() {
    let repo = Repo::new();
    let o = repo.run(&repo.path(), &["decide", "--situation", "--", "t"], &[], "");
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let mut tags = pattern_tags((&repo.project_id(), "repo"));
    tags.push(
        ["l", "situation", "celis.decision"]
            .map(String::from)
            .to_vec(),
    );
    let old = Note::write(&account(), 1111u16, support::now() - DAY, tags, "older").unwrap();
    put(&repo, &old);
    let o = repo.run(&repo.path(), &["grep", " pattern "], &[], "");
    let shown = out(&o);
    let line = shown.lines().find(|l| l.contains(&old.id[..8])).unwrap();
    assert!(line.ends_with("live {project} older"), "{shown}");
}

/// Patterns come with a named file first, then a glob; newest first within each (`spec/02-cli.md`,
/// "Search").
#[test]
fn patterns_come_narrowest_first_then_newest() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    let order = [
        (&[("path", "*.py")][..], "wide old", at),
        (&[("path", "*.py")], "wide new", at + 1),
        (&[("path", "a.py")], "named old", at + 4),
        (&[("scope", "at:a.py")], "at new", at + 5),
    ];
    for (tags, content, made) in order {
        tagged(&repo, tags, content, made);
    }
    assert_eq!(
        texts(&repo, &["a.py"]),
        ["at new", "named old", "wide new", "wide old"]
    );
}

/// Every pattern that matches comes, on each call of a session (`spec/02-cli.md`, "Search").
#[test]
fn every_pattern_comes_on_each_call() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    for n in 0..7u64 {
        tagged(&repo, &[("path", "*.md")], &format!("rule {n}"), at + n);
    }
    let first = texts(&repo, &["README.md"]);
    assert_eq!(first.len(), 7, "{first:?}");
    assert_eq!((first[0].as_str(), first[6].as_str()), ("rule 6", "rule 0"));
    assert_eq!(texts(&repo, &["README.md"]), first);
}

/// The grep view marks a pattern with a trigger and a rule (`spec/02-cli.md`, "Grep").
#[test]
fn grep_marks_triggers_and_rules() {
    let repo = Repo::new();
    let trigger = decide(&repo, &["--path", "Makefile"], "trigger");
    let all = decide(&repo, &["--path", "*.md", "--binds"], "all");
    let plain = decide(&repo, &[], "plain");
    let o = repo.run(&repo.path(), &["grep", " pattern "], &[], "");
    let shown = out(&o);
    let line = |id: &str| {
        shown
            .lines()
            .find(|l| l.contains(&id[..8]))
            .unwrap()
            .to_string()
    };
    assert!(
        line(&trigger).ends_with("live {project} trigger trigger"),
        "{shown}"
    );
    assert!(
        line(&all).ends_with("live {project} trigger binds all"),
        "{shown}"
    );
    assert!(line(&plain).ends_with("live {project} plain"), "{shown}");
}

/// `decide` writes each `--path` from the project's top folder and `--binds`, and a reader reads them
/// back; `--promoted`, `--tool`, `--content` and `--command` are no flags (`spec/02-cli.md`,
/// "Commands").
#[test]
fn decide_writes_the_trigger_and_labels() {
    let repo = Repo::new();
    std::fs::create_dir_all(repo.path().join("shelf")).unwrap();
    let o = repo.run(
        &repo.path().join("shelf"),
        &[
            "decide", "--path", "*.py", "--path", "b.md", "--binds", "--", "kept",
        ],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).trim().to_string();
    let step = add(&repo, &["--new", "--uses", &id, "--", "t"]);
    let step = step.lines().next().unwrap();
    let shown = objects(&repo, &["step", "start", step]);
    let pattern = shown.iter().find_map(|v| v.get("decision")).unwrap();
    assert_eq!(pattern["paths"], json!(["shelf/*.py", "shelf/b.md"]));
    assert_eq!(pattern["binds"], json!(true));
    assert_eq!(pattern["promoted"], json!(true));
    let o = repo.run(&repo.path(), &["decide", "--promoted", "--", "t"], &[], "");
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    for flag in ["--tool", "--content", "--command"] {
        let o = repo.run(&repo.path(), &["decide", flag, "x", "--", "t"], &[], "");
        assert_eq!(o.status.code(), Some(2), "{o:?}");
    }
}

/// A pattern from before triggers still reads, with no trigger and no label, and comes at its place
/// (`spec/05-patterns.md`, "Patterns").
#[test]
fn an_older_pattern_with_no_trigger_still_reads() {
    let repo = Repo::new();
    let id = repo.project_id();
    let at = support::now() - DAY;
    let old = stored_pattern(
        &repo,
        (&id, "repo"),
        &account(),
        (Some("at:a.py"), None),
        "old",
        at,
    );
    assert_eq!(texts(&repo, &["a.py"]), ["old"]);
    let step = add(&repo, &["--new", "--uses", &old.id, "--", "t"]);
    let step = step.lines().next().unwrap();
    let shown = objects(&repo, &["step", "start", step]);
    let pattern = shown.iter().find_map(|v| v.get("decision")).unwrap();
    assert_eq!(pattern["paths"], json!([]));
    assert_eq!(pattern["binds"], json!(false));
    assert_eq!(texts(&repo, &["a.py"]), ["old"]);
}

/// `decide --scope at:<place>` writes the place as a `path` tag, without its line and with `/**` after
/// a folder, and no `scope` tag (`spec/02-cli.md`, "Commands"; `spec/05-patterns.md`, "Scope").
#[test]
fn an_at_scope_writes_a_path() {
    let repo = Repo::new();
    repo.write("shelf/b.py", "x = 1\n");
    let file = decide(&repo, &["--scope", "at:a.py:3"], "at a line");
    let folder = decide(&repo, &["--scope", "at:shelf"], "in the shelf");
    let notes = support::labelled(&repo, "decision");
    let tags = |id: &str| {
        let note = notes.iter().find(|n| n.id == id).unwrap();
        (
            note.tag("path").map(str::to_string),
            note.tag("scope").map(str::to_string),
        )
    };
    assert_eq!(tags(&file), (Some("a.py".to_string()), None));
    assert_eq!(tags(&folder), (Some("shelf/**".to_string()), None));
}
