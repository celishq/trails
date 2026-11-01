//! `celis migrate`: the patterns of a retired form, and their rewrite in the current form.

use crate::support;

use celis_nostr::Note;
use support::{account, add, cli, err, now, out, put, Repo, DAY};

/// A pattern of `repo` that the test account signed at `at`, labelled `label` in `namespace`, with the
/// `scope` tag `scope` when given and the tags `extra`, put in the store.
fn old(
    repo: &Repo,
    (label, namespace): (&str, &str),
    scope: Option<&str>,
    extra: &[(&str, &str)],
    content: &str,
    at: u64,
) -> Note {
    let anchor = format!("project:{}", repo.project_id());
    let mut tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), "repo".into()],
        vec!["L".into(), namespace.into()],
        vec!["l".into(), label.into(), namespace.into()],
    ];
    tags.extend(scope.map(|s| vec!["scope".to_string(), s.to_string()]));
    tags.extend(
        extra
            .iter()
            .map(|(k, v)| vec![k.to_string(), v.to_string()]),
    );
    let note = Note::write(&account(), 1111u16, at, tags, content).unwrap();
    put(repo, &note);
    note
}

const DECISION: (&str, &str) = ("decision", "celis.decision");
const PATTERN: (&str, &str) = ("pattern", "celis.pattern");

/// `migrate` lists each live pattern of a retired form, oldest first, and no reader gives one
/// (`spec/02-cli.md`, "Migrate").
#[test]
fn migrate_lists_each_live_pattern_of_a_retired_form() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let at = now() - DAY;
    let gone = old(
        &repo,
        DECISION,
        None,
        &[],
        "When a logo is drawn: keep it round.",
        at,
    );
    let flat = old(
        &repo,
        DECISION,
        None,
        &[("replaces", &gone.id)],
        "When a logo is drawn: keep it flat.",
        at + 1,
    );
    let shelf = old(
        &repo,
        PATTERN,
        Some("at:shelf/a.py:3"),
        &[],
        "When a shelf is read: sort it.",
        at + 2,
    );
    let scope = format!("trail:{trail}");
    let vendor = old(
        &repo,
        PATTERN,
        Some(&scope),
        &[],
        "When a vendor prints: use initials.",
        at + 3,
    );
    let o = cli(&repo, &["migrate"]);
    assert!(o.status.success(), "{}", err(&o));
    let want = [
        format!(
            "old {} decision When a logo is drawn: keep it flat.",
            &flat.id[..8]
        ),
        format!("old {} at When a shelf is read: sort it.", &shelf.id[..8]),
        format!(
            "old {} trail When a vendor prints: use initials.",
            &vendor.id[..8]
        ),
    ];
    assert_eq!(out(&o).lines().collect::<Vec<_>>(), want);
    let index = out(&cli(&repo, &["patterns"]));
    assert_eq!(index.trim(), "no pattern yet");
}

/// `migrate --yes` writes each pattern again in the current form: a pattern that replaces it, with the
/// paths of an `at:<place>`, and a note on the trail of a `trail:<id>`; then nothing is left
/// (`spec/02-cli.md`, "Migrate").
#[test]
fn migrate_yes_writes_each_again_in_the_current_form() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let at = now() - DAY;
    let flat = old(
        &repo,
        DECISION,
        Some("me"),
        &[("path", "*.svg")],
        "When a logo is drawn: keep it flat.",
        at,
    );
    let shelf = old(
        &repo,
        PATTERN,
        Some("at:shelf"),
        &[],
        "When a shelf is read: sort it.",
        at + 1,
    );
    let scope = format!("trail:{trail}");
    old(
        &repo,
        PATTERN,
        Some(&scope),
        &[],
        "When a vendor prints: use initials.",
        at + 2,
    );
    let o = cli(&repo, &["migrate", "--yes"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).lines().count(), 4, "{}", out(&o));
    let svg = out(&cli(&repo, &["patterns", "logo.svg"]));
    assert!(svg.contains(" me When a logo is drawn"), "{svg}");
    let book = out(&cli(&repo, &["patterns", "shelf/b.py"]));
    assert!(book.contains(" project When a shelf is read"), "{book}");
    let new = out(&cli(&repo, &["grep", "-F", "When a logo"]));
    assert!(
        !new.contains(&flat.id[..8]) && !new.contains(&shelf.id[..8]),
        "{new}"
    );
    let started = out(&cli(&repo, &["step", "start", &trail[..8]]));
    assert!(
        started.contains("When a vendor prints: use initials."),
        "{started}"
    );
    assert_eq!(out(&cli(&repo, &["migrate"])).trim(), "Nothing to migrate.");
}

