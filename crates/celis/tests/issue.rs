//! CIP-05, "The issue".

mod support;

use support::{add, cli, err, objects, out, refusal, Repo, PHRASE};

/// What `celis issue <args>` prints in the main checkout with the variables `env`; it must succeed.
fn issue(repo: &Repo, args: &[&str], env: &[(&str, &str)]) -> String {
    let mut all = vec!["issue"];
    all.extend(args);
    let o = repo.run(&repo.path(), &all, env, "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// `issue` prints what went wrong as a heading and text, then the facts of this machine, with
/// `$CELIS_HOME` under `~` and no phrase (`spec/trails/05-reports.md`, "The issue").
#[test]
fn issue_prints_what_went_wrong_and_the_facts() {
    let repo = Repo::new();
    add(&repo, &["--", "a task"]);
    std::fs::write(repo.store().join("shared").join("bad.json"), "{").unwrap();
    let home = repo.home().parent().unwrap().to_str().unwrap().to_string();
    let env = [("HOME", home.as_str()), ("CLAUDE_CODE_SESSION_ID", "s1")];
    let shown = issue(&repo, &["--", "prompt fails\nin each session"], &env);
    let lines: Vec<&str> = shown.lines().collect();
    assert_eq!(
        lines[..3],
        ["## prompt fails", "", "in each session"],
        "{shown}"
    );
    let facts: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|l| l.starts_with("- "))
        .collect();
    assert!(facts[0].starts_with("- version: celis "), "{shown}");
    assert!(facts[1].starts_with("- system: "), "{shown}");
    assert_eq!(
        facts[2..],
        [
            "- sessions: CLAUDE_CODE_SESSION_ID",
            &format!("- project: repo `{}`", &repo.project_id()[..8]),
            "- home: `~/home`",
            "- phrase: kept",
            "- records: 2 files, 1 read, 1 steps",
        ],
        "{shown}"
    );
    assert!(
        !shown.contains(PHRASE.split(' ').next().unwrap()),
        "{shown}"
    );
}

/// With no text, the heading is `celis issue`; the log's last lines come in a code block, and the
/// section is left out while the log is empty (`spec/trails/02-cli.md`, "Issue").
#[test]
fn issue_prints_the_log_when_it_has_lines() {
    let repo = Repo::new();
    let shown = issue(&repo, &[], &[]);
    assert!(shown.starts_with("## celis issue\n"), "{shown}");
    assert!(!shown.contains("**Log**"), "{shown}");
    for _ in 0..55 {
        cli(&repo, &["nope"]);
    }
    let shown = issue(&repo, &[], &[]);
    let log = shown.split("**Log**\n\n```\n").nth(1).expect("a log");
    let log = log.split("\n```").next().unwrap();
    assert_eq!(log.lines().count(), 50, "{shown}");
    assert!(log.lines().all(|l| l.contains(" 2 celis nope · ")), "{log}");
}

/// `issue` prints the trails the ids name as `report` does, and refuses an id no step has with
/// `no-step` (`spec/trails/02-cli.md`, "Issue").
#[test]
fn issue_prints_the_trails_the_ids_name() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the badges"]);
    let shown = issue(&repo, &[&trail[..8], "--", "wrong state"], &[]);
    assert!(shown.contains("\n## the badges\n"), "{shown}");
    refusal(&cli(&repo, &["issue", "ffffffff", "--json"]), 1, "no-step");
    let printed = objects(&repo, &["issue"]);
    assert_eq!(printed.len(), 1, "{printed:?}");
    assert!(printed[0]["issue"]
        .as_str()
        .unwrap()
        .starts_with("## celis issue\n"));
}
