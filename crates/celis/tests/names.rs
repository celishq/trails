//! CIP-07, "Names", and CIP-02, "Name".

mod support;

use serde_json::Value;
use support::{add, cli, err, finish, objects, out, refusal, Repo};

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

/// The ids the step `id` of `trail` uses, as `tree <trail> --json` prints them.
fn uses(repo: &Repo, trail: &str, id: &str) -> Vec<String> {
    let steps = objects(repo, &["tree", &trail[..8]]).into_iter();
    let step: Value = steps
        .filter_map(|v| v.get("step").or(v.get("trail")).cloned())
        .find(|s| s["id"] == id)
        .expect("the step");
    let links = step["links"].as_array().unwrap().iter();
    links
        .filter(|l| l["kind"] == "uses")
        .map(|l| l["to"].as_str().unwrap().to_string())
        .collect()
}

/// The id `name <trail> <name>` prints.
fn name(repo: &Repo, trail: &str, name: &str) {
    let o = cli(repo, &["name", &trail[..8], name]);
    assert!(o.status.success(), "{}", err(&o));
}

/// `name` writes a name on a trail and prints its id; the same name on the same trail again writes
/// nothing and prints that id (`spec/02-cli.md`, "Name").
#[test]
fn name_writes_a_name_once() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Review a pull request"]);
    let first = cli(&repo, &["name", &trail[..8], "review"]);
    assert!(first.status.success(), "{}", err(&first));
    let id = out(&first).trim().to_string();
    assert_eq!(id.len(), 64, "{id}");
    let again = cli(&repo, &["name", &trail[..8], "review"]);
    assert!(again.status.success(), "{}", err(&again));
    assert_eq!(out(&again).trim(), id);
}

/// `name` refuses a step that is not a trail, a text that is no name, and a name another trail holds
/// (`spec/02-cli.md`, "Name"; `spec/07-names.md`, "Names").
#[test]
fn name_refuses_a_step_a_bad_name_and_a_held_name() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Review a pull request"]);
    let step = add(&repo, &["--under", &trail, "--", "Read the diff"]);
    let other = add(&repo, &["--new", "--", "Review a design"]);
    refusal(
        &cli(&repo, &["name", &step[..8], "diff", "--json"]),
        2,
        "usage",
    );
    for bad in ["Review", "9review", "re view", "re_view", ""] {
        refusal(
            &cli(&repo, &["name", &trail[..8], bad, "--json"]),
            2,
            "usage",
        );
    }
    let o = cli(&repo, &["name", &trail[..8], "review"]);
    assert!(o.status.success(), "{}", err(&o));
    refusal(
        &cli(&repo, &["name", &other[..8], "review", "--json"]),
        2,
        "usage",
    );
}

/// `step add --uses <name>` writes the whole id of the named trail's first ask
/// (`spec/02-cli.md`, "Step"; `spec/06-reuse.md`, "Uses").
#[test]
fn uses_takes_a_name_as_its_trail() {
    let repo = Repo::new();
    let skill = ids(&repo, "s1", &["--", "Review a pull request"]);
    let (ask, trail) = (&skill[0], &skill[1]);
    finish(&repo, &ask[..8]);
    name(&repo, trail, "review");
    let other = add(&repo, &["--new", "--", "Check the shop's pull request"]);
    let id = add(
        &repo,
        &[
            "--under",
            &other[..8],
            "--uses",
            "review",
            "--",
            "Review it",
        ],
    );
    assert_eq!(uses(&repo, &other, &id), [ask.as_str()]);
}

/// A name wins over a step id that starts with the same text (`spec/02-cli.md`, "Step").
#[test]
fn a_name_wins_over_an_id_that_starts_with_it() {
    let repo = Repo::new();
    let named = add(&repo, &["--", "Review a pull request"]);
    let other = add(&repo, &["--new", "--", "Check the shop's pull request"]);
    let lettered = (0..40)
        .map(|n| add(&repo, &["--new", "--", &format!("Another task {n}")]))
        .find(|id| id.starts_with(|c: char| c.is_ascii_lowercase()))
        .expect("an id that starts with a letter");
    name(&repo, &named, &lettered[..8]);
    let id = add(
        &repo,
        &[
            "--under",
            &other[..8],
            "--uses",
            &lettered[..8],
            "--",
            "Review it",
        ],
    );
    assert_eq!(uses(&repo, &other, &id), [named]);
}

/// `--project skill:<name>` writes a step and a rule of the project `skill:<name>` from a repo, and
/// refuses any other id (`spec/02-cli.md`, "Commands"; `spec/07-names.md`, "Skill projects").
#[test]
fn project_writes_into_a_skill_project() {
    let repo = Repo::new();
    let trail = add(
        &repo,
        &["--project", "skill:review", "--", "Review a pull request"],
    );
    let o = cli(
        &repo,
        &[
            "decide",
            "--project",
            "skill:review",
            "--scope",
            &format!("trail:{}", &trail[..8]),
            "--",
            "When you review: read the tests first, because they state the intent.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let tree = objects(&repo, &["tree", &trail[..8]]);
    assert_eq!(tree[0]["trail"]["project"], "skill:review", "{tree:?}");
    let step = add(&repo, &["--under", &trail[..8], "--", "Read the tests"]);
    let start = objects(&repo, &["step", "start", &step[..8]]);
    let rule = start
        .iter()
        .find_map(|v| v.get("decision"))
        .expect("a rule");
    assert_eq!(rule["project"], "skill:review", "{start:?}");
    for bad in ["web:review", "skill:Review", "skill:"] {
        refusal(
            &cli(
                &repo,
                &["step", "add", "--json", "--project", bad, "--", "x"],
            ),
            2,
            "usage",
        );
        refusal(
            &cli(&repo, &["decide", "--json", "--project", bad, "--", "x"]),
            2,
            "usage",
        );
    }
}
