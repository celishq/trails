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

/// The hook `hook` reads `json` on stdin.
fn hook(repo: &Repo, hook: &str, json: serde_json::Value) -> std::process::Output {
    let o = repo.run(&repo.path(), &[hook], &[], &json.to_string());
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
        "prompt",
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

/// `prompt` names the working step of the session's trail on stderr, asks for a step under it when the
/// next work does not fit, and gives the line a task for another agent carries; with no working step, it names none; either way it ends with the hint to
/// join a trail.
#[test]
fn the_prompt_hook_names_the_working_step() {
    let repo = Repo::new();
    let hook = |prompt: &str| {
        let json = serde_json::json!({"session_id": "s1", "prompt": prompt}).to_string();
        repo.run(&repo.path(), &["prompt"], &[], &json)
    };
    let trail = start(&repo, "s1", "the trail");
    let o = hook("first");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), format!("{}\n", joins(Some(&trail))));
    let step = add(&repo, Some("s1"), "Cache the price list");
    let o = repo.run(&repo.path(), &["step", "start", &step[..8]], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let o = hook("now make it blue");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        err(&o).lines().collect::<Vec<_>>(),
        [
            format!(
                "celis: you work on {} \"Cache the price list\"; when the next work does not fit it, \
                 add a step under it first, or replace it with step add --replaces. To hand work to \
                 another agent, add a step for it, and put in its task: celis step <id>: run celis \
                 step start <id> first and celis step done <id> last.",
                &step[..8]
            ),
            joins(Some(&trail)),
        ]
    );
}

/// `prompt` names every done step of the session's trail, its first ask too, and says a change to done
/// work replaces it.
#[test]
fn the_prompt_hook_names_the_done_steps() {
    let repo = Repo::new();
    let first = ids(&repo, Some("s1"), "make the badges orange");
    let (ask, trail) = (&first[0], &first[1]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let part = add(&repo, Some("s1"), "now the buttons too");
    for id in [ask, &part] {
        let o = repo.run(
            &repo.path(),
            &["step", "done", &id[..8], "--as-planned", "--", "p"],
            &[],
            "",
        );
        assert!(o.status.success(), "{}", err(&o));
    }
    let json = serde_json::json!({"session_id": "s1", "prompt": "make them red"}).to_string();
    let o = repo.run(&repo.path(), &["prompt"], &[], &json);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        err(&o).lines().collect::<Vec<_>>(),
        [
            "celis: when this prompt changes done work, add it with step add --replaces <its id>, \
             not as a new ask; the done steps:"
                .to_string(),
            format!("  {} \"make the badges orange\"", &ask[..8]),
            format!("  {} \"now the buttons too\"", &part[..8]),
            joins(Some(trail)),
        ]
    );
}

/// The hint `prompt` gives for a new ask: join the session's open trail `open` when it fits, else a
/// related trail of today, else a new one.
fn joins(open: Option<&str>) -> String {
    match open {
        Some(id) => format!(
            "celis: when this prompt is a new ask, add it under the session's trail {}, when it fits; \
             else run celis tree and add it --under a related trail of today; else add it --new. {RULES}",
            &id[..8]
        ),
        None => format!(
            "celis: when this prompt is a new ask, run celis tree and add it --under a related \
             trail of today; else start a new trail. {RULES}"
        ),
    }
}

/// The line of a prompt with no trail to join: an empty store, or no trail of today.
const NEW: &str = "celis: when this prompt is a new ask, add it with --new. When this prompt states how the \
                   work must be done (review feedback, a convention, a correction), write each point as a \
                   rule with celis decide before you change a file; give it --scope me when it holds for that \
                   kind of work in any project.";

/// The sentence of a new ask's line that asks to write the rules the prompt states.
const RULES: &str = "When this prompt states how the work must be done (review feedback, a \
                     convention, a correction), write each point as a rule with celis decide before you \
                     change a file; give it --scope me when it holds for that kind of work in any project.";

