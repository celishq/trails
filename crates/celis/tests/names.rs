//! CIP-07, "Names", and CIP-02, "Name".

mod support;

use support::{
    account, add, added, cli, drop_note, err, finish, links, now, objects, out, refusal, step, Repo,
};

/// The ids the step `id` of `trail` uses.
fn uses(repo: &Repo, trail: &str, id: &str) -> Vec<String> {
    links(&step(repo, trail, id), "uses")
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
    let named = added(&repo, "s1", &["--", "Review a pull request"]);
    let (ask, trail) = (&named[0], &named[1]);
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

/// `names` prints each named trail, most used first, then newest; `--limit` cuts the list, and
/// `--json` gives the name, id, title and uses (`spec/02-cli.md`, "Names").
#[test]
fn names_lists_the_named_trails_most_used_first() {
    let repo = Repo::new();
    let used = add(&repo, &["--", "Review a pull request"]);
    let newer = add(&repo, &["--new", "--", "Write a changelog"]);
    name(&repo, &used, "review");
    name(&repo, &newer, "changelog");
    let other = add(&repo, &["--new", "--", "Check the shop's pull request"]);
    add(
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
    let o = cli(&repo, &["names"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines,
        [
            format!("review {} Review a pull request", &used[..8]),
            format!("changelog {} Write a changelog", &newer[..8]),
        ],
    );
    let limited = out(&cli(&repo, &["names", "--limit", "1"]));
    assert_eq!(limited.lines().count(), 1, "{limited}");
    let json = objects(&repo, &["names"]);
    assert_eq!(json[0]["name"], "review");
    assert_eq!(json[0]["id"], used.as_str());
    assert_eq!(json[0]["title"], "Review a pull request");
    assert_eq!(json[0]["uses"], 1);
    assert_eq!(json[1]["uses"], 0);
    refusal(
        &cli(&repo, &["names", "--json", "--limit", "x"]),
        2,
        "usage",
    );
}

/// A drop of a done trail keeps it done and frees its name for another trail (`spec/03-steps.md`,
/// "Dropping"; `spec/07-names.md`, "Names").
#[test]
fn a_dropped_trail_keeps_its_state_and_frees_its_name() {
    let repo = Repo::new();
    let old = add(&repo, &["--", "Review a pull request"]);
    finish(&repo, &old[..8]);
    name(&repo, &old, "review");
    drop_note(&repo, &account(), &old, now() + 1);
    assert_eq!(out(&cli(&repo, &["names"])), "");
    let state = &objects(&repo, &["tree", &old[..8]])[0]["trail"]["state"];
    assert_eq!(state, "done");
    let grep = out(&cli(&repo, &["grep", &old[..8]]));
    assert!(
        grep.contains(&format!("trail {} done-dropped", &old[..8])),
        "{grep}"
    );
    let new = add(&repo, &["--new", "--", "Review a design"]);
    name(&repo, &new, "review");
    let lines = out(&cli(&repo, &["names"]));
    assert_eq!(
        lines.trim(),
        format!("review {} Review a design", &new[..8])
    );
}
