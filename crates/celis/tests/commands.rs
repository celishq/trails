//! CIP-11, "Commands".

mod support;

use serde_json::Value;
use std::process::Output;
use support::{account, decision, done_note, err, labelled, now, out, refusal, step_note, Repo};

const WORK: &str = "When you change prices: keep them in cents, because floats lost a cent.";

fn run(repo: &Repo, args: &[&str]) -> Output {
    repo.run(&repo.path(), args, &[], "")
}

fn short(id: &str) -> String {
    id[..8].to_string()
}

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared"))
        .map(|d| d.count())
        .unwrap_or(0)
}

/// The lines `celis <args>` prints in `folder`.
fn lines_of(repo: &Repo, folder: &std::path::Path, args: &[&str]) -> Vec<String> {
    let o = repo.run(folder, args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

fn project(id: &str) -> (&str, &str) {
    (id, "repo")
}

/// `read` prints the active trails, oldest first, each with its rules, then the open steps that are no
/// trail, then the live decisions that are no rule (`spec/trails/11-cli.md`, "Commands").
#[test]
fn read_prints_the_trails_with_their_rules_then_the_open_steps_then_the_decisions() {
    let repo = Repo::new();
    let id = repo.project_id();
    let t = now() - 5 * 60;
    let first = step_note(&repo, &account(), None, "first", t);
    let second = step_note(&repo, &account(), None, "second\nmore", t + 1);
    let scope = format!("trail:{}", second.id);
    let rule = decision(
        &repo,
        project(&id),
        &account(),
        (Some(&scope), None),
        "a rule",
        t + 2,
    );
    let work = step_note(&repo, &account(), Some(&first), "work", t + 3);
    let finished = step_note(&repo, &account(), Some(&first), "finished", t + 4);
    done_note(&repo, &account(), &finished, t + 5);
    let cents = decision(&repo, project(&id), &account(), (None, None), WORK, t + 6);
    assert_eq!(
        lines_of(&repo, &repo.path(), &["read"]),
        [
            format!("{} [repo] first", short(&first.id)),
            format!("{} [repo] second", short(&second.id)),
            format!("  rule {} a rule", short(&rule.id)),
            format!("{} [repo] work", short(&work.id)),
            format!("{} project {WORK}", short(&cents.id)),
        ]
    );
}

/// `read` prints only the working folder's project, and no trail that is reached, nor its rules
/// (`spec/trails/11-cli.md`, "Commands").
#[test]
fn read_prints_no_reached_trail_and_no_rule_of_one() {
    let repo = Repo::new();
    let id = repo.project_id();
    let t = now() - 5 * 60;
    let reached = step_note(&repo, &account(), None, "reached", t);
    done_note(&repo, &account(), &reached, t + 1);
    let scope = format!("trail:{}", reached.id);
    decision(
        &repo,
        project(&id),
        &account(),
        (Some(&scope), None),
        "old rule",
        t + 2,
    );
    decision(
        &repo,
        ("elsewhere", "shop"),
        &account(),
        (None, None),
        "theirs",
        t + 3,
    );
    assert!(lines_of(&repo, &repo.path(), &["read"]).is_empty());
}

/// `read` puts the decisions at each `--at` place first (`spec/trails/11-cli.md`, "Commands").
#[test]
fn read_puts_the_decisions_at_each_at_place_first() {
    let repo = Repo::new();
    let id = repo.project_id();
    let t = now() - 5 * 60;
    let a = decision(
        &repo,
        project(&id),
        &account(),
        (Some("at:a.py"), None),
        "a",
        t,
    );
    let b = decision(
        &repo,
        project(&id),
        &account(),
        (Some("at:b.py"), None),
        "b",
        t + 1,
    );
    let ids = |args: &[&str]| -> Vec<String> {
        lines_of(&repo, &repo.path(), args)
            .iter()
            .map(|l| l[..8].to_string())
            .collect()
    };
    assert_eq!(ids(&["read"]), [short(&b.id), short(&a.id)]);
    assert_eq!(ids(&["read", "--at", "a.py"]), [short(&a.id), short(&b.id)]);
}

/// `read` prints 30 decisions at most, those at an `--at` place first, then how many it left out
/// (`spec/trails/11-cli.md`, "Commands").
#[test]
fn read_prints_30_decisions_at_most_those_at_a_place_first() {
    let repo = Repo::new();
    let id = repo.project_id();
    let t = now() - 5 * 60;
    let there = decision(
        &repo,
        project(&id),
        &account(),
        (Some("at:a.py"), None),
        "there",
        t,
    );
    let others: Vec<_> = (1..=35)
        .map(|i| {
            decision(
                &repo,
                project(&id),
                &account(),
                (None, None),
                &format!("n{i}"),
                t + i,
            )
        })
        .collect();
    let mut lines = lines_of(&repo, &repo.path(), &["read", "--at", "a.py"]);
    assert_eq!(
        lines.pop().as_deref(),
        Some("6 more decisions"),
        "{lines:?}"
    );
    let got: Vec<String> = lines.iter().map(|l| l[..8].to_string()).collect();
    assert_eq!(got.len(), 30, "{got:?}");
    assert_eq!(got[0], short(&there.id));
    assert_eq!(got[1], short(&others[34].id));
    assert_eq!(got[29], short(&others[6].id));
}

/// With `--like`, `read` prints the past trails after the decisions (`spec/trails/11-cli.md`,
/// "Commands").
#[test]
fn read_with_like_prints_the_past_trails_after_the_decisions() {
    let repo = Repo::new();
    let id = repo.project_id();
    let t = now() - 5 * 60;
    let past = step_note(&repo, &account(), None, "Build the Vendors page", t);
    done_note(&repo, &account(), &past, t + 1);
    let cents = decision(&repo, project(&id), &account(), (None, None), WORK, t + 2);
    let lines = lines_of(&repo, &repo.path(), &["read", "--like", "Build a page"]);
    assert_eq!(lines[0], format!("{} project {WORK}", short(&cents.id)));
    assert_eq!(
        lines[1],
        format!(
            "{} [repo] Build the Vendors page · reached alone",
            short(&past.id)
        )
    );
}

/// `read` writes no record (`spec/trails/11-cli.md`, "Commands").
#[test]
fn read_writes_no_record() {
    let repo = Repo::new();
    let id = repo.project_id();
    let trail = step_note(&repo, &account(), None, "a trail", now() - 60);
    done_note(&repo, &account(), &trail, now() - 60);
    decision(
        &repo,
        project(&id),
        &account(),
        (None, None),
        "one",
        now() - 60,
    );
    let before = records(&repo);
    for args in [
        vec!["read"],
        vec!["read", "--at", "a.py"],
        vec!["read", "--json", "--like", "a trail"],
    ] {
        let o = repo.run(&repo.path(), &args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
    }
    assert_eq!(records(&repo), before);
}

/// A decision prints as `<id, first 8> <scope> <text>` (`spec/trails/11-cli.md`, "Commands").
#[test]
fn a_decision_prints_its_id_scope_and_text() {
    let repo = Repo::new();
    let decide = |args: &[&str]| {
        let mut all = vec!["decide"];
        all.extend(args);
        let o = repo.run(&repo.path(), &all, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        short(out(&o).trim())
    };
    let trail = out(&run(&repo, &["step", "add", "--", "t"]))
        .trim()
        .to_string();
    let scope = format!("trail:{}", short(&trail));
    let placed = decide(&["--scope", "at:a.py", "--", "one"]);
    let mine = decide(&["--scope", "me", "--", "two"]);
    let project = decide(&["--", "three"]);
    let rule = decide(&["--scope", &scope, "--", "four"]);
    let lines = lines_of(&repo, &repo.path(), &["read"]);
    for line in [
        format!("{placed} at:a.py one"),
        format!("{project} project three"),
        format!("{mine} me two"),
        format!("  rule {rule} four"),
    ] {
        assert!(lines.contains(&line), "{line}: {lines:?}");
    }
}

/// The json form of a decision is one object with the fields the section lists
/// (`spec/trails/11-cli.md`, "Commands").
#[test]
fn the_json_form_of_a_decision_has_the_listed_fields() {
    let repo = Repo::new();
    let id = repo.project_id();
    let note = decision(
        &repo,
        project(&id),
        &account(),
        (Some("at:a.py"), None),
        "one",
        now() - 60,
    );
    let o = repo.run(&repo.path(), &["read", "--json"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let line: Value = serde_json::from_str(out(&o).trim()).unwrap();
    let decision = line["decision"].as_object().unwrap();
    let mut keys: Vec<&str> = decision.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "content",
            "created_at",
            "id",
            "project",
            "replaces",
            "scope"
        ]
    );
    assert_eq!(decision["id"], note.id.as_str());
    assert_eq!(decision["scope"], "at:a.py");
}

/// `decide` writes a decision on the working folder's project with `--scope` and `--replaces` as its
/// tags, the whole id of a trail, and no `scope` tag for the project (`spec/trails/11-cli.md`,
/// "Commands").
#[test]
fn decide_writes_its_scope_and_replaces_as_tags() {
    let repo = Repo::new();
    let id = repo.project_id();
    let trail = out(&run(&repo, &["step", "add", "--", "t"]))
        .trim()
        .to_string();
    let old = decision(
        &repo,
        project(&id),
        &account(),
        (None, None),
        "old",
        now() - 60,
    );
    let written = |args: &[&str]| {
        let mut all = vec!["decide"];
        all.extend(args);
        all.extend(["--", WORK]);
        let o = repo.run(&repo.path(), &all, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        let id = out(&o).trim().to_string();
        labelled(&repo, "decision")
            .into_iter()
            .find(|n| n.id == id)
            .unwrap()
    };
    let replacing = written(&[
        "--scope",
        "at:shelf/prices.py",
        "--replaces",
        &short(&old.id),
    ]);
    assert_eq!(replacing.content, WORK);
    assert_eq!(replacing.tag("scope"), Some("at:shelf/prices.py"));
    assert_eq!(replacing.tag("replaces"), Some(old.id.as_str()));
    assert_eq!(replacing.tag("I"), Some(format!("project:{id}").as_str()));
    let rule = written(&["--scope", &format!("trail:{}", short(&trail))]);
    assert_eq!(rule.tag("scope"), Some(format!("trail:{trail}").as_str()));
    assert_eq!(written(&["--scope", "project"]).tag("scope"), None);
}

/// `decide` refuses empty text, and a scope it does not know, as a usage error
/// (`spec/trails/11-cli.md`, "Commands").
#[test]
fn decide_refuses_empty_text_and_an_unknown_scope_as_a_usage_error() {
    let repo = Repo::new();
    for args in [
        vec!["decide", "--json", "--", ""],
        vec!["decide", "--json"],
        vec!["decide", "--json", "--scope", "team:design", "--", WORK],
        vec!["decide", "--json", "--scope", "at:", "--", WORK],
    ] {
        refusal(&repo.run(&repo.path(), &args, &[], ""), 2, "usage");
    }
    assert_eq!(records(&repo), 0);
}

/// `decide` refuses `trail:<id>` when no active trail of the project has the id, with `no-trail`
/// (`spec/trails/11-cli.md`, "Commands").
#[test]
fn decide_refuses_a_trail_no_active_trail_of_the_project_has() {
    let repo = Repo::new();
    let t = now() - 5 * 60;
    let reached = step_note(&repo, &account(), None, "reached", t);
    done_note(&repo, &account(), &reached, t + 1);
    let active = step_note(&repo, &account(), None, "active", t + 2);
    let part = step_note(&repo, &account(), Some(&active), "part", t + 3);
    let unknown = "0".repeat(64);
    for trail in [unknown.as_str(), reached.id.as_str(), part.id.as_str()] {
        let scope = format!("trail:{trail}");
        let args = ["decide", "--json", "--scope", &scope, "--", WORK];
        refusal(&repo.run(&repo.path(), &args, &[], ""), 1, "no-trail");
    }
    let shop = repo.other("shop");
    let scope = format!("trail:{}", active.id);
    let args = ["decide", "--json", "--scope", &scope, "--", WORK];
    refusal(&repo.run(&shop, &args, &[], ""), 1, "no-trail");
    assert!(labelled(&repo, "decision").is_empty());
}

/// `decide` refuses a `--replaces` no live decision has, with `no-decision`
/// (`spec/trails/11-cli.md`, "Commands").
#[test]
fn decide_refuses_a_replaces_no_live_decision_has() {
    let repo = Repo::new();
    let id = repo.project_id();
    let old = decision(
        &repo,
        project(&id),
        &account(),
        (None, None),
        "old",
        now() - 3 * 60,
    );
    decision(
        &repo,
        project(&id),
        &account(),
        (None, Some(&old.id)),
        "new",
        now() - 2 * 60,
    );
    let unknown = "0".repeat(64);
    for target in [unknown.as_str(), old.id.as_str()] {
        let args = ["decide", "--json", "--replaces", target, "--", WORK];
        refusal(&repo.run(&repo.path(), &args, &[], ""), 1, "no-decision");
    }
    assert_eq!(labelled(&repo, "decision").len(), 2);
}

/// A flag no command takes is a usage error (`spec/trails/11-cli.md`, "Commands").
#[test]
fn a_flag_no_command_takes_is_a_usage_error() {
    let repo = Repo::new();
    for args in [
        vec!["read", "--json", "--bogus"],
        vec!["decide", "--json", "--bogus", "--", WORK],
        vec!["step", "add", "--json", "--needs", "x", "--", "work"],
    ] {
        refusal(&repo.run(&repo.path(), &args, &[], ""), 2, "usage");
    }
}

/// `home` prints the full path of the store and writes nothing; with `--json`, `{"path"}`
/// (`spec/trails/11-cli.md`, "Commands").
#[test]
fn home_prints_the_path_of_the_store() {
    let repo = Repo::new();
    let o = run(&repo, &["home"]);
    assert!(o.status.success(), "{}", err(&o));
    let store = repo.store().display().to_string();
    assert_eq!(out(&o).trim(), store);
    let o = run(&repo, &["home", "--json"]);
    let printed: Value = serde_json::from_str(out(&o).trim()).unwrap();
    assert_eq!(printed["path"], store.as_str());
    assert!(!repo.store().exists());
}
