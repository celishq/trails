//! CIP-04, "Sessions".

mod support;

use celis_nostr::Note;
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
/// the ask's id first; with no session, it writes the trail alone (`spec/04-trails.md`,
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
/// (`spec/04-trails.md`, "Sessions").
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

/// The prompt hook reads `json` on stdin.
fn hook(repo: &Repo, json: serde_json::Value) -> std::process::Output {
    let o = support::prompt(repo, &repo.path(), &[], &json.to_string());
    assert!(o.status.success(), "{}", err(&o));
    o
}

/// `prompt` writes a hook's prompt as a prompt mark on the session's open trail, also while a step
/// works, its first line cut to 200 characters; with no such trail, it writes nothing
/// (`spec/04-trails.md`, "Sessions").
#[test]
fn prompt_writes_the_prompt_as_a_prompt_mark_on_the_sessions_trail() {
    let repo = Repo::new();
    let hook = |session: &str, prompt: &str| {
        let json = serde_json::json!({"session_id": session, "prompt": prompt}).to_string();
        support::prompt(&repo, &repo.path(), &[], &json)
    };
    let o = hook("s1", "before any trail");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "");
    let trail = start(&repo, "s1", "the trail");
    let o = hook("s1", "make it blue\nand more");
    assert!(o.status.success(), "{}", err(&o));
    let step = add(&repo, Some("s1"), "a step");
    let o = repo.run(&repo.path(), &["step", "start", &step[..8]], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let long = "x".repeat(300);
    assert!(hook("s1", &long).status.success());
    assert_eq!(out(&hook("s2", "another session")), "");
    assert_eq!(marks(&repo, "steer"), []);
    let mut prompts = marks(&repo, "prompt");
    prompts.sort();
    assert_eq!(
        prompts,
        [
            (trail.clone(), "make it blue".to_string()),
            (trail.clone(), "x".repeat(200))
        ]
    );
}

/// A trail asked twice is reached.
#[test]
fn a_trail_asked_twice_is_reached() {
    let repo = Repo::new();
    let written = ids(&repo, Some("s1"), "the trail");
    let trail = &written[1];
    hook(
        &repo,
        serde_json::json!({"session_id": "s1", "prompt": "and more"}),
    );
    support::finish(&repo, &written[0][..8]);
    support::finish(&repo, &trail[..8]);
    let head = objects(&repo, &["tree", &trail[..8]]);
    assert_eq!(head[0]["trail"]["state"], "done", "{head:?}");
}

/// `step add --new` starts a second trail, which takes the session's next steps
/// (`spec/04-trails.md`, "Sessions").
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
/// trail, its next steps go there (`spec/04-trails.md`, "Sessions").
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

/// `prompt` gives one line on stderr: search, join the session's trail, replace done work by its id in
/// `celis tree`, and keep a correction as a rule and a reaction as a pattern; a working step changes
/// nothing of it.
#[test]
fn the_prompt_hook_gives_one_line() {
    let repo = Repo::new();
    let hook = |prompt: &str| {
        let json = serde_json::json!({"session_id": "s1", "prompt": prompt}).to_string();
        support::prompt(&repo, &repo.path(), &[], &json)
    };
    let trail = start(&repo, "s1", "the trail");
    let line = format!("{}\n", note(true, Some(&trail[..8]), true));
    let o = hook("first");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), line);
    let step = add(&repo, Some("s1"), "Cache the price list");
    let o = repo.run(&repo.path(), &["step", "start", &step[..8]], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let o = hook("now make it blue");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), line);
}

/// The line `prompt` gives: a new ask runs `search` when the store holds a record, then joins the
/// session's trail `trail`, else a trail of `today`, else a new one; how done work is fixed; and how a
/// correction becomes a rule and a reaction a pattern.
fn note(search: bool, trail: Option<&str>, today: bool) -> String {
    let join = match (trail, today) {
        (Some(id), _) => {
            format!("add it under the session's trail {id} when it fits, else with --new")
        }
        (None, true) => {
            "add it --under a related trail of today that celis tree shows, else with --new"
                .to_string()
        }
        (None, false) => "add it with --new".to_string(),
    };
    let ask = match search {
        true => format!(
            "run celis search [<each file it will touch>]... -- \"<the ask>\" before you change a file, \
             then {join}"
        ),
        false => join,
    };
    let fix = match trail {
        Some(id) => format!(
            "; when it changes done work, add it with step add --replaces <the id of that step, as celis \
             tree {id} shows it>"
        ),
        None => String::new(),
    };
    format!(
        "celis: when this prompt is a new ask, {ask}{fix}; when it says always or never, or states a \
         convention, write a rule with celis decide before you change a file, with no --good or --bad, and \
         with --scope me when it holds in any project; when it reacts to a result, such as \"this one\" or \"too much\", write \
         a pattern with celis decide --good <what the person liked> --bad <what they did not>."
    )
}

