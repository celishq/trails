//! CIP-05, "The report".

mod support;

use std::time::Duration;
use support::watch::Watch;
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

/// The lines of `text`, each time in it as `<t>`.
fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| {
            let words = l.split(' ').map(|w| match w.len() > 1 && w.ends_with('s') {
                true if w[..w.len() - 1].chars().all(|c| c.is_ascii_digit()) => "<t>",
                _ => w,
            });
            words.collect::<Vec<_>>().join(" ")
        })
        .collect()
}

/// The rows of the steps table of `text`.
fn rows(text: &str) -> Vec<String> {
    let mut lines = text.lines().skip_while(|l| !l.starts_with("| | Step |"));
    lines.nth(1);
    lines
        .take_while(|l| l.starts_with('|'))
        .map(str::to_string)
        .collect()
}

/// `report` prints the trail's title as a heading, a table of its id, state, time, counts and project,
/// a table of the steps under it with their marks, times and proofs, then their texts, places and checks,
/// and its rules, each in a details block (`spec/02-cli.md`, "Report").
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
        String::new(),
        "| Id | State | Took | Steps | Rules | Revised | Dropped | Project |".to_string(),
        "|----|-------|-----:|------:|------:|--------:|--------:|---------|".to_string(),
        format!("| `{t}` | ○ open | <t> | 4 | 1 | 0 | 1 | repo |"),
        String::new(),
        "with a contacts list".to_string(),
        String::new(),
        "| | Step | Took | Proof |".to_string(),
        "|-|------|-----:|-------|".to_string(),
        "| ✓ | the list | <t> | list shows |".to_string(),
        "| ✓ | \u{a0}\u{a0}└ a row | <t> | row test passes · ~ changed: two lines |".to_string(),
        "| ○ | the page | — |  |".to_string(),
        "| ✗ | the old table | — | dropped: the list replaces it |".to_string(),
        String::new(),
        "<details><summary>Texts, places and checks</summary>".to_string(),
        String::new(),
        "- ✓ a row".to_string(),
        "  - expect: one line a contact".to_string(),
        "  - at `src/row.rs`".to_string(),
        "  - check passed `true`".to_string(),
        "- ○ the page".to_string(),
        "  - check `true`".to_string(),
        String::new(),
        "</details>".to_string(),
        String::new(),
        "<details><summary>1 rule</summary>".to_string(),
        String::new(),
        "- colours are tokens".to_string(),
        String::new(),
        "</details>".to_string(),
    ];
    assert_eq!(shown, want, "{shown:#?}");
}

/// `report` prints the trail of each id once, in the order of the ids, with a blank line between two
/// trails (`spec/02-cli.md`, "Report").
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
/// (`spec/05-reports.md`, "The report").
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
    assert_eq!(rows(&shown), ["| ○ | the badge colour | — |  |"], "{shown}");
}

/// `report` refuses an id no step has with `no-step`, and no id as a usage error
/// (`spec/02-cli.md`, "Report").
#[test]
fn report_refuses_an_unknown_id_and_no_id() {
    let repo = Repo::new();
    add(&repo, &["--", "a task"]);
    refusal(&cli(&repo, &["report", "ffffffff", "--json"]), 1, "no-step");
    refusal(&cli(&repo, &["report", "--json"]), 2, "usage");
}

/// With `--json`, `report` prints its text as one object, and it writes no record
/// (`spec/02-cli.md`, "Report").
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

/// What a terminal running `celis <args>` shows once it holds `text`.
fn shown_with(watch: &mut Watch, text: &str) -> String {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        let shown = watch.shown(Duration::from_millis(50));
        if shown.contains(text) {
            return shown;
        }
    }
    panic!("no {text:?} in {:?}", watch.shown(Duration::ZERO));
}

/// On a terminal, `report` prints a card a trail: its id, project and title, its time and counts, a
/// line a step under it, its rules, and the keys it reads (`spec/02-cli.md`, "Report").
#[test]
fn report_on_a_terminal_prints_a_card() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the CRM"]);
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
    let mut watch = Watch::start(&repo, &["report", t]);
    let shown = shown_with(&mut watch, "m copy markdown · any key to leave");
    for text in [
        &format!("REPORT · {t} · repo"),
        "Build the CRM",
        "STEPS",
        "✓ the list",
        "§ 1 rule",
    ] {
        assert!(shown.contains(text), "{text:?} in {shown}");
    }
    assert!(!shown.contains("| Id |"), "{shown}");
}

/// On a terminal, `m` copies the report's Markdown and says so, and another key leaves; each clears the
/// key line and ends (`spec/02-cli.md`, "Report").
#[test]
fn report_on_a_terminal_copies_its_markdown_with_m() {
    let repo = Repo::new();
    let bin = repo.home().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let copy = bin.join("pbcopy");
    std::fs::write(&copy, "#!/bin/sh\ncat > \"$CELIS_HOME/copied\"\n").unwrap();
    std::fs::set_permissions(&copy, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    let trail = add(&repo, &["--", "a task"]);
    let mut watch = Watch::start(&repo, &["report", short(&trail)]);
    shown_with(&mut watch, "any key to leave");
    watch.press("m");
    shown_with(&mut watch, "copied markdown");
    assert_eq!(watch.child.wait().unwrap().code(), Some(0));
    let copied = std::fs::read_to_string(repo.home().join("copied")).unwrap();
    assert!(copied.starts_with("## a task\n"), "{copied}");
    let mut watch = Watch::start(&repo, &["report", short(&trail)]);
    shown_with(&mut watch, "any key to leave");
    watch.press("x");
    assert_eq!(watch.child.wait().unwrap().code(), Some(0));
    let shown = watch.shown(Duration::from_millis(100));
    assert!(shown.ends_with("\r\x1b[2K"), "{shown:?}");
}

/// `report --demo` prints a made-up trail with its rules and steps from a folder it removes, writes
/// nothing to the store, and refuses an id (`spec/02-cli.md`, "Report").
#[test]
fn report_demo_prints_a_made_up_trail_and_writes_nothing() {
    let repo = Repo::new();
    let temp = tempfile::tempdir().unwrap();
    let env = [("TMPDIR", temp.path().to_str().unwrap())];
    let o = repo.run(&repo.path(), &["report", "--demo"], &env, "");
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    assert!(
        text.starts_with("## Add a senior tier to the price list\n"),
        "{text}"
    );
    for part in [
        "| Id |",
        "| | Step |",
        "<details><summary>2 rules</summary>",
    ] {
        assert!(text.contains(part), "{part:?}: {text}");
    }
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    assert!(
        !repo.store().join("shared").exists()
            || std::fs::read_dir(repo.store().join("shared"))
                .unwrap()
                .count()
                == 0
    );
    let o = repo.run(
        &repo.path(),
        &["report", "--demo", "abcd", "--json"],
        &env,
        "",
    );
    refusal(&o, 2, "usage");
}
