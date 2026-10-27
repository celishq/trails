//! CIP-04, "Sessions".

use crate::support;

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

/// While a session has a current trail, open or done less than 30 minutes ago, a step with no `--under`,
/// `--replaces` or `--new` is refused as a usage error that names it; `--new` adds a new trail, and so
/// does a step with no session (`spec/02-cli.md`, "Step"; `spec/04-trails.md`, "Sessions").
#[test]
fn a_step_beside_the_sessions_current_trail_needs_under_or_new() {
    let repo = Repo::new();
    let first = add(&repo, Some("s1"), "make the badges orange");
    let refused = || {
        let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
        let o = repo.run(
            &repo.path(),
            &["step", "add", "--", "the buttons"],
            &env,
            "",
        );
        assert_eq!(o.status.code(), Some(2), "{}", err(&o));
        assert!(err(&o).contains(&first[..8]), "{}", err(&o));
    };
    refused();
    support::finish(&repo, &first[..8]);
    refused();
    let next = run_add(&repo, Some("s1"), &[], &["--new", "--", "the buttons"]).remove(0);
    let none = add(&repo, None, "no session");
    for id in [&first, &next, &none] {
        let tree = objects(&repo, &["tree", &id[..8]]);
        assert_eq!(tree.len(), 1, "{tree:?}");
        assert_eq!(tree[0]["trail"]["id"], id.as_str(), "{tree:?}");
    }
}

/// A session whose trail was done 30 minutes ago or more has no current trail, so a step with no
/// `--under` is a new trail (`spec/04-trails.md`, "Sessions").
#[test]
fn a_trail_done_30_minutes_ago_is_not_current() {
    use support::{account, mark_note, now, put, step_tags};
    let repo = Repo::new();
    let old = now() - 31 * 60;
    let mut tags = step_tags((&repo.project_id(), "repo"));
    tags.push(vec!["session".into(), "s1".into()]);
    let trail = Note::write(&account(), 1111u16, old, tags, "the old trail").unwrap();
    put(&repo, &trail);
    mark_note(&repo, &account(), "done", &trail.id, "proof", old);
    let next = add(&repo, Some("s1"), "a new ask");
    let tree = objects(&repo, &["tree", &next[..8]]);
    assert_eq!(tree[0]["trail"]["id"], next.as_str(), "{tree:?}");
}

