//! CIP-02, "Output and errors".

mod support;

use serde_json::Value;
use std::process::Output;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use support::{
    account, err, out, put, rule_tags, sharing_a_start, sharing_a_start_with, step_tags, Repo,
};

fn objects(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).expect("one JSON object a line"))
        .collect()
}

/// Every command accepts `--json`; `decide` prints the id, or `{"id"}` (`spec/02-cli.md`,
/// "Output and errors").
#[test]
fn every_command_accepts_json_and_prints_the_id() {
    let repo = Repo::new();
    let dir = repo.path();
    let o = repo.run(&dir, &["decide", "--", "one"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(is_id(out(&o).trim()), "{}", out(&o));

    let again = repo.run(&dir, &["decide", "--json", "--", "two"], &[], "");
    let objs = objects(&out(&again));
    assert_eq!(objs.len(), 1);
    assert!(is_id(objs[0]["id"].as_str().unwrap()));

    let o = repo.run(&dir, &["tree", "--all", "--json"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    objects(&out(&o));
}

/// Runs `celis <args>` with stdin left open and returns its output.
fn without_asking(repo: &Repo, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_celis"))
        .args(args)
        .current_dir(repo.path())
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("CELIS_HOME", repo.home())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let held = child.stdin.take().unwrap();
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(20) {
            child.kill().unwrap();
            panic!("{args:?} waited on stdin");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(held);
    let o = child.wait_with_output().unwrap();
    assert!(o.status.success(), "{args:?}: {}", err(&o));
    let said = format!("{}{}", out(&o), err(&o));
    let asked = said.lines().any(|l| l.trim_end().ends_with('?'));
    assert!(!asked, "{args:?}: {said}");
    o
}

fn step_id(repo: &Repo, text: &str) -> String {
    let o = repo.run(&repo.path(), &["step", "add", "--", text], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

fn trail_id(repo: &Repo, text: &str) -> String {
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--json", "--", text],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    objects(&out(&o))[0]["id"].as_str().unwrap().to_string()
}

fn is_id(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// A trail or step command run with its stdin left open finishes, and prints no question
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_trail_or_step_command_with_stdin_left_open_asks_nothing() {
    let repo = Repo::plain();
    let closed = step_id(&repo, "two")[..8].to_string();
    let trail = trail_id(&repo, "senior tier");
    let scope = format!("trail:{}", &trail[..8]);
    let runs: Vec<Vec<&str>> = vec![
        vec!["decide", "--scope", &scope, "--", "80%"],
        vec!["tree", &trail[..8]],
        vec!["step", "add", "--", "four"],
        vec!["grep", "senior"],
        vec!["step", "done", &closed, "--as-planned", "--", "it passes"],
    ];
    for args in runs {
        without_asking(&repo, &args);
    }
}

/// `grep` run with its stdin left open finishes, and prints no question (`spec/02-cli.md`, "Output
/// and errors").
#[test]
fn grep_with_stdin_left_open_asks_nothing() {
    let repo = Repo::new();
    trail_id(&repo, "senior tier");
    without_asking(&repo, &["grep", "senior"]);
}

/// `decide` and a `grep` for its place run with their stdin left open finish, and print no question
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn decide_and_grep_at_with_stdin_left_open_ask_nothing() {
    let repo = Repo::new();
    for args in [
        vec![
            "decide",
            "--scope",
            "at:a.py",
            "--",
            "When you edit: test, because.",
        ],
        vec!["grep", "at:a.py"],
    ] {
        without_asking(&repo, &args);
    }
}

/// A command takes an id whole, or as its first 4 or more characters when only one record's id starts
/// with them; 3 characters name no record (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_command_takes_an_id_whole_or_as_a_start_of_4_or_more_characters() {
    let repo = Repo::plain();
    let (a, b, c) = (
        step_id(&repo, "one"),
        step_id(&repo, "two"),
        step_id(&repo, "three"),
    );
    let o = repo.run(
        &repo.path(),
        &["step", "done", &c[..3], "--as-planned", "--", "p"],
        &[],
        "",
    );
    assert!(!o.status.success(), "{}", out(&o));
    for text in [a.as_str(), &b[..8], &c[..4]] {
        let o = repo.run(
            &repo.path(),
            &["step", "done", text, "--as-planned", "--", "p"],
            &[],
            "",
        );
        assert!(o.status.success(), "{text}: {}", err(&o));
    }
    let (g, h) = (trail_id(&repo, "one"), trail_id(&repo, "two"));
    for text in [g.as_str(), &h[..8]] {
        let o = repo.run(&repo.path(), &["tree", text], &[], "");
        assert!(o.status.success(), "{text}: {}", err(&o));
        assert!(!out(&o).is_empty(), "{text}");
    }
}

/// `decide --replaces` takes the first 8 characters of a rule's id
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn decide_replaces_takes_the_first_8_characters_of_an_id() {
    let repo = Repo::plain();
    let first = repo.run(&repo.path(), &["decide", "--", "old"], &[], "");
    assert!(first.status.success(), "{}", err(&first));
    let id = out(&first).trim().to_string();
    let args = ["decide", "--replaces", &id[..8], "--", "new"];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let read = out(&repo.run(&repo.path(), &["grep", " live "], &[], ""));
    assert!(read.contains("new") && !read.contains("old"), "{read}");
}

/// A trail or step command accepts `--json` and prints one JSON object a line, and nothing else
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_trail_or_step_command_prints_one_json_object_a_line_with_json() {
    let repo = Repo::plain();
    let id = step_id(&repo, "one");
    let trail = trail_id(&repo, "g");
    let scope = format!("trail:{trail}");
    for args in [
        vec!["decide", "--json", "--scope", &scope, "--", "80%"],
        vec!["tree", "--json", &trail],
        vec!["tree", "--all", "--json"],
        vec!["step", "done", "--json", &id, "--as-planned", "--", "p"],
    ] {
        let o = repo.run(&repo.path(), &args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
        for line in out(&o).lines() {
            assert!(serde_json::from_str::<Value>(line).unwrap().is_object());
        }
    }
}

/// `decide` accepts `--json` and prints one JSON object a line (`spec/02-cli.md`, "Output and
/// errors").
#[test]
fn decide_prints_one_json_object_a_line_with_json() {
    let repo = Repo::plain();
    let o = repo.run(&repo.path(), &["decide", "--json", "--", "one"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(objects(&out(&o)).len(), 1);
}

/// A trail or step command that writes a record prints its id, and `{"id"}` with `--json`
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_command_that_writes_a_record_prints_its_id() {
    let repo = Repo::plain();
    for args in [vec!["decide", "--", "g"], vec!["step", "add", "--", "p"]] {
        let o = repo.run(&repo.path(), &args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
        assert!(is_id(out(&o).trim()), "{args:?}: {}", out(&o));
    }
    for args in [
        vec!["decide", "--json", "--", "g2"],
        vec!["step", "add", "--json", "--", "p2"],
    ] {
        let o = repo.run(&repo.path(), &args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
        let lines = objects(&out(&o));
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_object().unwrap().len(), 1, "{}", lines[0]);
        assert!(is_id(lines[0]["id"].as_str().unwrap()));
    }
}

/// `decide` prints the id of its record, and `{"id"}` with `--json` (`spec/02-cli.md`,
/// "Output and errors").
#[test]
fn decide_prints_its_id() {
    let repo = Repo::new();
    let o = repo.run(&repo.path(), &["decide", "--", "one"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(is_id(out(&o).trim()), "{}", out(&o));
    let o = repo.run(&repo.path(), &["decide", "--json", "--", "two"], &[], "");
    let lines = objects(&out(&o));
    assert_eq!(lines[0].as_object().unwrap().len(), 1);
    assert!(is_id(lines[0]["id"].as_str().unwrap()));
}

/// With `--json`, a `--replaces` no live rule has is named `no-decision`
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_replaces_no_live_rule_has_is_named_no_rule_under_json() {
    let repo = Repo::new();
    let none = "0".repeat(64);
    let args = ["decide", "--json", "--replaces", &none, "--", "x"];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let object: Value = serde_json::from_str(err(&o).trim()).expect("one JSON object");
    assert_eq!(object["error"], "no-decision");
}

/// A usage error names the narrowest help page that covers it.
#[test]
fn a_usage_error_names_the_narrowest_help_page() {
    let repo = Repo::new();
    let dir = repo.path();
    for (args, page) in [
        (&["tree", "--nope"][..], "celis tree --help"),
        (&["frobnicate"][..], "celis help"),
    ] {
        let usage = repo.run(&dir, args, &[], "");
        assert_eq!(usage.status.code(), Some(2), "{args:?}");
        assert!(err(&usage).contains(page), "{args:?}: {}", err(&usage));
        assert_eq!(out(&usage), "");
    }
}

/// An unknown command names the known one nearest it, when two edits or fewer turn one into the
/// other.
#[test]
fn an_unknown_command_names_the_known_one_nearest_it() {
    let repo = Repo::new();
    let usage = repo.run(&repo.path(), &["decid"], &[], "");
    assert_eq!(usage.status.code(), Some(2));
    assert!(err(&usage).contains("\"decide\""), "{}", err(&usage));
}

/// Without `--json`, an error is one line on stderr: `celis: <message>`; with `--json` it is one object
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn without_json_an_error_is_one_line() {
    let repo = Repo::new();
    let dir = repo.path();
    let usage = repo.run(&dir, &["frobnicate"], &[], "");
    let message = err(&usage);
    assert_eq!(message.trim_end().lines().count(), 1, "{message:?}");
    assert!(message.starts_with("celis: "), "{message:?}");

    let json = repo.run(&dir, &["--json", "frobnicate"], &[], "");
    assert_eq!(json.status.code(), Some(2));
    let object: Value = serde_json::from_str(err(&json).trim()).expect("one JSON object");
    assert_eq!(object["error"], "usage");
    assert!(object["message"].is_string());
}

/// A code-1 error without `--json` is one line on stderr, `celis: <message>`.
#[test]
fn a_code_1_error_without_json_is_one_celis_line() {
    let repo = Repo::plain();
    let o = repo.run(
        &repo.path(),
        &["step", "done", "deadbeef", "--as-planned", "--", "p"],
        &[],
        "",
    );
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let message = err(&o);
    assert_eq!(message.matches('\n').count(), 1, "{message:?}");
    assert!(
        message.starts_with("celis: ") && message.ends_with('\n'),
        "{message:?}"
    );
    assert_eq!(out(&o), "");
}

/// Any other error exits 1, and with `--json` it is one object on stderr (`spec/02-cli.md`,
/// "Output and errors").
#[test]
fn any_other_error_exits_1() {
    let repo = Repo::plain();
    let other = repo.run(
        &repo.path(),
        &[
            "step",
            "done",
            &"0".repeat(64),
            "--json",
            "--as-planned",
            "--",
            "p",
        ],
        &[],
        "",
    );
    assert_eq!(other.status.code(), Some(1));
    let object: Value = serde_json::from_str(err(&other).trim()).expect("one JSON object");
    assert!(object["error"].is_string() && object["message"].is_string());
    assert_eq!(out(&other), "");
}

/// A command refuses an 8-character start two steps share, and takes a full id of either
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_command_refuses_an_8_character_start_two_steps_share() {
    let repo = Repo::plain();
    let by = &account();
    let (a, b) = sharing_a_start(&repo, by, "step", "step");
    assert_eq!(a.id[..8], b.id[..8]);
    assert_ne!(a.id, b.id);
    put(&repo, &a);
    put(&repo, &b);

    let args = ["step", "done", &a.id[..8], "--as-planned", "--", "p"];
    let refused = repo.run(&repo.path(), &args, &[], "");
    assert!(!refused.status.success());
    assert_eq!(out(&refused), "");

    let o = repo.run(
        &repo.path(),
        &["step", "done", &a.id, "--as-planned", "--", "p"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
}

/// A command refuses an 8-character start two steps share with `no-step`, or `no-trail` as a scope, and
/// two rules share with `no-decision`, under `--json` (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_command_refuses_an_8_character_start_two_trails_rules_or_steps_share() {
    let repo = Repo::plain();
    let project = (repo.project_id(), "repo".to_string());
    let project = (project.0.as_str(), project.1.as_str());
    let at = support::now() - 86400;

    let (g1, g2) = sharing_a_start_with(&account(), at, &step_tags(project));
    put(&repo, &g1);
    put(&repo, &g2);
    assert_eq!(g1.id[..8], g2.id[..8]);
    let scope = format!("trail:{}", &g1.id[..8]);
    for (args, name) in [
        (vec!["tree", "--json", &g1.id[..8]], "no-step"),
        (
            vec!["decide", "--json", "--scope", &scope, "--", "x"],
            "no-trail",
        ),
    ] {
        let o = repo.run(&repo.path(), &args, &[], "");
        support::refusal(&o, 1, name);
        assert_eq!(out(&o), "");
    }
    let o = repo.run(&repo.path(), &["tree", &g2.id], &[], "");
    assert!(o.status.success(), "{}", err(&o));

    let (d1, d2) = sharing_a_start_with(&account(), at, &rule_tags(project));
    put(&repo, &d1);
    put(&repo, &d2);
    assert_eq!(d1.id[..8], d2.id[..8]);
    let args = ["decide", "--json", "--replaces", &d1.id[..8], "--", "x"];
    let o = repo.run(&repo.path(), &args, &[], "");
    support::refusal(&o, 1, "no-decision");
    assert_eq!(out(&o), "");
    let args = ["decide", "--replaces", &d1.id, "--", "x"];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));

    let by = &account();
    let (a, b) = sharing_a_start(&repo, by, "step", "step");
    put(&repo, &a);
    put(&repo, &b);
    let args = [
        "step",
        "done",
        "--json",
        &a.id[..8],
        "--as-planned",
        "--",
        "p",
    ];
    let o = repo.run(&repo.path(), &args, &[], "");
    support::refusal(&o, 1, "no-step");
    assert_eq!(out(&o), "");
}

/// `step done` prints one JSON object a line with `--json` (`spec/02-cli.md`, "Output and
/// errors").
#[test]
fn step_done_prints_json_objects_with_json() {
    let repo = Repo::plain();
    let id = step_id(&repo, "one");
    let args = [
        "step",
        "done",
        "--json",
        &id,
        "--as-planned",
        "--",
        "it passes",
    ];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let lines = objects(&out(&o));
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].as_object().unwrap().len(), 1, "{}", lines[0]);
    assert!(is_id(lines[0]["id"].as_str().unwrap()));
}

/// An unknown command more than two edits from every known one names no "did you mean".
#[test]
fn an_unknown_command_far_from_every_known_one_names_no_suggestion() {
    let repo = Repo::plain();
    let usage = repo.run(&repo.path(), &["frobnicate"], &[], "");
    assert_eq!(usage.status.code(), Some(2));
    assert!(err(&usage).contains("frobnicate"), "{}", err(&usage));
    assert!(!err(&usage).contains("did you mean"), "{}", err(&usage));
}

/// With `--json`, each refusal names itself: `usage`, `no-decision`, `no-step` and `closed`, and
/// any other failure `error` (`spec/02-cli.md`, "Output and errors").
#[test]
fn each_refusal_names_itself_under_json() {
    let repo = Repo::plain();
    let dir = repo.path();
    let name = |args: &[&str]| -> String {
        let o = repo.run(&dir, args, &[], "");
        assert!(!o.status.success(), "{args:?}");
        let object: Value = serde_json::from_str(err(&o).trim()).expect("one JSON object");
        object["error"].as_str().unwrap().to_string()
    };
    let none = "0".repeat(64);
    let added = repo.run(&dir, &["step", "add", "--", "work"], &[], "");
    assert!(added.status.success(), "{}", err(&added));
    let id = out(&added).trim().to_string();

    assert_eq!(name(&["--json", "frobnicate"]), "usage");
    assert_eq!(
        name(&["--json", "decide", "--replaces", &none, "--", "x"]),
        "no-decision"
    );
    assert_eq!(
        name(&["--json", "step", "done", &none, "--as-planned", "--", "p"]),
        "no-step"
    );
    let done = repo.run(
        &dir,
        &["step", "done", &id, "--as-planned", "--", "p"],
        &[],
        "",
    );
    assert!(done.status.success(), "{}", err(&done));
    assert_eq!(
        name(&["--json", "step", "done", &id, "--as-planned", "--", "p"]),
        "closed"
    );
    let file = repo.home().join("not-a-folder");
    std::fs::write(&file, "x").unwrap();
    let broken = repo.run(
        &dir,
        &["--json", "decide", "--", "x"],
        &[("CELIS_HOME", file.to_str().unwrap())],
        "",
    );
    assert_eq!(broken.status.code(), Some(1), "{}", err(&broken));
    let object: Value = serde_json::from_str(err(&broken).trim()).expect("one JSON object");
    assert_eq!(object["error"], "error");
}

/// A command whose reader closes stdout stops with no message.
#[test]
fn a_closed_reader_stops_a_command_with_no_message() {
    let repo = Repo::plain();
    let o = repo.run(&repo.path(), &["step", "add", "--", "work"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_celis"))
        .args(["tree", "--all"])
        .current_dir(repo.path())
        .env("CELIS_HOME", repo.home())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let o = child.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stderr), "");
}

/// A write to a folder celis may not write, as in an agent's sandbox, fails as `error` with a message
/// that names the folder celis keeps its notes in.
#[test]
fn a_read_only_store_names_the_folder_to_let_the_agent_write() {
    use std::os::unix::fs::PermissionsExt;
    let repo = Repo::new();
    let home = repo.home();
    std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o555)).unwrap();
    let o = repo.run(
        &repo.path(),
        &["--json", "decide", "--", "When x: y."],
        &[],
        "",
    );
    std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let object: Value = serde_json::from_str(err(&o).trim()).expect("one JSON object");
    assert_eq!(object["error"], "error");
    let message = object["message"].as_str().unwrap();
    assert!(message.contains(home.to_str().unwrap()), "{message}");
    assert!(message.contains("sandbox"), "{message}");
}
