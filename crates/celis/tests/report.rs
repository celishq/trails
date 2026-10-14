//! CIP-05, "The report".

mod support;

use support::{add, cli, err, objects, out, refusal, Repo};

fn short(id: &str) -> &str {
    &id[..8]
}

/// What `celis report <args>` prints in the main checkout; it must succeed.
fn report(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["report"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// Waits past the second, since steps made in one second order by id.
fn tick() {
    std::thread::sleep(std::time::Duration::from_millis(1100));
}

/// Runs `celis <args>` in the main checkout, and asserts it succeeds.
fn ok(repo: &Repo, args: &[&str]) {
    let o = cli(repo, args);
    assert!(o.status.success(), "{}", err(&o));
}

/// The lines of `text`, without the time a meta line ends with.
fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| match l.find(" · took ") {
            Some(n) => l[..n].to_string(),
            None => l.to_string(),
        })
        .collect()
}

/// `report` prints the trail's title as a heading, its id, project, state and steps, its rules, and the
/// steps under it as a nested list with their marks, places, checks, proofs, changes and reasons
/// (`spec/trails/02-cli.md`, "Report").
#[test]
fn report_prints_the_trail_its_rules_and_its_steps_as_markdown() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the CRM\nwith a contacts list"]);
    let t = short(&trail);
    ok(
        &repo,
        &[
            "decide",
            "--scope",
            &format!("trail:{t}"),
            "--",
            "colours are tokens",
        ],
    );
    let list = add(&repo, &["--under", t, "--", "the list"]);
    let row = add(
        &repo,
        &[
            "--under",
            short(&list),
            "--at",
            "src/row.rs",
            "--check",
            "true",
            "--",
            "a row\nexpect: one line a contact",
        ],
    );
    tick();
    add(&repo, &["--under", t, "--check", "true", "--", "the page"]);
    tick();
    let old = add(&repo, &["--under", t, "--", "the old table"]);
    ok(
        &repo,
        &[
            "step",
            "done",
            short(&row),
            "--changed",
            "two lines",
            "--",
            "row test passes",
        ],
    );
    ok(
        &repo,
        &[
            "step",
            "done",
            short(&list),
            "--as-planned",
            "--",
            "list shows",
        ],
    );
    ok(
        &repo,
        &["step", "drop", short(&old), "--", "the list replaces it"],
    );
    let shown = lines(&report(&repo, &[t]));
    let want = [
        "## Build the CRM".to_string(),
        format!("`{t}` · repo · active · 4 steps"),
        String::new(),
        "with a contacts list".to_string(),
        String::new(),
        "**Rules**".to_string(),
        String::new(),
        "- colours are tokens".to_string(),
        String::new(),
        "**Steps**".to_string(),
        String::new(),
        "- ✓ the list".to_string(),
        "  - done: list shows".to_string(),
        "  - ✓ a row".to_string(),
        "    - expect: one line a contact".to_string(),
        "    - at `src/row.rs`".to_string(),
        "    - check passed `true`".to_string(),
        "    - done: row test passes".to_string(),
        "    - changed: two lines".to_string(),
        "- ○ the page".to_string(),
        "  - check `true`".to_string(),
        "- ✗ the old table".to_string(),
        "  - dropped: the list replaces it".to_string(),
    ];
    assert_eq!(shown, want, "{shown:#?}");
}

/// `report` prints the trail of each id once, in the order of the ids, with a blank line between two
/// trails (`spec/trails/02-cli.md`, "Report").
#[test]
fn report_prints_each_trail_once_in_the_order_of_the_ids() {
    let repo = Repo::new();
    let first = add(&repo, &["--", "first task"]);
    let part = add(&repo, &["--under", short(&first), "--", "a part"]);
    let second = add(&repo, &["--new", "--", "second task"]);
    let shown = report(&repo, &[short(&second), short(&part), short(&first)]);
    let heads: Vec<&str> = shown.lines().filter(|l| l.starts_with("## ")).collect();
    assert_eq!(heads, ["## second task", "## first task"], "{shown}");
    assert!(shown.contains("\n\n## first task\n"), "{shown}");
}

/// When a trail's first ask has its text, `report` leaves the ask out and prints its steps first
/// (`spec/trails/05-reports.md`, "The report").
#[test]
fn report_leaves_out_the_ask_with_the_trails_text() {
    let repo = Repo::new();
    let session = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--", "orange badges"],
        &session,
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let ids: Vec<String> = out(&o).lines().map(str::to_string).collect();
    add(
        &repo,
        &["--under", short(&ids[0]), "--", "the badge colour"],
    );
    let shown = report(&repo, &[short(&ids[1])]);
    let items: Vec<&str> = shown.lines().filter(|l| l.contains("- ")).collect();
    assert_eq!(items, ["- ○ the badge colour"], "{shown}");
}

/// `report` refuses an id no step has with `no-step`, and no id as a usage error
/// (`spec/trails/02-cli.md`, "Report").
#[test]
fn report_refuses_an_unknown_id_and_no_id() {
    let repo = Repo::new();
    add(&repo, &["--", "a task"]);
    refusal(&cli(&repo, &["report", "ffffffff", "--json"]), 1, "no-step");
    refusal(&cli(&repo, &["report", "--json"]), 2, "usage");
}

/// With `--json`, `report` prints its text as one object, and it writes no record
/// (`spec/trails/02-cli.md`, "Report").
#[test]
fn report_with_json_prints_the_text_and_writes_no_record() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "a task"]);
    let before = objects(&repo, &["tree", short(&trail)]);
    let printed = objects(&repo, &["report", short(&trail)]);
    assert_eq!(printed.len(), 1, "{printed:?}");
    let text = printed[0]["report"].as_str().unwrap();
    assert!(text.starts_with("## a task\n"), "{text}");
    assert_eq!(objects(&repo, &["tree", short(&trail)]), before);
}