/// A trail carries its session, and `--under` joins another session's trail (`spec/04-trails.md`,
/// "Sessions").
#[test]
fn a_trail_carries_its_session_and_under_joins_another_sessions_trail() {
    let repo = Repo::new();
    let trail = add(&repo, Some("s1"), "make the badges orange");
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
    let tree = objects(&repo, &["tree", &trail[..8]]);
    assert_eq!(tree[0]["trail"]["session"], "s1", "{tree:?}");
    assert_eq!(tree[1]["step"]["id"], joined.as_str(), "{tree:?}");
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
/// works, its first line cut to 200 characters (`spec/04-trails.md`, "Sessions").
#[test]
fn prompt_writes_the_prompt_as_a_prompt_mark_on_the_sessions_trail() {
    let repo = Repo::new();
    let hook = |session: &str, prompt: &str| {
        let json = serde_json::json!({"session_id": session, "prompt": prompt}).to_string();
        support::prompt(&repo, &repo.path(), &[], &json)
    };
    let trail = add(&repo, Some("s1"), "the trail");
    let o = hook("s1", "make it blue\nand more");
    assert!(o.status.success(), "{}", err(&o));
    let step = support::added(&repo, "s1", &["--under", &trail[..8], "--", "a step"]).remove(0);
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

/// A prompt with no open trail goes on the trail of the session's next `step add`, once, and not on
/// another session's trail; the learn brief of that ask gives it (`spec/04-trails.md`, "Sessions").
#[test]
fn a_prompt_with_no_open_trail_goes_on_the_trail_of_the_sessions_next_step() {
    let repo = Repo::new();
    let json =
        serde_json::json!({"session_id": "s1", "prompt": "From now on, add type hints\nand more"});
    hook(&repo, json);
    add(&repo, Some("s2"), "another session");
    let written = ids(&repo, Some("s1"), "the trail");
    support::added(&repo, "s1", &["--new", "--", "another ask"]);
    assert_eq!(
        marks(&repo, "prompt"),
        [(
            written[0].clone(),
            "From now on, add type hints".to_string()
        )]
    );
    let done = [
        "step",
        "done",
        &written[0][..8],
        "--as-planned",
        "--by",
        "person",
        "--",
        "p",
    ];
    let o = repo.run(&repo.path(), &done, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        err(&o).contains("\n  asked   From now on, add type hints\n"),
        "{}",
        err(&o)
    );
}

/// A trail asked twice is reached.
#[test]
fn a_trail_asked_twice_is_reached() {
    let repo = Repo::new();
    let trail = add(&repo, Some("s1"), "the trail");
    hook(
        &repo,
        serde_json::json!({"session_id": "s1", "prompt": "and more"}),
    );
    support::finish(&repo, &trail[..8]);
    let head = objects(&repo, &["tree", &trail[..8]]);
    assert_eq!(head[0]["trail"]["state"], "done", "{head:?}");
}

/// `step add --new` adds a new trail with no notice (`spec/02-cli.md`, "Step").
#[test]
fn add_new_adds_a_trail_with_no_notice() {
    let repo = Repo::new();
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--new", "--", "second"],
        &env,
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(!err(&o).contains("deprecated"), "{}", err(&o));
    let second = out(&o);
    assert_eq!(second.lines().count(), 1, "{second}");
    let head = objects(&repo, &["tree", &second[..8]]);
    assert_eq!(head[0]["trail"]["id"], second.trim(), "{head:?}");
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
/// trail, its prompt goes there (`spec/04-trails.md`, "Sessions").
#[test]
fn a_sessions_open_trail_is_the_trail_of_its_newest_step() {
    let repo = Repo::new();
    let first = add(&repo, Some("s1"), "the trail");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    add(&repo, Some("s2"), "its own");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    run_add(
        &repo,
        Some("s2"),
        &[],
        &["--under", &first[..8], "--", "a part"],
    );
    hook(
        &repo,
        serde_json::json!({"session_id": "s2", "prompt": "and more"}),
    );
    assert_eq!(marks(&repo, "prompt"), [(first, "and more".to_string())]);
}

/// `prompt` gives one line on stderr: go on with the session's current ask under its id, add a new ask
/// with `--new`, and replace done work by its id in `celis tree`; a working step and the ask's done
/// change nothing of it.
#[test]
fn the_prompt_hook_gives_one_line() {
    let repo = Repo::new();
    let hook = |prompt: &str| {
        let json = serde_json::json!({"session_id": "s1", "prompt": prompt}).to_string();
        support::prompt(&repo, &repo.path(), &[], &json)
    };
    let trail = add(&repo, Some("s1"), "the trail");
    let line = format!("{}\n", note(true, Some(&trail[..8])));
    let o = hook("first");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), line);
    let step = run_add(
        &repo,
        Some("s1"),
        &[],
        &["--under", &trail[..8], "--", "Cache the price list"],
    );
    let o = repo.run(&repo.path(), &["step", "start", &step[0][..8]], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let o = hook("now make it blue");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), line);
    support::finish(&repo, &step[0][..8]);
    support::finish(&repo, &trail[..8]);
    let o = hook("now commit it");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), line);
}

/// The line `prompt` gives: a prompt that goes on with the current ask `current` adds steps under it,
/// and a new ask is a new step; how done work is fixed; and, when the store holds a record, when a
/// pattern is read.
fn note(held: bool, current: Option<&str>) -> String {
    let ask = match current {
        Some(id) => format!(
            "when this prompt goes on with the ask {id}, add its steps --under {id}; when it is a new ask, \
             add it with celis step add --new"
        ),
        None => "when this prompt is a new ask, add it with celis step add".to_string(),
    };
    let load = match held {
        true => " When a choice of the work matches a When of the index, read that pattern with celis \
                 patterns <id> before you choose, and name it with --uses <id> on the step of that choice.",
        false => "",
    };
    format!(
        "celis: {ask}; when it changes done work, add it with step add --replaces <the id of that step, as \
         celis tree shows it>.{load}"
    )
}