/// After the past agent was asked for, a later prompt with no open trail gives the hint to join a trail
/// of today, of any project; with no trail of today, it asks for a new trail.
#[test]
fn a_later_prompt_with_no_open_trail_hints_at_todays_trails() {
    let repo = Repo::new();
    let hook = |session: &str| {
        let json = serde_json::json!({"session_id": session, "prompt": "Add a filter"});
        let o = repo.run(&repo.path(), &["prompt"], &[], &json.to_string());
        assert!(o.status.success(), "{}", err(&o));
        err(&o)
    };
    assert_eq!(hook("s9"), format!("{NEW}\n"));
    let shop = repo.other("shop");
    let o = repo.run(&shop, &["step", "add", "--", "a shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(hook("s9").contains("celis:past agent"));
    assert_eq!(hook("s9"), format!("{}\n", joins(None)));
    let o = repo.run(
        &repo.path(),
        &["decide", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(hook("s8").contains("celis:past agent"));
    assert_eq!(hook("s8"), format!("{}\n", joins(None)));
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
        let o = repo.run(&repo.path(), &["prompt"], &[], &json);
        assert!(o.status.success(), "{}", err(&o));
        assert_eq!(out(&o), "", "{prompt}");
        assert_eq!(err(&o), "", "{prompt}");
    }
    assert!(marks(&repo, "prompt").is_empty());
}

/// With no open trail in the session, `prompt` writes nothing, and prints on stderr one line that asks for
/// the past agent once the store holds any record, and for a new trail before.
#[test]
fn a_prompt_with_no_open_trail_asks_for_the_past_agent() {
    let repo = Repo::new();
    let hook = |prompt: &str| {
        let json = serde_json::json!({"session_id": "s9", "prompt": prompt}).to_string();
        let o = repo.run(&repo.path(), &["prompt"], &[], &json);
        assert!(o.status.success(), "{}", err(&o));
        assert_eq!(out(&o), "");
        err(&o)
    };
    assert_eq!(hook("Build a Vendors table"), format!("{NEW}\n"));
    let o = repo.run(
        &repo.path(),
        &["decide", "--", "When you test: set SHELF_OFFLINE=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let said = hook("Build a Vendors table");
    assert_eq!(said.lines().count(), 1, "{said}");
    assert!(said.contains("celis:past agent"), "{said}");
    assert!(said.contains("before any other tool"), "{said}");
    assert!(said.contains("wait for its answer"), "{said}");
    assert!(
        said.contains("it brings the rules the ask must keep and the steps of past asks like it"),
        "{said}"
    );
    assert!(
        said.contains("--uses <id> for each rule and past step the ask follows"),
        "{said}"
    );
    assert!(said.contains(RULES), "{said}");
    assert!(!said.contains("SHELF_OFFLINE"), "{said}");
    assert!(marks(&repo, "prompt").is_empty());
}

/// `prompt` asks for the past agent once in a session, and again after a compaction's
/// `SessionStart`.
#[test]
fn a_session_asks_for_the_past_agent_once_until_a_compaction() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["decide", "--", "When you test: set SHELF_OFFLINE=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let hook = |json: serde_json::Value| {
        let o = repo.run(&repo.path(), &["prompt"], &[], &json.to_string());
        assert!(o.status.success(), "{}", err(&o));
        assert_eq!(out(&o), "");
        err(&o)
    };
    let ask = serde_json::json!({"session_id": "s9", "prompt": "Build a Vendors table"});
    assert!(hook(ask.clone()).contains("celis:past agent"));
    assert_eq!(hook(ask.clone()), format!("{NEW}\n"));
    let other = serde_json::json!({"session_id": "s8", "prompt": "Build a Vendors table"});
    assert!(hook(other).contains("celis:past agent"));
    assert_eq!(
        hook(serde_json::json!({"session_id": "s9", "source": "startup"})),
        ""
    );
    assert_eq!(hook(ask.clone()), format!("{NEW}\n"));
    assert_eq!(
        hook(serde_json::json!({"session_id": "s9", "source": "compact"})),
        ""
    );
    assert!(hook(ask).contains("celis:past agent"));
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
        "prompt",
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

/// A session's first prompt that names a step, as a worker gets it, writes nothing and does not ask for
/// the past agent (`spec/02-cli.md`, "Prompt").
#[test]
fn a_first_prompt_that_names_its_step_skips_the_past_agent() {
    let repo = Repo::new();
    let ask = add(&repo, Some("s1"), "the trail");
    let said = format!("Fix it. celis step {}", &ask[..8]);
    let json = serde_json::json!({"session_id": "w1", "prompt": said});
    let o = repo.run(&repo.path(), &["prompt"], &[], &json.to_string());
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "");
    assert!(!err(&o).contains("celis:past"), "{}", err(&o));
}
