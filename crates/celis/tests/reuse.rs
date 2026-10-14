//! CIP-06, "Reuse".

mod support;

use serde_json::Value;
use support::{add, cli, err, finish, objects, out, refusal, Repo};

/// The id `decide -- <text>` prints.
fn decide(repo: &Repo, text: &str) -> String {
    let o = cli(repo, &["decide", "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// The ids `step add <args>` prints in the session `session`.
fn ids(repo: &Repo, session: &str, args: &[&str]) -> Vec<String> {
    let mut all = vec!["step", "add"];
    all.extend(args);
    let o = repo.run(
        &repo.path(),
        &all,
        &[("CLAUDE_CODE_SESSION_ID", session)],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

/// The JSON of the step `id` that `tree <trail> --json` prints.
fn step(repo: &Repo, trail: &str, id: &str) -> Value {
    objects(repo, &["tree", &trail[..8]])
        .into_iter()
        .filter_map(|v| v.get("step").or(v.get("trail")).cloned())
        .find(|s| s["id"] == id)
        .expect("the step")
}

/// The links of `step` of `kind`.
fn links(step: &Value, kind: &str) -> Vec<String> {
    step["links"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l["kind"] == kind)
        .map(|l| l["to"].as_str().unwrap().to_string())
        .collect()
}

/// `step add --uses` takes a live decision and a step of another trail, writes each whole id, and gives
/// a trail id as its first ask (`spec/trails/06-reuse.md`, "Uses").
#[test]
fn uses_takes_a_decision_and_a_step_of_another_trail() {
    let repo = Repo::new();
    let rule = decide(
        &repo,
        "When a test adds two steps: wait 1.1 s between them.",
    );
    let past = ids(&repo, "s1", &["--", "the senior price"]);
    let (past_ask, past_trail) = (&past[0], &past[1]);
    finish(&repo, &past_ask[..8]);
    let trail = add(&repo, &["--new", "--", "the student price"]);
    let id = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &rule[..8],
            "--uses",
            &past_trail[..8],
            "--",
            "a student pays 70%",
        ],
    );
    let json = step(&repo, &trail, &id);
    assert_eq!(links(&json, "uses"), [rule, past_ask.clone()], "{json}");
    assert_eq!(links(&json, "under"), [trail], "{json}");
}

/// `step add --uses` refuses an id no decision or step has with `no-step`, and a step of its own trail
/// as a usage error (`spec/trails/06-reuse.md`, "Uses").
#[test]
fn uses_refuses_an_unknown_id_and_a_step_of_the_same_trail() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    let o = cli(
        &repo,
        &[
            "step", "add", "--json", "--uses", "ffffffff", "--", "unknown",
        ],
    );
    refusal(&o, 1, "no-step");
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--under",
            &trail[..8],
            "--uses",
            &part[..8],
            "--",
            "same trail",
        ],
    );
    refusal(&o, 2, "usage");
}

/// On a session's new trail and its ask, the `uses` tags go on the ask only
/// (`spec/trails/06-reuse.md`, "Uses").
#[test]
fn uses_goes_on_the_ask_of_a_new_trail() {
    let repo = Repo::new();
    let rule = decide(&repo, "When a price rounds: round down to the cent.");
    let written = ids(&repo, "s1", &["--uses", &rule[..8], "--", "the price"]);
    let (ask, trail) = (&written[0], &written[1]);
    assert_eq!(links(&step(&repo, trail, ask), "uses"), [rule]);
    assert!(links(&step(&repo, trail, trail), "uses").is_empty());
}

/// `step add --replaces` takes a done step of another trail: the new step goes where `step add` puts it,
/// the old one stays done, and its grade and trail read as revised (`spec/trails/06-reuse.md`,
/// "Revisions").
#[test]
fn replaces_takes_a_done_step_and_reads_it_as_revised() {
    let repo = Repo::new();
    let old = ids(&repo, "s1", &["--", "the badge"]);
    let (old_ask, old_trail) = (&old[0], &old[1]);
    let part = add(&repo, &["--under", &old_ask[..8], "--", "the colour"]);
    finish(&repo, &part[..8]);
    finish(&repo, &old_ask[..8]);
    let trail = add(&repo, &["--new", "--", "fix the badge"]);
    let fix = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--replaces",
            &part[..8],
            "--",
            "the badge keeps its colour",
        ],
    );
    let json = step(&repo, &trail, &fix);
    assert_eq!(links(&json, "under"), [trail], "{json}");
    assert_eq!(links(&json, "replaces"), [part.as_str()], "{json}");
    let revised = step(&repo, old_trail, &part);
    assert_eq!(
        revised["grade"]["revised"],
        serde_json::json!([fix]),
        "{revised}"
    );
    let head = objects(&repo, &["tree", &old_trail[..8]]);
    assert_eq!(head[0]["trail"]["state"], "revised", "{head:?}");
    let tree = out(&cli(&repo, &["tree", &old_trail[..8], "--full"]));
    let line = format!(" · revised by {}", &fix[..8]);
    assert!(tree.contains(&line), "{tree}");
}

