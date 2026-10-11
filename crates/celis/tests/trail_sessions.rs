//! CIP-14, "Sessions".

mod support;

use celis_wire::Note;
use support::{err, objects, out, Repo};

/// The ids `step add -- <text>` prints in the session `session`, or in none.
fn ids(repo: &Repo, session: Option<&str>, text: &str) -> Vec<String> {
    let env: Vec<(&str, &str)> = session
        .map(|s| ("CLAUDE_CODE_SESSION_ID", s))
        .into_iter()
        .collect();
    let o = repo.run(&repo.path(), &["step", "add", "--", text], &env, "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

/// The id `step add -- <text>` prints first in the session `session`, or in none.
fn add(repo: &Repo, session: Option<&str>, text: &str) -> String {
    ids(repo, session, text).remove(0)
}

/// The trail of a session's first ask: the second id `step add` prints.
fn start(repo: &Repo, session: &str, text: &str) -> String {
    ids(repo, Some(session), text).remove(1)
}

/// A session's first `step add` writes the trail and the ask under it with the same text, and prints
/// the ask's id first; with no session, it writes the trail alone (`spec/trails/14-trails.md`,
/// "Sessions").
#[test]
fn a_sessions_first_ask_is_a_step_under_its_trail() {
    let repo = Repo::new();
    let written = ids(&repo, Some("s1"), "make the badges orange");
    assert_eq!(written.len(), 2, "{written:?}");
    let tree = objects(&repo, &["tree", &written[1][..8]]);
    assert_eq!(
        tree[0]["trail"]["content"], "make the badges orange",
        "{tree:?}"
    );
    assert_eq!(tree[1]["step"]["id"], written[0].as_str(), "{tree:?}");
    assert_eq!(
        tree[1]["step"]["content"], "make the badges orange",
        "{tree:?}"
    );
    let o = repo.run(
        &repo.path(),
        &["step", "done", &written[0][..8], "--as-planned", "--", "p"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let next = add(&repo, Some("s1"), "now the buttons too");
    let tree = objects(&repo, &["tree", &written[1][..8]]);
    assert_eq!(tree[2]["step"]["id"], next.as_str(), "{tree:?}");
    assert_eq!(ids(&repo, None, "no session").len(), 1);
}

/// A trail carries its session, and a session's next step with no `--under` goes under its trail; a
/// session with no trail, or no session, starts a trail, and `--under` joins another session's trail
/// (`spec/trails/14-trails.md`, "Sessions").
#[test]
fn a_sessions_steps_go_under_its_trail_and_a_session_with_no_trail_starts_one() {
    let repo = Repo::new();
    let trail = start(&repo, "s1", "make the badges orange");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let next = add(&repo, Some("s1"), "now the buttons too");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let other = ids(&repo, Some("s2"), "another session");
    let o = repo.run(
        &repo.path(),
        &[
            "step",
            "add",
            "--under",
            &trail[..8],
            "--",
            "joins on purpose",
        ],
        &[("CLAUDE_CODE_SESSION_ID", "s3")],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let joined = out(&o).trim().to_string();
    let none = add(&repo, None, "no session");
    let tree = objects(&repo, &["tree", &trail[..8]]);
    assert_eq!(tree[0]["trail"]["session"], "s1", "{tree:?}");
    assert_eq!(tree[2]["step"]["id"], next.as_str(), "{tree:?}");
    assert_eq!(tree[3]["step"]["id"], joined.as_str(), "{tree:?}");
    assert_eq!(other.len(), 2, "{other:?}");
    let own = objects(&repo, &["tree", &other[1][..8]]);
    assert_eq!(own[0]["trail"]["session"], "s2", "{own:?}");
    let head = objects(&repo, &["tree", &none[..8]]);
    assert!(head[0].get("trail").is_some(), "{head:?}");
}

/// The marks labelled `label` in the store, as (the step they name, their text).
fn marks(repo: &Repo, label: &str) -> Vec<(String, String)> {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .filter(|n| n.kind == 1985 && n.tag("l") == Some(label))
        .map(|n| (n.tag("e").unwrap_or("").to_string(), n.content.clone()))
        .collect()
}

/// The hook `hook` reads `json` on stdin.
fn hook(repo: &Repo, hook: &str, json: serde_json::Value) -> std::process::Output {
    let o = repo.run(&repo.path(), &[hook], &[], &json.to_string());
    assert!(o.status.success(), "{}", err(&o));
    o
}

/// `prompt` writes a hook's prompt as a steer on the oldest working step of the session's open trail,
/// else as a prompt mark on the trail, its first line cut to 200 characters; with no such trail, it writes
/// nothing (`spec/trails/14-trails.md`, "Sessions").
#[test]
fn prompt_writes_the_prompt_as_a_steer_or_a_prompt_mark_on_the_sessions_trail() {
    let repo = Repo::new();
    let hook = |session: &str, prompt: &str| {
        let json = serde_json::json!({"session_id": session, "prompt": prompt}).to_string();
        repo.run(&repo.path(), &["prompt"], &[], &json)
    };
    let o = hook("s1", "before any trail");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "");
    let trail = start(&repo, "s1", "the trail");
    let o = hook("s1", "make it blue\nand more");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim().len(), 64, "{}", out(&o));
    let step = add(&repo, Some("s1"), "a step");
    let o = repo.run(&repo.path(), &["step", "start", &step[..8]], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let long = "x".repeat(300);
    assert!(hook("s1", &long).status.success());
    assert_eq!(out(&hook("s2", "another session")), "");
    assert_eq!(marks(&repo, "steer"), [(step.clone(), "x".repeat(200))]);
    assert_eq!(
        marks(&repo, "prompt"),
        [(trail.clone(), "make it blue".to_string())]
    );
}

/// A prompt mark is no steer: a trail asked twice is still reached alone (`spec/trails/14-trails.md`,
/// "Sessions").
#[test]
fn a_prompt_mark_is_no_steer() {
    let repo = Repo::new();
    let written = ids(&repo, Some("s1"), "the trail");
    let trail = &written[1];
    hook(
        &repo,
        "prompt",
        serde_json::json!({"session_id": "s1", "prompt": "and more"}),
    );
    support::finish(&repo, &written[0][..8]);
    support::finish(&repo, &trail[..8]);
    let head = objects(&repo, &["tree", &trail[..8]]);
    assert_eq!(head[0]["trail"]["state"], "reached alone", "{head:?}");
}

/// `step add --new` starts a second trail, which takes the session's next steps
/// (`spec/trails/14-trails.md`, "Sessions").
#[test]
fn new_starts_a_trail_that_takes_the_sessions_next_steps() {
    let repo = Repo::new();
    ids(&repo, Some("s1"), "the trail");
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--new", "--", "second"],
        &env,
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let second = out(&o).lines().nth(1).unwrap_or("").to_string();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let third = add(&repo, Some("s1"), "under the second");
    let head = objects(&repo, &["tree", &second[..8]]);
    assert!(head[0].get("trail").is_some(), "{head:?}");
    assert_eq!(head[2]["step"]["id"], third.as_str(), "{head:?}");
}

/// `step add` in the session `session` with `env` set too, as `args`; the ids it prints.
fn run_add(repo: &Repo, session: Option<&str>, env: &[(&str, &str)], args: &[&str]) -> Vec<String> {
    let mut all: Vec<(&str, &str)> = env.to_vec();
    all.extend(session.map(|s| ("CLAUDE_CODE_SESSION_ID", s)));
    let mut line = vec!["step", "add"];
    line.extend(args);
    let o = repo.run(&repo.path(), &line, &all, "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

/// A session's open trail is the trail of its newest step: after a step `--under` another session's
/// trail, its next steps go there (`spec/trails/14-trails.md`, "Sessions").
#[test]
fn a_sessions_open_trail_is_the_trail_of_its_newest_step() {
    let repo = Repo::new();
    let first = ids(&repo, Some("s1"), "the trail")[1].clone();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let other = run_add(&repo, Some("s2"), &[], &["--new", "--", "its own"])[1].clone();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    run_add(
        &repo,
        Some("s2"),
        &[],
        &["--under", &first[..8], "--", "a part"],
    );
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let next = add(&repo, Some("s2"), "another part");
    let head = objects(&repo, &["tree", &first[..8]]);
    assert_eq!(head[3]["step"]["id"], next.as_str(), "{head:?}");
    let own = objects(&repo, &["tree", &other[..8]]);
    assert_eq!(own.len(), 2, "{own:?}");
}

/// With `CELIS_TRAIL`, `step add` with no `--under` goes under that step, and the session's next steps
/// follow it; with no session, the step goes there too (`spec/trails/14-trails.md`, "Sessions").
#[test]
fn celis_trail_names_where_new_steps_go() {
    let repo = Repo::new();
    let trail = ids(&repo, Some("s1"), "the trail")[1].clone();
    let env = [("CELIS_TRAIL", &trail[..8])];
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let first = run_add(&repo, Some("s2"), &env, &["--", "from s2"]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let second = add(&repo, Some("s2"), "s2 again");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let third = run_add(&repo, None, &env, &["--", "no session"]);
    let head = objects(&repo, &["tree", &trail[..8]]);
    let under: Vec<&str> = head[2..]
        .iter()
        .map(|o| o["step"]["id"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(under, [&first[0], &second, &third[0]], "{head:?}");
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--json", "--", "x"],
        &[("CELIS_TRAIL", "deadbeef")],
        "",
    );
    support::refusal(&o, 1, "no-step");
}

/// `prompt` names the working step of the session's trail on stderr, and asks for a step under it when
/// the next work does not fit; with no working step, it names none (`spec/trails/14-trails.md`,
/// "Sessions").
#[test]
fn the_prompt_hook_names_the_working_step() {
    let repo = Repo::new();
    let hook = |prompt: &str| {
        let json = serde_json::json!({"session_id": "s1", "prompt": prompt}).to_string();
        repo.run(&repo.path(), &["prompt"], &[], &json)
    };
    start(&repo, "s1", "the trail");
    let o = hook("first");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), "");
    let step = add(&repo, Some("s1"), "Cache the price list");
    let o = repo.run(&repo.path(), &["step", "start", &step[..8]], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let o = hook("now make it blue");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        err(&o).trim(),
        format!(
            "celis: you work on {} \"Cache the price list\"; when the next work does not fit it, \
             add a step under it first, or replace it with step add --replaces.",
            &step[..8]
        )
    );
}

/// `prompt` writes and prints nothing for a prompt that is a task notification of the harness, not the
/// person's words (`spec/trails/14-trails.md`, "Sessions").
#[test]
fn prompt_skips_a_task_notification() {
    let repo = Repo::new();
    start(&repo, "s1", "the trail");
    let json = serde_json::json!({"session_id": "s1",
        "prompt": "<task-notification>\n<task-id>b1</task-id>\n</task-notification>"})
    .to_string();
    let o = repo.run(&repo.path(), &["prompt"], &[], &json);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "");
    assert!(marks(&repo, "prompt").is_empty());
}
