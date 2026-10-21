//! CIP-05, "Triggers", and CIP-02, "Rules".

mod support;

use celis_nostr::Note;
use serde_json::{json, Value};
use support::{account, add, err, objects, out, put, rule_tags, stored_rule, Repo, DAY};

/// The id `celis decide <args> -- <text>` prints in the main checkout.
fn decide(repo: &Repo, args: &[&str], text: &str) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    all.extend(["--", text]);
    let o = repo.run(&repo.path(), &all, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// A rule of this repo with the extra tags `tags`, saying `content`, made at `at`, put in the store.
fn tagged(repo: &Repo, tags: &[(&str, &str)], content: &str, at: u64) -> Note {
    let mut all = rule_tags((&repo.project_id(), "repo"));
    all.extend(tags.iter().map(|(n, v)| vec![n.to_string(), v.to_string()]));
    let note = Note::write(&account(), 1111u16, at, all, content).unwrap();
    put(repo, &note);
    note
}

/// The lines `celis rules <args>` prints in the main checkout.
fn rules(repo: &Repo, args: &[&str]) -> Vec<String> {
    let mut all = vec!["rules"];
    all.extend(args);
    let o = repo.run(&repo.path(), &all, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

/// The text of each rule `celis rules <args>` prints, without its id and scope.
fn texts(repo: &Repo, args: &[&str]) -> Vec<String> {
    let lines = rules(repo, args).into_iter();
    lines
        .map(|l| l.splitn(3, ' ').nth(2).unwrap().to_string())
        .collect()
}

/// On a read, a `path` glob matches the file and `content` matches the file's text on disk
/// (`spec/05-rules.md`, "Triggers").
#[test]
fn a_read_matches_the_glob_and_the_files_text() {
    let repo = Repo::new();
    decide(
        &repo,
        &["--path", "**/*.py", "--content", "float"],
        "floats",
    );
    decide(&repo, &["--path", "shelf/*.py"], "shelf");
    repo.write("shelf/a.py", "x = float(1)\n");
    repo.write("shelf/b.py", "x = 1\n");
    repo.write("c.rs", "float\n");
    assert_eq!(texts(&repo, &["shelf/a.py"]), ["floats", "shelf"]);
    assert_eq!(texts(&repo, &["shelf/b.py"]), ["shelf"]);
    assert!(texts(&repo, &["c.rs"]).is_empty());
}

/// A file that does not exist yet matches by its `path` alone, whatever its `content`
/// (`spec/05-rules.md`, "Triggers").
#[test]
fn a_file_not_yet_made_matches_by_its_path_alone() {
    let repo = Repo::new();
    decide(
        &repo,
        &["--path", "**/*.py", "--content", "float"],
        "floats",
    );
    decide(&repo, &["--path", "*.rs"], "rust");
    assert_eq!(texts(&repo, &["shelf/new.py"]), ["floats"]);
    repo.write("shelf/new.py", "x = 1\n");
    assert!(texts(&repo, &["shelf/new.py"]).is_empty());
}

/// `--diff` counts as a read of each file git shows changed or new, and not of a deleted one
/// (`spec/02-cli.md`, "Rules").
#[test]
fn diff_reads_the_files_git_shows_changed() {
    let repo = Repo::new();
    decide(&repo, &["--path", "*.md"], "markdown");
    decide(&repo, &["--path", "*.py"], "python");
    decide(&repo, &["--path", "*.rs"], "rust");
    assert!(rules(&repo, &["--diff"]).is_empty());
    repo.write("new.py", "x = 1\n");
    repo.write("README.md", "changed\n");
    let mut found = texts(&repo, &["--diff"]);
    found.sort();
    assert_eq!(found, ["markdown", "python"]);
}

/// Past 16 KB, `rules` first prints how many rules match and how to read them in pages; `--page`
/// prints one page of 100 rules after its line, and refuses a page that is no whole number from 1
/// (`spec/02-cli.md`, "Rules").
#[test]
fn many_rules_come_with_their_size_and_in_pages() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    let long = "x".repeat(150);
    for n in 0..120u64 {
        tagged(
            &repo,
            &[("path", "*.md")],
            &format!("rule {n} {long}"),
            at + n,
        );
    }
    let all = rules(&repo, &["README.md"]);
    assert_eq!(all.len(), 121);
    assert!(
        all[0].starts_with("120 rules match (") && all[0].ends_with("--page 1 to --page 2"),
        "{}",
        all[0]
    );
    let second = rules(&repo, &["README.md", "--page", "2"]);
    assert_eq!(second[0], "page 2 of 2: rules 101-120 of 120");
    assert_eq!(second.len(), 21);
    assert!(second[1].contains(" rule 19 "), "{}", second[1]);
    for page in ["0", "x"] {
        let o = repo.run(
            &repo.path(),
            &["rules", "README.md", "--page", page],
            &[],
            "",
        );
        assert_eq!(o.status.code(), Some(2), "{o:?}");
    }
}

/// A folder counts as a read of each file git gives under it, ignored files left out
/// (`spec/02-cli.md`, "Rules").
#[test]
fn a_folder_gives_the_rules_of_its_files() {
    let repo = Repo::new();
    decide(
        &repo,
        &["--path", "shelf/*.py", "--content", "float"],
        "floats",
    );
    decide(&repo, &["--path", "**/*.md"], "markdown");
    decide(&repo, &["--path", "*.log"], "logs");
    repo.write("shelf/a.py", "x = float(1)\n");
    repo.write("shelf/docs/b.md", "b\n");
    repo.write("shelf/out.log", "x\n");
    repo.write(".gitignore", "*.log\n");
    let mut shelf = texts(&repo, &["shelf"]);
    shelf.sort();
    assert_eq!(shelf, ["floats", "markdown"]);
    let o = repo.run(&repo.path().join("shelf"), &["rules", "."], &[], "");
    let shown = out(&o);
    let mut here: Vec<&str> = shown
        .lines()
        .map(|l| l.splitn(3, ' ').nth(2).unwrap())
        .collect();
    here.sort();
    assert_eq!(here, ["floats", "markdown"]);
}

/// A rule from an older version with only a `tool` tag reads with no trigger, and never comes
/// (`spec/README.md`, "Names").
#[test]
fn an_older_rule_with_a_tool_tag_has_no_trigger() {
    let repo = Repo::new();
    tagged(&repo, &[("tool", "^Grep$")], "grep", support::now() - DAY);
    let o = repo.run(&repo.path(), &["grep", " rule "], &[], "");
    assert!(out(&o).contains("live {project} grep"), "{}", out(&o));
    assert!(rules(&repo, &[".", "--", "Grep"]).is_empty());
}

/// On a shell command, `command` matches it, or a file the command names matches as a file call with
/// the command as its text (`spec/05-rules.md`, "Triggers").
#[test]
fn a_command_matches_its_regex_and_the_files_it_names() {
    let repo = Repo::new();
    decide(&repo, &["--command", "^pytest"], "tests");
    decide(&repo, &["--path", "shelf/*.py"], "shelf");
    decide(&repo, &["--path", "*.md", "--content", "^sed"], "sed");
    let bash = |line: &str| texts(&repo, &["--", line]);
    assert_eq!(bash("pytest -q"), ["tests"]);
    assert_eq!(bash("cat shelf/a.py | head"), ["shelf"]);
    assert_eq!(bash("sed -i s/a/b/ README.md"), ["sed"]);
    assert!(bash("cat README.md").is_empty());
    assert!(bash("ls").is_empty());
    assert_eq!(
        texts(&repo, &["--", "pytest", "shelf/a.py"]),
        ["tests", "shelf"]
    );
}

/// The files and the command of one call each match (`spec/02-cli.md`, "Rules").
#[test]
fn the_files_and_the_command_each_match() {
    let repo = Repo::new();
    decide(&repo, &["--command", "^pytest"], "tests");
    decide(&repo, &["--path", "*.md"], "markdown");
    decide(&repo, &["--path", "*.py"], "python");
    let mut shown = texts(&repo, &["README.md", "a.py", "--", "pytest -q"]);
    shown.sort();
    assert_eq!(shown, ["markdown", "python", "tests"]);
}

/// With no file and no command, `rules` gives each word that two or more live rules use before their
/// "because", most used first (`spec/02-cli.md`, "Rules").
#[test]
fn rules_with_no_file_and_no_command_give_the_index() {
    let repo = Repo::new();
    decide(
        &repo,
        &[],
        "When prices change: keep prices in cents, because floats drift.",
    );
    decide(
        &repo,
        &["--path", "*.py"],
        "When prices show: round prices, because cents.",
    );
    decide(
        &repo,
        &["--situation"],
        "When a refund comes: log the refund.",
    );
    for args in [&[][..], &["--", " "]] {
        let lines = rules(&repo, args);
        assert_eq!(
            lines,
            [
                "3 live rules. Their words, by how many rules use each:",
                "prices 2",
            ],
            "{lines:?}"
        );
    }
}

/// A pattern that does not compile matches nothing (`spec/05-rules.md`, "Triggers").
#[test]
fn a_pattern_that_does_not_compile_matches_nothing() {
    let repo = Repo::new();
    decide(&repo, &["--path", "*.md", "--content", "("], "content");
    decide(&repo, &["--command", "("], "command");
    let bash = texts(&repo, &["--", "cat README.md ("]);
    assert!(bash.is_empty(), "{bash:?}");
    assert!(rules(&repo, &["README.md"]).is_empty());
}

/// A rule `at:<place>` counts as a rule with that place as its `path`, and holds under a folder it
/// names (`spec/05-rules.md`, "Triggers").
#[test]
fn an_at_rule_comes_at_its_place() {
    let repo = Repo::new();
    decide(&repo, &["--scope", "at:shelf"], "in the shelf");
    decide(&repo, &["--scope", "at:a.py:3"], "at a line");
    assert_eq!(texts(&repo, &["shelf/b.py"]), ["in the shelf"]);
    assert_eq!(texts(&repo, &["a.py"]), ["at a line"]);
    assert!(rules(&repo, &["b.py"]).is_empty());
}

/// A rule with no trigger and no `at:<place>` never comes, and a situation rule with a trigger comes at
/// its calls (`spec/05-rules.md`, "Triggers"; `spec/02-cli.md`, "Rules").
#[test]
fn a_rule_with_no_trigger_never_comes() {
    let repo = Repo::new();
    decide(&repo, &["--situation"], "situation");
    decide(&repo, &[], "no trigger");
    decide(&repo, &["--scope", "me"], "mine");
    assert!(rules(&repo, &["README.md", "--", "ls"]).is_empty());
    decide(&repo, &["--path", "*.md", "--situation"], "both");
    assert_eq!(texts(&repo, &["README.md"]), ["both"]);
}

/// Rules come with a named file first, then a glob with content or a command, then a glob with no
/// content; newest first within each (`spec/02-cli.md`, "Rules").
#[test]
fn rules_come_narrowest_first_then_newest() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    let order = [
        (&[("path", "*.py")][..], "wide old", at),
        (&[("path", "*.py")], "wide new", at + 1),
        (
            &[("path", "**/*.py"), ("content", "x")],
            "content old",
            at + 2,
        ),
        (&[("command", "cat")], "command new", at + 3),
        (&[("path", "a.py")], "named old", at + 4),
        (&[("scope", "at:a.py")], "at new", at + 5),
    ];
    for (tags, content, made) in order {
        tagged(&repo, tags, content, made);
    }
    assert_eq!(
        texts(&repo, &["--", "cat a.py x"]),
        [
            "at new",
            "named old",
            "command new",
            "content old",
            "wide new",
            "wide old"
        ]
    );
}

/// A rule given before in the session comes only as its id, on one line after the others; outside the
/// session it comes in full; `--json` gives each as an object (`spec/02-cli.md`, "Rules").
#[test]
fn a_rule_given_before_in_the_session_comes_as_its_id() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    let ids: Vec<String> = (0..3u64)
        .map(|n| tagged(&repo, &[("path", "*.md")], &format!("rule {n}"), at + n).id)
        .collect();
    let first = texts(&repo, &["README.md", "--session", "s1"]);
    assert_eq!(first, ["rule 2", "rule 1", "rule 0"]);
    tagged(&repo, &[("path", "*.md")], "rule 3", at + 3);
    let short: Vec<&str> = ids.iter().rev().map(|i| &i[..8]).collect();
    assert_eq!(
        rules(&repo, &["README.md", "--session", "s1"])[1..],
        [format!("given before: {}", short.join(" "))]
    );
    assert_eq!(texts(&repo, &["README.md", "--session", "s2"]).len(), 4);
    assert_eq!(texts(&repo, &["README.md"]).len(), 4);
    let json = objects(&repo, &["rules", "README.md", "--session", "s1"]);
    assert_eq!(json.len(), 1, "{json:?}");
    assert_eq!(json[0]["given"].as_array().unwrap().len(), 4);
    let json = objects(&repo, &["rules", "README.md"]);
    let keys: Vec<&String> = json[0].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["content", "id", "scope"]);
}