/// A prompt with no open trail gives the hint to join a trail of today, of any project; with no trail
/// of today, it asks for a new trail.
#[test]
fn a_later_prompt_with_no_open_trail_hints_at_todays_trails() {
    let repo = Repo::new();
    let hook = |session: &str| {
        let json = serde_json::json!({"session_id": session, "prompt": "Add a filter"});
        let o = support::prompt(&repo, &repo.path(), &[], &json.to_string());
        assert!(o.status.success(), "{}", err(&o));
        err(&o)
    };
    assert_eq!(hook("s9"), format!("{}\n", note(false, None, false)));
    let shop = repo.other("shop");
    let o = repo.run(&shop, &["step", "add", "--", "a shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(hook("s9"), format!("{}\n", note(true, None, true)));
    let o = repo.run(
        &repo.path(),
        &["decide", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(hook("s8"), format!("{}\n", note(true, None, true)));
}

/// `prompt` writes and prints nothing for a prompt that is a task notification or an agent's message of
/// the harness, not the person's words (`spec/04-trails.md`, "Sessions").
#[test]
fn prompt_skips_a_harness_message() {
    let repo = Repo::new();
    start(&repo, "s1", "the trail");
    for prompt in [
        "<task-notification>\n<task-id>b1</task-id>\n</task-notification>",
        "<agent-message from=\"a1\">\nThe report.\n</agent-message>",
    ] {
        let json = serde_json::json!({"session_id": "s1", "prompt": prompt}).to_string();
        let o = support::prompt(&repo, &repo.path(), &[], &json);
        assert!(o.status.success(), "{}", err(&o));
        assert_eq!(out(&o), "", "{prompt}");
        assert_eq!(err(&o), "", "{prompt}");
    }
    assert!(marks(&repo, "prompt").is_empty());
}

/// With no open trail in the session, `prompt` writes nothing, and prints on stderr the search line once
/// the store holds any record, then how to add a new trail; a session start or a compaction prints
/// nothing.
#[test]
fn a_prompt_with_no_open_trail_asks_for_search_and_a_new_trail() {
    let repo = Repo::new();
    let hook = |json: serde_json::Value| {
        let o = support::prompt(&repo, &repo.path(), &[], &json.to_string());
        assert!(o.status.success(), "{}", err(&o));
        assert_eq!(out(&o), "");
        err(&o)
    };
    let ask = serde_json::json!({"session_id": "s9", "prompt": "Build a Vendors table"});
    assert_eq!(hook(ask.clone()), format!("{}\n", note(false, None, false)));
    let o = repo.run(
        &repo.path(),
        &["decide", "--", "When you test: set SHELF_OFFLINE=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(hook(ask.clone()), format!("{}\n", note(true, None, false)));
    for source in ["startup", "compact"] {
        let start = serde_json::json!({"session_id": "s9", "source": source});
        assert_eq!(hook(start), "");
    }
    assert_eq!(hook(ask), format!("{}\n", note(true, None, false)));
    assert!(marks(&repo, "prompt").is_empty());
}

/// The steps `tree <trail>` shows.
fn under(repo: &Repo, trail: &str) -> Vec<String> {
    objects(repo, &["tree", &trail[..8]])
        .iter()
        .filter_map(|o| o["step"]["id"].as_str().map(str::to_string))
        .collect()
}

/// `step add` takes its session from `--session`, else from the first set variable of
/// `CLAUDE_CODE_SESSION_ID`, `CODEX_THREAD_ID` and `CELIS_SESSION_ID` (`spec/04-trails.md`,
/// "Sessions").
#[test]
fn a_session_comes_from_the_flag_else_the_first_set_variable() {
    let repo = Repo::new();
    let add = |env: &[(&str, &str)], args: &[&str]| {
        let args = [&["step", "add"], args, &["--", "a step"]].concat();
        let o = repo.run(&repo.path(), &args, env, "");
        assert!(o.status.success(), "{}", err(&o));
        out(&o).lines().next().unwrap().to_string()
    };
    let trail = ids(&repo, Some("c1"), "the trail").remove(1);
    let steps = [
        add(&[], &["--session", "c1"]),
        add(&[("CODEX_THREAD_ID", "c1")], &[]),
        add(&[("CELIS_SESSION_ID", "c1")], &[]),
        add(&[("CELIS_SESSION_ID", "c9")], &["--session", "c1"]),
        add(
            &[("CLAUDE_CODE_SESSION_ID", "c1"), ("CODEX_THREAD_ID", "c9")],
            &[],
        ),
    ];
    let found = under(&repo, &trail);
    for step in steps {
        assert!(found.contains(&step), "{step} not in {found:?}");
    }
}

/// `prompt` takes the session from `conversation_id` when the hook has one, else from `session_id`
/// (`spec/02-cli.md`, "Prompt").
#[test]
fn prompt_takes_the_conversation_id_first() {
    let repo = Repo::new();
    let trail = start(&repo, "c1", "the trail");
    hook(
        &repo,
        serde_json::json!({"conversation_id": "c1", "session_id": "c9", "prompt": "make it blue"}),
    );
    assert_eq!(
        marks(&repo, "prompt"),
        [(trail, "make it blue".to_string())]
    );
}

/// A session's first `step add` says on stderr which printed id is the ask and which the new trail.
#[test]
fn a_sessions_first_ask_names_its_ids() {
    let repo = Repo::new();
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--", "make the badges orange"],
        &env,
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let ids: Vec<String> = out(&o).lines().map(str::to_string).collect();
    let line = format!(
        "celis: {} is the ask, under its new trail {}.",
        &ids[0][..8],
        &ids[1][..8]
    );
    assert!(err(&o).contains(&line), "{}", err(&o));
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--", "now the buttons"],
        &env,
        "",
    );
    assert!(!err(&o).contains("is the ask"), "{}", err(&o));
}

/// A session's first prompt that names a step, as a worker gets it, writes and prints nothing
/// (`spec/02-cli.md`, "Prompt").
#[test]
fn a_first_prompt_that_names_its_step_prints_nothing() {
    let repo = Repo::new();
    let ask = add(&repo, Some("s1"), "the trail");
    let said = format!("Fix it. celis step {}", &ask[..8]);
    let json = serde_json::json!({"session_id": "w1", "prompt": said});
    let o = support::prompt(&repo, &repo.path(), &[], &json.to_string());
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "");
    assert_eq!(err(&o), "");
}