/// `migrate` with nothing to migrate says so, and prints nothing with `--json` (`spec/02-cli.md`,
/// "Migrate").
#[test]
fn migrate_with_nothing_to_migrate_says_so() {
    let repo = Repo::new();
    let o = cli(&repo, &["migrate"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim(), "Nothing to migrate.");
    assert!(out(&cli(&repo, &["migrate", "--json"])).is_empty());
}

/// `migrate --json` gives each pattern of a retired form as one object (`spec/02-cli.md`, "Migrate").
#[test]
fn migrate_json_gives_each_pattern_as_one_object() {
    let repo = Repo::new();
    let flat = old(
        &repo,
        DECISION,
        None,
        &[],
        "When a logo is drawn: keep it flat.",
        now() - DAY,
    );
    let line = out(&cli(&repo, &["migrate", "--json"]));
    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(value["old"]["id"], flat.id.as_str());
    assert_eq!(value["old"]["form"], "decision");
    assert_eq!(value["old"]["text"], "When a logo is drawn: keep it flat.");
}

/// The session start asks to offer `/celis:migrate` while a pattern of a retired form remains.
#[test]
fn the_session_start_offers_migrate_while_an_old_pattern_remains() {
    let repo = Repo::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/celis");
    let root = root.display().to_string();
    let json = serde_json::json!({"session_id": "s1", "cwd": repo.path()}).to_string();
    let start = |repo: &Repo| {
        out(&repo.run(
            &repo.path(),
            &["hook", "start", "claude", &root],
            &[],
            &json,
        ))
    };
    old(
        &repo,
        DECISION,
        None,
        &[],
        "When a logo is drawn: keep it flat.",
        now() - DAY,
    );
    assert!(start(&repo).contains("/celis:migrate"), "{}", start(&repo));
    assert!(cli(&repo, &["migrate", "--yes"]).status.success());
    assert!(!start(&repo).contains("/celis:migrate"), "{}", start(&repo));
}

/// A retired flag or command is a usage error (`spec/README.md`, "Names").
#[test]
fn a_retired_flag_or_command_is_a_usage_error() {
    let repo = Repo::new();
    let step = add(&repo, &["--", "Run the tests"]);
    let runs: [&[&str]; 5] = [
        &["pattern", "--binds", "--", "When you test: set A=1."],
        &[
            "step",
            "add",
            "--recalled",
            "abcd1234",
            "--",
            "Test the app",
        ],
        &[
            "step",
            "done",
            &step,
            "--as-planned",
            "--by",
            "judge",
            "--",
            "tests pass",
        ],
        &["lesson", &step],
        &["search", "logo"],
    ];
    for run in runs {
        let o = cli(&repo, run);
        assert_eq!(o.status.code(), Some(2), "{run:?}: {}", err(&o));
    }
}

/// `migrate` lists a pattern of a retired form that another account signed, and no `decision` whose
/// scope no reader knows (`spec/02-cli.md`, "Migrate").
#[test]
fn migrate_lists_another_accounts_pattern_and_no_unknown_scope() {
    let repo = Repo::new();
    let at = now() - DAY;
    let task = old(
        &repo,
        DECISION,
        Some("task:abcd"),
        &[],
        "When a task ends: say so.",
        at,
    );
    let mut other = old(&repo, DECISION, None, &[], "unused", at);
    let by = celis_nostr::Account::new(&[2u8; 32]).unwrap();
    other = Note::write(
        &by,
        1111u16,
        at + 1,
        other.tags.clone(),
        "When a team reads: be short.",
    )
    .unwrap();
    put(&repo, &other);
    let listed = out(&cli(&repo, &["migrate"]));
    assert!(
        listed.contains(&format!("old {} decision", &other.id[..8])),
        "{listed}"
    );
    assert!(!listed.contains(&task.id[..8]), "{listed}");
}
