//! CIP-02, "Step": `--under`, and a step with open steps under it.

mod support;

use support::{add, cli, err, finish, objects, out, refusal, Repo};

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared")).map_or(0, |d| d.count())
}

/// `add` refuses an `--under` no step has with `no-step`, and a closed one with `closed`
/// (`spec/02-cli.md`, "Step").
#[test]
fn add_refuses_an_unknown_or_closed_under() {
    let repo = Repo::new();
    let closed = add(&repo, &["--", "closed"]);
    finish(&repo, &closed[..8]);
    let o = cli(
        &repo,
        &["step", "add", "--under", "deadbeef", "--json", "--", "x"],
    );
    refusal(&o, 1, "no-step");
    let o = cli(
        &repo,
        &["step", "add", "--under", &closed[..8], "--json", "--", "x"],
    );
    refusal(&o, 1, "closed");
}

/// `add` carries the whole id of `--under`, and the JSON of a step gives its `under` and `replaces` as
/// `links` (`spec/02-cli.md`, "Step").
#[test]
fn add_carries_the_whole_id_of_under_and_links_it() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let first = add(&repo, &["--under", &trail[..8], "--", "first"]);
    let fix = add(&repo, &["--replaces", &first[..8], "--", "fix it"]);
    let fix = fix.lines().next().unwrap();
    let step = objects(&repo, &["tree", &trail[..8]])
        .into_iter()
        .filter_map(|v| v.get("step").cloned())
        .find(|s| s["id"] == fix)
        .expect("the replacement");
    assert_eq!(
        step["links"],
        serde_json::json!([
            {"to": trail, "kind": "under"},
            {"to": first, "kind": "replaces"},
        ])
    );
    assert!(step.get("under").is_none(), "{step}");
}

/// `done` refuses a step with an open step under it, with `open-steps`, and writes nothing
/// (`spec/02-cli.md`, "Step").
#[test]
fn done_refuses_a_step_with_an_open_step_under_it() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    let before = records(&repo);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &trail[..8],
            "--json",
            "--as-planned",
            "--",
            "p",
        ],
    );
    refusal(&o, 1, "open-steps");
    assert_eq!(records(&repo), before);
    finish(&repo, &part[..8]);
    finish(&repo, &trail[..8]);
}

/// `add` of an ask, a session's first or one right under a trail, says on stderr to add its first job
/// under it; `add` of a step under an ask says nothing of it.
#[test]
fn add_of_an_ask_says_to_add_its_first_job_under_it() {
    let repo = Repo::new();
    let hint = |o: &std::process::Output| {
        let id = out(o)[..8].to_string();
        err(o).contains(&format!("When {id} holds more than one job"))
    };
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(&repo.path(), &["step", "add", "--", "first ask"], &env, "");
    assert!(hint(&o), "{}", err(&o));
    let trail = add(&repo, &["--", "the trail"]);
    let o = cli(
        &repo,
        &["step", "add", "--under", &trail[..8], "--", "an ask"],
    );
    assert!(hint(&o), "{}", err(&o));
    let ask = out(&o)[..8].to_string();
    let o = cli(&repo, &["step", "add", "--under", &ask, "--", "a job"]);
    assert!(!hint(&o), "{}", err(&o));
}