/// `prompt` writes and prints nothing for a prompt that is a task notification or an agent's message of
/// the harness, not the person's words (`spec/04-trails.md`, "Sessions").
#[test]
fn prompt_skips_a_harness_message() {
    let repo = Repo::new();
    add(&repo, Some("s1"), "the trail");
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

/// With no open trail in the session, `prompt` writes nothing, and prints on stderr how to add a new ask,
/// then when to read a pattern once the store holds any record; a session start or a compaction prints
/// nothing.
#[test]
fn a_prompt_with_no_open_trail_asks_for_a_new_trail() {
    let repo = Repo::new();
    let hook = |json: serde_json::Value| {
        let o = support::prompt(&repo, &repo.path(), &[], &json.to_string());
        assert!(o.status.success(), "{}", err(&o));
        assert_eq!(out(&o), "");
        err(&o)
    };
    let ask = serde_json::json!({"session_id": "s9", "prompt": "Build a Vendors table"});
    assert_eq!(hook(ask.clone()), format!("{}\n", note(false, None)));
    let o = repo.run(
        &repo.path(),
        &["pattern", "--", "When you test: set SHELF_OFFLINE=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(hook(ask.clone()), format!("{}\n", note(true, None)));
    for source in ["startup", "compact"] {
        let start = serde_json::json!({"session_id": "s9", "source": source});
        assert_eq!(hook(start), "");
    }
    assert_eq!(hook(ask), format!("{}\n", note(true, None)));
    assert!(marks(&repo, "prompt").is_empty());
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
    for step in steps {
        let tree = objects(&repo, &["tree", &step[..8]]);
        assert_eq!(tree[0]["trail"]["session"], "c1", "{tree:?}");
    }
}

/// `prompt` takes the session from `conversation_id` when the hook has one, else from `session_id`
/// (`spec/02-cli.md`, "Prompt").
#[test]
fn prompt_takes_the_conversation_id_first() {
    let repo = Repo::new();
    let trail = add(&repo, Some("c1"), "the trail");
    hook(
        &repo,
        serde_json::json!({"conversation_id": "c1", "session_id": "c9", "prompt": "make it blue"}),
    );
    assert_eq!(
        marks(&repo, "prompt"),
        [(trail, "make it blue".to_string())]
    );
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

/// A step names its agent: `claude` from `CLAUDE_CODE_SESSION_ID`, `codex` from `CODEX_THREAD_ID`,
/// else `CELIS_AGENT`, and none with none of them (`spec/04-trails.md`, "Sessions").
#[test]
fn a_step_names_the_agent_its_session_variable_gives() {
    let repo = Repo::new();
    let cases = [
        ("CLAUDE_CODE_SESSION_ID", Some("claude")),
        ("CODEX_THREAD_ID", Some("codex")),
        ("CELIS_AGENT", Some("gemini")),
        ("", None),
    ];
    for (n, (name, agent)) in cases.into_iter().enumerate() {
        let env: Vec<(&str, &str)> = [(name, "gemini")]
            .into_iter()
            .filter(|(name, _)| !name.is_empty())
            .collect();
        let env = env.as_slice();
        let text = format!("step {n}");
        let o = repo.run(
            &repo.path(),
            &["step", "add", "--new", "--", &text],
            env,
            "",
        );
        assert!(o.status.success(), "{}", err(&o));
        let step = support::labelled(&repo, "step")
            .into_iter()
            .find(|s| s.content == text)
            .unwrap();
        assert_eq!(step.tag("agent"), agent, "{env:?}");
    }
}