/// Each rule printed in a session is logged in `$CELIS_HOME/given`, with the session from `--session`,
/// else the environment; with no session, none is logged (`spec/02-cli.md`, "Rules").
#[test]
fn each_rule_printed_in_a_session_is_logged() {
    let repo = Repo::new();
    let id = decide(&repo, &["--path", "*.md"], "markdown");
    assert!(rules(&repo, &["a.rs", "--session", "s1"]).is_empty());
    rules(&repo, &["README.md"]);
    assert!(!repo.home().join("given").exists());
    rules(&repo, &["README.md", "--session", "s1"]);
    let env = [("CELIS_SESSION_ID", "env")];
    repo.run(&repo.path(), &["rules", "README.md"], &env, "");
    let log = std::fs::read_to_string(repo.home().join("given")).unwrap();
    let lines: Vec<Value> = log
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 2, "{log}");
    assert_eq!(
        (&lines[0]["session"], &lines[0]["rule"]),
        (&json!("s1"), &json!(id))
    );
    assert!(lines[0]["at"].is_u64());
    assert_eq!(lines[1]["session"], "env");
}

/// The grep view marks a rule with a trigger, a situation rule and a promoted rule (`spec/02-cli.md`,
/// "Grep").
#[test]
fn grep_marks_triggers_situations_and_promoted_rules() {
    let repo = Repo::new();
    let trigger = decide(&repo, &["--command", "^make"], "trigger");
    let all = decide(
        &repo,
        &["--path", "*.md", "--situation", "--promoted"],
        "all",
    );
    let plain = decide(&repo, &[], "plain");
    let o = repo.run(&repo.path(), &["grep", " rule "], &[], "");
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
        line(&all).ends_with("live {project} trigger situation promoted all"),
        "{shown}"
    );
    assert!(line(&plain).ends_with("live {project} plain"), "{shown}");
}

