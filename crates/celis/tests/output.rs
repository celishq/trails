//! CIP-11, "Output and errors".

mod support;

use serde_json::Value;
use std::process::Output;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use support::{agent, err, finding, labelled, now, out, put, sharing_a_start, Repo, DAY};

fn objects(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).expect("one JSON object a line"))
        .collect()
}

const SESSION: [(&str, &str); 1] = [("CELIS_SESSION", "s1")];

fn written(repo: &Repo, extra: &[&str]) -> Output {
    let mut args = vec!["add", "--task", "t", "finding"];
    args.extend_from_slice(extra);
    args.extend(["--", "the note"]);
    repo.run(&repo.path(), &args, &SESSION, "")
}

fn id_of(repo: &Repo) -> String {
    let o = written(repo, &[]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// Asserts that the store holds a rating, kind 7, with id `rating` and an `e` tag naming `cairn`.
fn assert_rating_cites(repo: &Repo, rating: &str, cairn: &str) {
    let dir = repo.store().join("shared");
    let found = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Value>(&t).ok())
        .find(|n| n["id"] == rating)
        .expect("the printed id is a record in the store");
    assert_eq!(found["kind"], 7);
    assert!(found["tags"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t[0] == "e" && t[1] == cairn));
    assert_ne!(rating, cairn);
}

/// Each `list` line is `<id8> [<project name>] <content>`, a cairn of this project included, with no mark
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn list_prints_the_text_form_on_one_line() {
    let repo = Repo::new();
    let id = id_of(&repo);
    let name = objects(&out(&repo.run(&repo.path(), &["list", "--json"], &[], "")))[0]["project"]
        .as_str()
        .unwrap()
        .to_string();

    let text = out(&repo.run(&repo.path(), &["list"], &[], ""));
    assert_eq!(text.trim_end(), format!("{} [{name}] the note", &id[..8]));
    assert_eq!(text.lines().count(), 1, "{text}");
}

/// A `--json` list object carries `"score"`, a number
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn a_json_list_object_carries_a_score() {
    let repo = Repo::new();
    id_of(&repo);

    let objs = objects(&out(&repo.run(&repo.path(), &["list", "--json"], &[], "")));

    assert!(objs[0]["score"].is_number(), "{}", objs[0]);
}

/// `list` prints one JSON object a line with `--json`
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn list_prints_one_json_object_a_line() {
    let repo = Repo::new();
    let id = id_of(&repo);
    let dir = repo.path();

    let objs = objects(&out(&repo.run(&dir, &["list", "--json"], &[], "")));
    assert_eq!(objs.len(), 1);
    for key in [
        "id",
        "label",
        "agent",
        "created_at",
        "score",
        "anchor",
        "content",
    ] {
        assert!(objs[0].get(key).is_some(), "{key}");
    }
    assert_eq!(objs[0]["id"], id);
}

/// A cairn of 9000 bytes of content.
fn long(repo: &Repo) {
    let by = agent(repo, "w");
    let anchor = format!("project:{}", repo.project_id());
    let tags = vec![
        vec!["I".to_string(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["L".into(), "celis.cairn".into()],
        vec!["l".into(), "finding".into(), "celis.cairn".into()],
    ];
    let note = celis_wire::Note::write(by.key(), 1111u16, now() - 3600, tags, "a".repeat(9000));
    put(repo, &note.unwrap());
}

/// Past 8000 bytes of content, `list` ends with the line `note: <n> cairns, about <tokens> tokens`
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn past_8000_bytes_list_ends_with_the_note_line() {
    let repo = Repo::new();
    long(&repo);

    let text = out(&repo.run(&repo.path(), &["list"], &[], ""));

    assert_eq!(
        text.lines().last(),
        Some("note: 1 cairns, about 2250 tokens"),
        "{text}"
    );
}

/// With `--json`, the note is the object `{"note", "cairns", "tokens"}`
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn with_json_the_note_is_an_object_of_note_cairns_and_tokens() {
    let repo = Repo::new();
    long(&repo);

    let objs = objects(&out(&repo.run(&repo.path(), &["list", "--json"], &[], "")));

    let last = objs.last().unwrap();
    assert_eq!(last["cairns"], 1, "{last}");
    assert_eq!(last["tokens"], 2250, "{last}");
    assert!(last["note"].is_string(), "{last}");
}

/// The JSON form names the agent and the time of each cairn, which the one-line text form leaves out
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn the_json_form_names_the_agent_and_the_time() {
    let repo = Repo::new();
    let dir = repo.path();
    let old = agent(&repo, "old");
    let at = now() - 2 * DAY;
    let cairn = finding(&repo, &old, at);
    let mine = out(&repo.run(
        &dir,
        &["add", "--task", "t", "finding", "--", "the note"],
        &SESSION,
        "",
    ))
    .trim()
    .to_string();

    let objs = objects(&out(&repo.run(&dir, &["list", "--json"], &[], "")));
    assert_eq!(objs.len(), 2);
    let by = |id: &str| objs.iter().find(|o| o["id"] == id).unwrap().clone();
    assert_eq!(by(&cairn.id)["agent"], cairn.pubkey.npub());
    assert_eq!(by(&cairn.id)["created_at"], at);
    assert_ne!(by(&mine)["agent"], by(&cairn.id)["agent"]);
}

/// `rate` takes the first 8 characters of an id, when only one cairn's id starts with them
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn rate_takes_the_first_8_characters_of_an_id() {
    let repo = Repo::new();
    let id = id_of(&repo);
    let other = [("CELIS_SESSION", "s2")];
    let o = repo.run(&repo.path(), &["rate", &id[..8], "+"], &other, "");
    assert!(o.status.success(), "{}", err(&o));
    assert_rating_cites(&repo, out(&o).trim(), &id);
}

/// Every command accepts `--json`; `add` and `rate` print the id, or `{"id"}`; no command asks a
/// question (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn every_command_accepts_json_and_prints_the_id_without_asking() {
    let repo = Repo::new();
    let dir = repo.path();
    let id = id_of(&repo);
    assert!(
        id.len() == 64 && id.chars().all(|c| c.is_ascii_hexdigit()),
        "{id}"
    );

    let again = written(&repo, &["--json"]);
    let objs = objects(&out(&again));
    assert_eq!(objs.len(), 1);
    let again_id = objs[0]["id"].as_str().unwrap();
    assert_eq!(again_id.len(), 64);
    assert!(labelled(&repo, "finding").iter().any(|n| n.id == again_id));

    let other = [("CELIS_SESSION", "s2")];
    let rate = repo.run(&dir, &["rate", &id, "+", "--json"], &other, "");
    assert!(rate.status.success(), "{}", err(&rate));
    let printed = objects(&out(&rate))[0]["id"].as_str().unwrap().to_string();
    assert_rating_cites(&repo, &printed, &id);

    for args in [vec!["list", "--json"], vec!["help", "--json"]] {
        let o = repo.run(&dir, &args, &[], "");
        assert!(o.status.success(), "{args:?}");
        objects(&out(&o));
    }
}

/// A command run with its stdin left open finishes, and prints no question
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn a_command_with_stdin_left_open_finishes_and_asks_nothing() {
    let repo = Repo::new();
    let id = id_of(&repo);
    let runs: [&[&str]; 7] = [
        &["list"],
        &[
            "add",
            "--task",
            "t",
            "--",
            "When you note: write it, because the next run needs it.",
        ],
        &["rate", &id[..8], "+"],
        &["report", "--done"],
        &["show", &id[..8]],
        &["log"],
        &["stats"],
    ];
    for args in runs {
        let mut child = Command::new(env!("CARGO_BIN_EXE_celis"))
            .arg("cairn")
            .args(args)
            .current_dir(repo.path())
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("CELIS_HOME", repo.home())
            .env("CELIS_SESSION", "s2")
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
        assert_eq!(err(&o), "", "{args:?}");
        let asked = out(&o).lines().any(|l| l.trim_end().ends_with('?'));
        assert!(!asked, "{args:?}: {}", out(&o));
    }
}

/// A usage error names the narrowest help page that covers it (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn a_usage_error_names_the_narrowest_help_page() {
    let repo = Repo::new();
    let dir = repo.path();
    for (args, page) in [
        (&["list", "--nope"][..], "celis cairn list --help"),
        (&["frobnicate"][..], "celis cairn help"),
    ] {
        let usage = repo.run(&dir, args, &[], "");
        assert_eq!(usage.status.code(), Some(2), "{args:?}");
        assert!(err(&usage).contains(page), "{args:?}: {}", err(&usage));
        assert_eq!(out(&usage), "");
    }
}

/// An unknown command names the known one nearest it, when two edits or fewer turn one into the other
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn an_unknown_command_names_the_known_one_nearest_it() {
    let repo = Repo::new();
    let usage = repo.run(&repo.path(), &["lsit"], &[], "");
    assert_eq!(usage.status.code(), Some(2));
    assert!(err(&usage).contains("\"list\""), "{}", err(&usage));
}

/// Without `--json`, an error is one line on stderr: `celis: <message>`; with `--json` it is one object
/// (`spec/cairns/11-cli.md`, "Output and errors").
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

/// A code-1 error without `--json` is one line on stderr, `celis: <message>`
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn a_code_1_error_without_json_is_one_celis_line() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["rate", "deadbeef", "+"],
        &[("CELIS_SESSION", "s2")],
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

/// `init` with no terminal finishes without waiting and asks no question
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn init_with_no_terminal_asks_no_question() {
    for agent_args in [&["--for", "claude"][..], &[][..]] {
        for closed in [true, false] {
            let repo = Repo::new();
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_celis"));
            cmd.arg("init")
                .args(agent_args)
                .current_dir(repo.path())
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", repo.home())
                .env("CELIS_HOME", repo.home())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = if closed {
                cmd.stdin(Stdio::null()).spawn().unwrap()
            } else {
                cmd.stdin(Stdio::piped()).spawn().unwrap()
            };
            let held = child.stdin.take();
            let start = Instant::now();
            while child.try_wait().unwrap().is_none() {
                if start.elapsed() > Duration::from_secs(20) {
                    child.kill().unwrap();
                    panic!("init {agent_args:?} waited on stdin");
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            drop(held);
            let o = child.wait_with_output().unwrap();
            assert!(o.status.success(), "init {agent_args:?}: {}", err(&o));
            let text = format!("{}{}", out(&o), err(&o));
            assert!(!text.contains('?'), "{text}");
        }
    }
}

/// Any other error exits 1, and with `--json` it is one object on stderr (`spec/cairns/11-cli.md`,
/// "Output and errors").
#[test]
fn any_other_error_exits_1() {
    let repo = Repo::new();
    let env = [("CELIS_SESSION", "s2")];
    let other = repo.run(
        &repo.path(),
        &["rate", &"0".repeat(64), "+", "--json"],
        &env,
        "",
    );
    assert_eq!(other.status.code(), Some(1));
    let object: Value = serde_json::from_str(err(&other).trim()).expect("one JSON object");
    assert!(object["error"].is_string() && object["message"].is_string());
    assert_eq!(out(&other), "");
}

/// `rate` refuses an 8-character start two cairns share, and takes a full id of either
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn rate_refuses_an_8_character_start_two_cairns_share() {
    let repo = Repo::new();
    let by = agent(&repo, "s1");
    let (a, b) = sharing_a_start(&repo, &by, "finding", "finding");
    assert_eq!(a.id[..8], b.id[..8]);
    assert_ne!(a.id, b.id);
    put(&repo, &a);
    put(&repo, &b);
    let other = [("CELIS_SESSION", "s2")];

    let refused = repo.run(&repo.path(), &["rate", &a.id[..8], "+"], &other, "");
    assert!(!refused.status.success());
    assert_eq!(out(&refused), "");

    let o = repo.run(&repo.path(), &["rate", &a.id, "+"], &other, "");
    assert!(o.status.success(), "{}", err(&o));
    assert_rating_cites(&repo, out(&o).trim(), &a.id);
}

/// `log`, `show` and `stats` print only JSON objects, one a line, on stdout with `--json`
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn log_show_and_stats_print_only_json_objects_with_json() {
    let repo = Repo::new();
    let dir = repo.path();
    let id = id_of(&repo);
    let listed = repo.run(&dir, &["list"], &SESSION, "");
    assert!(listed.status.success(), "{}", err(&listed));

    for args in [
        vec!["log", "--json"],
        vec!["show", id.as_str(), "--json"],
        vec!["stats", "--json"],
    ] {
        let o = repo.run(&dir, &args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
        let text = out(&o);
        assert!(!text.trim().is_empty(), "{args:?}");
        for line in text.lines() {
            let value: Value = serde_json::from_str(line)
                .unwrap_or_else(|_| panic!("{args:?}: not JSON: {line:?}"));
            assert!(value.is_object(), "{args:?}: {line:?}");
        }
    }
}

/// An unknown command more than two edits from every known one names no "did you mean"
/// (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn an_unknown_command_far_from_every_known_one_names_no_suggestion() {
    let repo = Repo::new();
    let usage = repo.run(&repo.path(), &["frobnicate"], &[], "");
    assert_eq!(usage.status.code(), Some(2));
    assert!(err(&usage).contains("frobnicate"), "{}", err(&usage));
    assert!(!err(&usage).contains("did you mean"), "{}", err(&usage));
}

/// `list` names the project of each cairn, and `--json` names its project and task (`spec/cairns/11-cli.md`, "Output and errors").
#[test]
fn list_marks_a_cairn_of_another_project_with_its_name() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let args = [
        "add",
        "--task",
        "add a tier",
        "--",
        "When pricing, use cents.",
    ];
    let o = repo.run(&shop, &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).trim().to_string();
    let text = repo.run(&repo.path(), &["list"], &[], "");
    assert_eq!(
        out(&text),
        format!("{} [shop] When pricing, use cents.\n", &id[..8])
    );
    let there = repo.run(&shop, &["list"], &[], "");
    assert_eq!(
        out(&there),
        format!("{} [shop] When pricing, use cents.\n", &id[..8])
    );
    let json = repo.run(&repo.path(), &["list", "--json"], &[], "");
    let object: Value = serde_json::from_str(out(&json).trim()).unwrap();
    assert_eq!(object["project"], "shop");
    assert_eq!(object["task"], "add a tier");
}
