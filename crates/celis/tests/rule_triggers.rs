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

/// The lines `celis rules <args>` prints in the main checkout, with `stdin`.
fn rules(repo: &Repo, args: &[&str], stdin: &str) -> Vec<String> {
    let mut all = vec!["rules"];
    all.extend(args);
    let o = repo.run(&repo.path(), &all, &[], stdin);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

/// The texts of the rules `celis rules` prints for the hook `tool` with `input` on `event`, in the
/// session `session`.
fn hooked(repo: &Repo, event: &str, tool: &str, input: Value, session: &str) -> Vec<String> {
    let hook = json!({
        "session_id": session,
        "cwd": repo.path(),
        "hook_event_name": event,
        "tool_name": tool,
        "tool_input": input,
    });
    texts(&rules(repo, &[], &hook.to_string()))
}

/// The text of each rule line of `lines`, without its id and scope.
fn texts(lines: &[String]) -> Vec<String> {
    let rules = lines.iter().filter(|l| !l.starts_with("celis: "));
    rules
        .map(|l| l.splitn(3, ' ').nth(2).unwrap().to_string())
        .collect()
}

fn file(repo: &Repo, rel: &str) -> String {
    repo.path().join(rel).display().to_string()
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
    let read = |rel: &str| {
        hooked(
            &repo,
            "PreToolUse",
            "Read",
            json!({"file_path": file(&repo, rel)}),
            "s",
        )
    };
    assert_eq!(read("shelf/a.py"), ["floats", "shelf"]);
    assert_eq!(texts(&rules(&repo, &["shelf/b.py"], "")), ["shelf"]);
    assert!(read("c.rs").is_empty());
}

/// On a write or an edit, `content` matches the new text: `content`, `new_string`, each
/// `edits[].new_string`, or `new_source` (`spec/05-rules.md`, "Triggers"; `spec/02-cli.md`, "Rules").
#[test]
fn a_write_or_an_edit_matches_its_new_text() {
    let repo = Repo::new();
    decide(&repo, &["--path", "*.py", "--content", "float"], "floats");
    repo.write("a.py", "float on disk\n");
    let path = file(&repo, "a.py");
    let call = |tool: &str, input: Value| hooked(&repo, "PreToolUse", tool, input, "");
    assert_eq!(
        call("Write", json!({"file_path": path, "content": "float(1)"})),
        ["floats"]
    );
    assert!(call("Write", json!({"file_path": path, "content": "1"})).is_empty());
    assert_eq!(
        call("Edit", json!({"file_path": path, "new_string": "float"})),
        ["floats"]
    );
    assert!(call("Edit", json!({"file_path": path, "new_string": "int"})).is_empty());
    let edits = json!([{"new_string": "int"}, {"new_string": "float"}]);
    assert_eq!(
        call("MultiEdit", json!({"file_path": path, "edits": edits})),
        ["floats"]
    );
    let edits = json!([{"new_string": "int"}]);
    assert!(call("MultiEdit", json!({"file_path": path, "edits": edits})).is_empty());
    let notebook = json!({"notebook_path": file(&repo, "n.py"), "new_source": "float"});
    assert_eq!(call("NotebookEdit", notebook), ["floats"]);
}

/// On a shell command, `command` matches it, or a file the command names matches as a file call with
/// the command as its text (`spec/05-rules.md`, "Triggers").
#[test]
fn a_command_matches_its_regex_and_the_files_it_names() {
    let repo = Repo::new();
    decide(&repo, &["--command", "^pytest"], "tests");
    decide(&repo, &["--path", "shelf/*.py"], "shelf");
    decide(&repo, &["--path", "*.md", "--content", "^sed"], "sed");
    let bash = |line: &str| hooked(&repo, "PreToolUse", "Bash", json!({"command": line}), "");
    assert_eq!(bash("pytest -q"), ["tests"]);
    assert_eq!(bash("cat shelf/a.py | head"), ["shelf"]);
    assert_eq!(bash("sed -i s/a/b/ README.md"), ["sed"]);
    assert!(bash("cat README.md").is_empty());
    assert!(bash("ls").is_empty());
    assert_eq!(
        texts(&rules(&repo, &["--", "pytest", "shelf/a.py"], "")),
        ["tests", "shelf"]
    );
}

/// After a shell command, each file it changed matches as a file call with its text: the files
/// `git status --porcelain` gives (`spec/05-rules.md`, "Triggers"; `spec/02-cli.md`, "Rules").
#[test]
fn after_a_command_each_changed_file_matches_with_its_text() {
    let repo = Repo::new();
    decide(
        &repo,
        &["--path", "**/*.py", "--content", "float"],
        "floats",
    );
    decide(&repo, &["--path", "README.md"], "readme");
    let after = || hooked(&repo, "PostToolUse", "Bash", json!({"command": "make"}), "");
    assert!(after().is_empty());
    repo.write("shelf/new.py", "x = float(1)\n");
    repo.write("old.py", "x = 1\n");
    assert_eq!(after(), ["floats"]);
    repo.write("README.md", "changed\n");
    assert_eq!(after(), ["readme", "floats"]);
}

/// On another tool call, `tool` matches its name (`spec/05-rules.md`, "Triggers").
#[test]
fn another_tool_call_matches_by_its_name() {
    let repo = Repo::new();
    decide(&repo, &["--tool", "^Grep$"], "grep");
    let call = |tool: &str| hooked(&repo, "PreToolUse", tool, json!({"pattern": "x"}), "");
    assert_eq!(call("Grep"), ["grep"]);
    assert!(call("Glob").is_empty());
}

/// A pattern that does not compile matches nothing (`spec/05-rules.md`, "Triggers").
#[test]
fn a_pattern_that_does_not_compile_matches_nothing() {
    let repo = Repo::new();
    decide(&repo, &["--path", "*.md", "--content", "("], "content");
    decide(&repo, &["--command", "("], "command");
    decide(&repo, &["--tool", "["], "tool");
    let bash = hooked(
        &repo,
        "PreToolUse",
        "Bash",
        json!({"command": "cat README.md ("}),
        "",
    );
    assert!(bash.is_empty(), "{bash:?}");
    assert!(hooked(&repo, "PreToolUse", "[", json!({}), "").is_empty());
    assert!(rules(&repo, &["README.md"], "").is_empty());
}

/// A rule `at:<place>` counts as a rule with that place as its `path`, and holds under a folder it
/// names (`spec/05-rules.md`, "Triggers").
#[test]
fn an_at_rule_comes_at_its_place() {
    let repo = Repo::new();
    decide(&repo, &["--scope", "at:shelf"], "in the shelf");
    decide(&repo, &["--scope", "at:a.py:3"], "at a line");
    assert_eq!(texts(&rules(&repo, &["shelf/b.py"], "")), ["in the shelf"]);
    assert_eq!(texts(&rules(&repo, &["a.py"], "")), ["at a line"]);
    assert!(rules(&repo, &["b.py"], "").is_empty());
}

/// A rule with no trigger and no `at:<place>` never comes, and a situation rule with a trigger comes at
/// its calls (`spec/05-rules.md`, "Triggers"; `spec/02-cli.md`, "Rules").
#[test]
fn a_rule_with_no_trigger_never_comes() {
    let repo = Repo::new();
    decide(&repo, &["--situation"], "situation");
    decide(&repo, &[], "no trigger");
    decide(&repo, &["--scope", "me"], "mine");
    assert!(rules(&repo, &["README.md", "--all", "--", "ls"], "").is_empty());
    decide(&repo, &["--path", "*.md", "--situation"], "both");
    assert_eq!(texts(&rules(&repo, &["README.md"], "")), ["both"]);
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
    let shown = texts(&rules(&repo, &["--all", "--", "cat a.py x"], ""));
    assert_eq!(
        shown,
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

/// At most 5 rules come a call; the rest come on the next call of the session, with a pointer, or all
/// with `--all`, which logs none; each rule once a session (`spec/02-cli.md`, "Rules").
#[test]
fn five_rules_a_call_then_a_pointer_to_the_rest() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    for n in 0..7u64 {
        tagged(&repo, &[("path", "*.md")], &format!("rule {n}"), at + n);
    }
    let first = rules(&repo, &["README.md", "--session", "s1"], "");
    assert_eq!(
        texts(&first),
        ["rule 6", "rule 5", "rule 4", "rule 3", "rule 2"]
    );
    assert_eq!(
        first.last().unwrap(),
        "celis: 2 more rules match this call; run it again for the next 5, or add --all."
    );
    let second = rules(&repo, &["README.md", "--session", "s1"], "");
    assert_eq!(second.len(), 2, "{second:?}");
    assert_eq!(texts(&second), ["rule 1", "rule 0"]);
    assert!(rules(&repo, &["README.md", "--session", "s1"], "").is_empty());
    let look = ["README.md", "--session", "s1", "--all"];
    assert_eq!(texts(&rules(&repo, &look, "")).len(), 7);
    assert_eq!(texts(&rules(&repo, &look, "")).len(), 7);
    let fresh = rules(&repo, &["README.md", "--session", "s2"], "");
    assert_eq!(texts(&fresh).len(), 5);
    let alone = rules(&repo, &["README.md"], "");
    assert_eq!(
        alone.last().unwrap(),
        "celis: 2 more rules match this call; add --all to see them."
    );
    assert_eq!(texts(&rules(&repo, &["README.md"], "")).len(), 5);
    let json = objects(&repo, &["rules", "README.md"]);
    assert_eq!(json.len(), 6);
    assert_eq!(json[5], json!({"more": 2}));
    let keys: Vec<&String> = json[0].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["content", "id", "scope"]);
}

/// Each rule given in a session is logged in `$CELIS_HOME/given`; after a compaction that `prompt` logs,
/// it comes again (`spec/02-cli.md`, "Rules" and "Prompt").
#[test]
fn a_compaction_gives_the_rules_again() {
    let repo = Repo::new();
    let id = decide(&repo, &["--path", "*.md"], "markdown");
    let hook = |session: &str| {
        hooked(
            &repo,
            "PreToolUse",
            "Read",
            json!({"file_path": file(&repo, "README.md")}),
            session,
        )
    };
    assert_eq!(hook("s1"), ["markdown"]);
    assert!(hook("s1").is_empty());
    let compact = json!({"session_id": "s1", "source": "compact"}).to_string();
    let o = repo.run(&repo.path(), &["prompt"], &[], &compact);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).is_empty() && err(&o).is_empty(), "{o:?}");
    assert_eq!(hook("s1"), ["markdown"]);
    let log = std::fs::read_to_string(repo.home().join("given")).unwrap();
    let lines: Vec<Value> = log
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(
        (&lines[0]["session"], &lines[0]["rule"]),
        (&json!("s1"), &json!(id))
    );
    assert!(lines[0]["at"].is_u64());
    assert_eq!(lines[1]["compact"], true);
    assert_eq!(lines[2]["rule"], id.as_str());
}

