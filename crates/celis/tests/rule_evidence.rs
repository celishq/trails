//! CIP-02, "Rules": the step and the misses in `$CELIS_HOME/given`, and the rank they give.

mod support;

use celis_nostr::Note;
use serde_json::{json, Value};
use support::{account, added, err, out, put, rule_tags, Repo, DAY};

/// What `celis <args>` prints in the main checkout, in the session `session`.
fn run(repo: &Repo, session: &str, args: &[&str]) -> String {
    let o = repo.run(&repo.path(), args, &[("CELIS_SESSION_ID", session)], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// The id `celis decide <args> -- <text>` prints in the session `session`.
fn decide(repo: &Repo, session: &str, args: &[&str], text: &str) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    all.extend(["--", text]);
    run(repo, session, &all).trim().to_string()
}

/// The texts of the rules `celis rules <file>` gives in the session `session`.
fn rules(repo: &Repo, session: &str, file: &str) -> Vec<String> {
    let shown = run(repo, session, &["rules", file, "--session", session]);
    let rules = shown.lines().filter(|l| !l.starts_with("celis: "));
    rules
        .map(|l| l.splitn(3, ' ').nth(2).unwrap().to_string())
        .collect()
}

/// A rule of this repo with the extra tags `tags`, saying `content`, made at `at`, put in the store.
fn tagged(repo: &Repo, tags: &[(&str, &str)], content: &str, at: u64) -> Note {
    let mut all = rule_tags((&repo.project_id(), "repo"));
    all.extend(tags.iter().map(|(n, v)| vec![n.to_string(), v.to_string()]));
    let note = Note::write(&account(), 1111u16, at, all, content).unwrap();
    put(repo, &note);
    note
}

/// Each line of `$CELIS_HOME/given`.
fn log(repo: &Repo) -> Vec<Value> {
    let text = std::fs::read_to_string(repo.home().join("given")).unwrap_or_default();
    let lines = text.lines();
    lines.map(|l| serde_json::from_str(l).unwrap()).collect()
}

/// A rule given while the session has a working step names that step; with none, the line has no step
/// (`spec/02-cli.md`, "Rules").
#[test]
fn a_rule_given_names_the_working_step() {
    let repo = Repo::new();
    decide(&repo, "s1", &["--path", "*.md"], "markdown");
    decide(&repo, "s1", &["--path", "*.py"], "python");
    added(&repo, "s1", &["--new", "--", "ask"]);
    assert_eq!(rules(&repo, "s1", "README.md"), ["markdown"]);
    let part = added(&repo, "s1", &["--", "part"]).remove(0);
    run(&repo, "s1", &["step", "start", &part]);
    assert_eq!(rules(&repo, "s1", "a.py"), ["python"]);
    let lines = log(&repo);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].get("step").is_none(), "{lines:?}");
    assert_eq!(lines[1]["step"], json!(part));
}

/// A correction of a rule the session was not given logs a miss of that rule; a correction of a rule
/// it was given logs none (`spec/02-cli.md`, "Rules").
#[test]
fn a_correction_of_a_rule_not_given_logs_a_miss() {
    let repo = Repo::new();
    let old = decide(&repo, "s0", &["--path", "*.md"], "old markdown");
    let new = decide(&repo, "s1", &["--replaces", &old], "new markdown");
    let lines = log(&repo);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(
        (&lines[0]["session"], &lines[0]["rule"], &lines[0]["miss"]),
        (&json!("s1"), &json!(old), &json!(true))
    );
    assert!(lines[0]["at"].is_u64());
    let given = decide(&repo, "s2", &["--path", "*.py"], "python");
    assert_eq!(rules(&repo, "s2", "a.py"), ["python"]);
    decide(&repo, "s2", &["--replaces", &given], "new python");
    assert_eq!(log(&repo).len(), 2);
    assert!(log(&repo).iter().all(|l| l["rule"] != json!(new)));
}

/// A rule made by a correction comes first, then a rule a miss names, then the rest narrowest first,
/// then newest (`spec/02-cli.md`, "Rules").
#[test]
fn rules_rank_a_correction_then_a_miss_then_the_narrowest() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    let gone = tagged(&repo, &[("path", "*.py")], "gone", at);
    tagged(
        &repo,
        &[("path", "*.py"), ("replaces", &gone.id)],
        "correction",
        at + 1,
    );
    let missed = tagged(&repo, &[("path", "*.py")], "missed", at + 2);
    tagged(&repo, &[("path", "a.py")], "named", at + 3);
    tagged(&repo, &[("path", "*.py")], "wide", at + 4);
    let miss = json!({"at": at, "session": "s0", "rule": missed.id, "miss": true});
    std::fs::write(repo.home().join("given"), format!("{miss}\n")).unwrap();
    assert_eq!(
        rules(&repo, "s1", "a.py"),
        ["correction", "missed", "named", "wide"]
    );
}

/// `tree` shows under each step the rules given while it worked, whole with `--full`.
#[test]
fn the_tree_shows_the_rules_given_under_each_step() {
    let repo = Repo::new();
    let rule = decide(
        &repo,
        "s1",
        &["--path", "*.md"],
        "When you write docs: keep them short, because people skim.",
    );
    added(&repo, "s1", &["--new", "--", "ask"]);
    let part = added(&repo, "s1", &["--", "part"]).remove(0);
    run(&repo, "s1", &["step", "start", &part]);
    rules(&repo, "s1", "README.md");
    let tree = run(&repo, "s1", &["tree"]);
    let lines: Vec<&str> = tree.lines().collect();
    let at = lines.iter().position(|l| l.contains(&part[..8])).unwrap();
    let below = format!("§ {} When you write docs: keep them short", &rule[..8]);
    assert_eq!(lines[at + 1].trim(), below, "{tree}");
    let full = run(&repo, "s1", &["tree", "--full"]);
    let whole = format!(
        "§ {} When you write docs: keep them short, because people skim.",
        &rule[..8]
    );
    assert!(full.lines().any(|l| l.trim() == whole), "{full}");
}