/// `step add --replaces` still refuses a dropped step with `closed` (`spec/trails/03-steps.md`,
/// "Dropping").
#[test]
fn replaces_refuses_a_dropped_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    let o = cli(&repo, &["step", "drop", &part[..8], "--", "not wanted"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--replaces",
            &part[..8],
            "--",
            "again",
        ],
    );
    refusal(&o, 1, "closed");
}

/// A chain of follows: the ask `a` of one trail, `b` of a second that uses `a` and the decision, and
/// `c` of a third that uses `b` and the decision; each done. Gives `(a, b, c, decision)`.
fn chain(repo: &Repo) -> (String, String, String, String) {
    let rule = decide(repo, "When a price rounds: round down to the cent.");
    let a = ids(repo, "s1", &["--", "a senior pays 80%"]).remove(0);
    finish(repo, &a[..8]);
    let follow = |source: &str, text: &str| {
        let trail = add(repo, &["--new", "--", text]);
        let step = add(
            repo,
            &[
                "--under",
                &trail[..8],
                "--uses",
                &source[..8],
                "--uses",
                &rule[..8],
                "--",
                text,
            ],
        );
        finish(repo, &step[..8]);
        step
    };
    let b = follow(&a, "a student pays 70%");
    let c = follow(&b, "a member pays 90%");
    (a, b, c, rule)
}

/// The line of `text` that holds `id`.
fn line<'a>(text: &'a str, id: &str) -> &'a str {
    text.lines().find(|l| l.contains(&id[..8])).unwrap_or("")
}

/// `stats` with nothing reused says so, and prints nothing with `--json`; it refuses an unknown id
/// with `no-step` (`spec/trails/02-cli.md`, "Stats").
#[test]
fn stats_says_when_nothing_was_reused() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    assert_eq!(
        out(&cli(&repo, &["stats"])).trim(),
        "Nothing was reused yet."
    );
    assert_eq!(out(&cli(&repo, &["stats", "--json"])), "");
    refusal(&cli(&repo, &["stats", "--json", "ffffffff"]), 1, "no-step");
}

/// `stats` prints each followed step, most followed at any depth first, then each used decision, then
/// the total (`spec/trails/02-cli.md`, "Stats").
#[test]
fn stats_prints_the_followed_steps_the_used_decisions_and_the_total() {
    let repo = Repo::new();
    let (a, b, _, rule) = chain(&repo);
    let text = out(&cli(&repo, &["stats"]));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with(&a[..8]), "{text}");
    assert!(lines[0].contains(" · followed 2 · saved "), "{text}");
    assert!(lines[1].starts_with(&b[..8]), "{text}");
    assert!(lines[1].contains(" · followed 1 · saved "), "{text}");
    assert!(
        lines[2].starts_with(&format!("decision {}", &rule[..8])),
        "{text}"
    );
    assert!(lines[2].ends_with(" · used 2"), "{text}");
    assert!(lines[3].starts_with("total 2 follows · saved "), "{text}");
    assert!(lines[3].ends_with(" · 2 uses"), "{text}");
    let json = objects(&repo, &["stats"]);
    assert_eq!(json[0]["followed"]["id"], a, "{json:?}");
    assert_eq!(json[0]["followed"]["followed"], 2, "{json:?}");
    assert_eq!(json[2]["used"]["used"], 2, "{json:?}");
    assert_eq!(json[3]["total"]["follows"], 2, "{json:?}");
}

/// `stats <id>` prints the step, then each step it follows, then each step that follows it, two spaces
/// further in a level (`spec/trails/02-cli.md`, "Stats").
#[test]
fn stats_of_a_step_prints_its_sources_then_its_followers() {
    let repo = Repo::new();
    let (a, b, c, _) = chain(&repo);
    let text = out(&cli(&repo, &["stats", &c[..8]]));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with(&c[..8]), "{text}");
    assert!(
        lines[1].starts_with(&format!("  follows {}", &b[..8])),
        "{text}"
    );
    assert!(
        lines[2].starts_with(&format!("    follows {}", &a[..8])),
        "{text}"
    );
    let text = out(&cli(&repo, &["stats", &a[..8]]));
    assert!(line(&text, &b).starts_with("  followed "), "{text}");
    assert!(line(&text, &b).contains(" · saved "), "{text}");
    assert!(line(&text, &c).starts_with("    followed "), "{text}");
}

/// `tree --full` prints a `follows` line under a step for each step it follows, and a `reused` line
/// when steps follow it; `tree --json` gives the count (`spec/trails/02-cli.md`, "Tree").
#[test]
fn tree_prints_follows_and_reused_lines() {
    let repo = Repo::new();
    let (a, b, _, _) = chain(&repo);
    let tree = out(&cli(&repo, &["tree", &b[..8], "--full"]));
    let follows = format!("follows {} a senior pays 80%", &a[..8]);
    assert!(
        tree.lines()
            .any(|l| l.trim_start_matches([' ', '│']) == follows),
        "{tree}"
    );
    assert!(
        tree.lines().any(|l| l
            .trim_start_matches([' ', '│'])
            .starts_with("reused 1 · saved ")),
        "{tree}"
    );
    let json = objects(&repo, &["tree", &b[..8]]);
    let at = json
        .iter()
        .find_map(|v| v.get("step").or(v.get("trail")))
        .unwrap();
    assert_eq!(at["reused"], 1, "{json:?}");
}