/// `decide` writes each `--path` from the project's top folder, `--content`, `--command`,
/// `--situation` and `--promoted`, and a reader reads them back; `--tool` is no flag
/// (`spec/02-cli.md`, "Commands").
#[test]
fn decide_writes_the_trigger_and_labels() {
    let repo = Repo::new();
    std::fs::create_dir_all(repo.path().join("shelf")).unwrap();
    let o = repo.run(
        &repo.path().join("shelf"),
        &[
            "decide",
            "--scope",
            "at:a.py",
            "--path",
            "*.py",
            "--path",
            "b.md",
            "--content",
            "x",
            "--command",
            "^make",
            "--situation",
            "--promoted",
            "--",
            "kept",
        ],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).trim().to_string();
    let step = add(&repo, &["--new", "--uses", &id, "--", "t"]);
    let step = step.lines().next().unwrap();
    let shown = objects(&repo, &["step", "start", step]);
    let rule = shown.iter().find_map(|v| v.get("decision")).unwrap();
    assert_eq!(
        rule["trigger"],
        json!({"paths": ["shelf/*.py", "shelf/b.md"], "content": "x", "command": "^make"})
    );
    assert_eq!(
        (&rule["situation"], &rule["promoted"]),
        (&json!(true), &json!(true))
    );
    let o = repo.run(
        &repo.path(),
        &["decide", "--tool", "Grep", "--", "t"],
        &[],
        "",
    );
    assert_eq!(o.status.code(), Some(2), "{o:?}");
}

/// A rule from before triggers still reads, with no trigger and no label, and comes at its place
/// (`spec/05-rules.md`, "Rules").
#[test]
fn an_older_rule_with_no_trigger_still_reads() {
    let repo = Repo::new();
    let id = repo.project_id();
    let at = support::now() - DAY;
    let old = stored_rule(
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
    let rule = shown.iter().find_map(|v| v.get("decision")).unwrap();
    let empty = json!({"paths": [], "content": null, "command": null});
    assert_eq!(rule["trigger"], empty);
    assert_eq!(
        (&rule["situation"], &rule["promoted"]),
        (&json!(false), &json!(false))
    );
    assert_eq!(texts(&repo, &["a.py"]), ["old"]);
}