/// A hook that matches nothing prints nothing; the session comes from the environment when the hook
/// and `--session` name none (`spec/02-cli.md`, "Rules").
#[test]
fn a_hook_that_matches_nothing_prints_nothing() {
    let repo = Repo::new();
    decide(&repo, &["--path", "*.md"], "markdown");
    let hook = json!({"hook_event_name": "PreToolUse", "tool_name": "Read", "tool_input": {"file_path": file(&repo, "a.rs")}});
    let o = repo.run(&repo.path(), &["rules"], &[], &hook.to_string());
    assert!(
        o.status.success() && out(&o).is_empty() && err(&o).is_empty(),
        "{o:?}"
    );
    let env = [("CELIS_SESSION_ID", "env")];
    for _ in 0..2 {
        repo.run(&repo.path(), &["rules", "README.md"], &env, "");
    }
    let log = std::fs::read_to_string(repo.home().join("given")).unwrap();
    assert_eq!(log.lines().count(), 1, "{log}");
}

/// The grep view marks a rule with a trigger, a situation rule and a promoted rule (`spec/02-cli.md`,
/// "Grep").
#[test]
fn grep_marks_triggers_situations_and_promoted_rules() {
    let repo = Repo::new();
    let trigger = decide(&repo, &["--tool", "Grep"], "trigger");
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

/// `decide` writes each `--path` from the project's top folder, `--content`, `--command`, `--tool`,
/// `--situation` and `--promoted`, and a reader reads them back (`spec/02-cli.md`, "Commands").
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
            "--tool",
            "Grep",
            "--situation",
            "--promoted",
            "--",
            "kept",
        ],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let step = add(&repo, &["--new", "--at", "shelf/a.py", "--", "t"]);
    let step = step.lines().next().unwrap();
    let shown = objects(&repo, &["step", "start", step]);
    let rule = shown.iter().find_map(|v| v.get("decision")).unwrap();
    assert_eq!(
        rule["trigger"],
        json!({"paths": ["shelf/*.py", "shelf/b.md"], "content": "x", "command": "^make", "tool": "Grep"})
    );
    assert_eq!(
        (&rule["situation"], &rule["promoted"]),
        (&json!(true), &json!(true))
    );
}

/// A rule from before triggers still reads, with no trigger and no label, and comes at its place
/// (`spec/05-rules.md`, "Rules").
#[test]
fn an_older_rule_with_no_trigger_still_reads() {
    let repo = Repo::new();
    let id = repo.project_id();
    let at = support::now() - DAY;
    stored_rule(
        &repo,
        (&id, "repo"),
        &account(),
        (Some("at:a.py"), None),
        "old",
        at,
    );
    let step = add(&repo, &["--new", "--at", "a.py", "--", "t"]);
    let step = step.lines().next().unwrap();
    let shown = objects(&repo, &["step", "start", step]);
    let rule = shown.iter().find_map(|v| v.get("decision")).unwrap();
    let empty = json!({"paths": [], "content": null, "command": null, "tool": null});
    assert_eq!(rule["trigger"], empty);
    assert_eq!(
        (&rule["situation"], &rule["promoted"]),
        (&json!(false), &json!(false))
    );
    assert_eq!(texts(&rules(&repo, &["a.py"], "")), ["old"]);
}
