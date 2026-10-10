//! CIP-14, "The commands": `trail tree`.

mod support;

use support::{add, cli, err, finish, objects, out, refusal, Repo};

fn short(id: &str) -> &str {
    &id[..8]
}

/// The lines `tree <args>` prints, with each grade line cut to `grade`, since its time varies.
fn printed(repo: &Repo, args: &[&str]) -> Vec<String> {
    let mut all = vec!["tree"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
        .lines()
        .map(|l| match l.trim_start().starts_with("took ") {
            true => format!("{}grade", &l[..l.len() - l.trim_start().len()]),
            false => l.to_string(),
        })
        .collect()
}

/// `tree` prints the trail with its state and rules, then the steps under it, oldest first, each
/// followed by its own, marked done or open, with the proof and grade of a done one
/// (`spec/trails/14-trails.md`, "The commands").
#[test]
fn tree_prints_the_trail_its_rules_and_the_steps_under_it() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "build the CRM"]);
    let o = cli(
        &repo,
        &[
            "decide",
            "--scope",
            &format!("trail:{}", short(&trail)),
            "--",
            "colours are tokens",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rule = out(&o).trim().to_string();
    let colours = add(&repo, &["--under", short(&trail), "--", "colours"]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let page = add(&repo, &["--under", short(&trail), "--", "customers page"]);
    let badge = add(&repo, &["--under", short(&page), "--", "badge"]);
    let other = add(&repo, &["--", "other"]);
    add(&repo, &["--under", short(&other), "--", "elsewhere"]);
    let o = cli(
        &repo,
        &["step", "done", short(&colours), "--", "lint failed, passes"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        printed(&repo, &[short(&trail)]),
        [
            format!("{} [repo] build the CRM · active", short(&trail)),
            format!("  rule {} colours are tokens", short(&rule)),
            format!("✓ {} [repo] colours", short(&colours)),
            "    done lint failed, passes".to_string(),
            "    grade".to_string(),
            format!("○ {} [repo] customers page", short(&page)),
            format!("  ○ {} [repo] badge", short(&badge)),
        ]
    );
}

/// `tree` prints a done trail's proof and grade under its first line (`spec/trails/14-trails.md`, "The
/// commands").
#[test]
fn tree_prints_a_done_trails_proof_and_grade() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "a small ask"]);
    finish(&repo, short(&trail));
    assert_eq!(
        printed(&repo, &[short(&trail)]),
        [
            format!("{} [repo] a small ask · reached alone", short(&trail)),
            "    done failed before, passes now".to_string(),
            "    grade".to_string(),
        ]
    );
}

/// `tree --json` prints the trail, then one object a step with its `under` and state, and writes no
/// record (`spec/trails/14-trails.md`, "The commands").
#[test]
fn tree_json_prints_each_step_with_its_under_and_state_and_writes_no_record() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "g"]);
    let first = add(&repo, &["--under", short(&trail), "--", "first"]);
    let second = add(&repo, &["--under", short(&first), "--", "second"]);
    let count = || {
        std::fs::read_dir(repo.store().join("shared"))
            .unwrap()
            .count()
    };
    let before = count();
    let lines = objects(&repo, &["tree", &trail]);
    assert_eq!(lines[0]["trail"]["id"], trail.as_str());
    assert_eq!(lines[0]["trail"]["state"], "active");
    assert_eq!(lines[1]["step"]["id"], first.as_str());
    assert_eq!(lines[1]["step"]["under"], trail.as_str());
    assert_eq!(lines[1]["step"]["state"], "open");
    assert_eq!(lines[2]["step"]["id"], second.as_str());
    assert_eq!(lines[2]["step"]["under"], first.as_str());
    assert_eq!(lines.len(), 3);
    assert_eq!(count(), before);
}

/// `tree` with no id prints every trail of the project, newest first, and refuses an id no step has
/// with `no-step` (`spec/trails/14-trails.md`, "The commands").
#[test]
fn tree_with_no_id_prints_every_trail_newest_first() {
    let repo = Repo::new();
    let first = add(&repo, &["--", "first trail"]);
    add(&repo, &["--under", short(&first), "--", "a part"]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let second = add(&repo, &["--", "second trail"]);
    let heads: Vec<String> = printed(&repo, &[])
        .into_iter()
        .filter(|l| !l.starts_with(' ') && !l.starts_with('○'))
        .map(|l| l[..8].to_string())
        .collect();
    assert_eq!(heads, [short(&second), short(&first)]);
    refusal(&cli(&repo, &["tree", "deadbeef", "--json"]), 1, "no-step");
}
