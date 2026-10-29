//! The agent view of `tree`.

use crate::support;

use support::{add, cli, err, finish, noon, objects, out, refusal, Repo};

fn short(id: &str) -> &str {
    &id[..8]
}

/// The lines `tree <args>` prints.
fn printed(repo: &Repo, args: &[&str]) -> Vec<String> {
    let mut all = vec!["tree"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

/// `tree <id>` prints the trail as a block: its mark, id and title, then two spaces in its progress and
/// time, its open steps, its notes, then its closed steps, each group newest first and each step
/// followed by its own two spaces further in, in the same order; `--full` adds the proof and grade of a
/// done one.
#[test]
fn tree_prints_the_trail_its_notes_and_the_steps_under_it() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 600;
    let trail = support::step_note(&repo, &by, None, "build the CRM", at);
    let scope = format!("trail:{}", trail.id);
    let project = (repo.project_id(), "repo");
    let pattern = support::stored_pattern(
        &repo,
        (&project.0, project.1),
        &by,
        (Some(&scope), None),
        "colours are tokens",
        at,
    );
    let colours = support::step_note(&repo, &by, Some(&trail), "colours", at);
    support::mark_note(&repo, &by, "done", &colours.id, "proof", at + 90);
    let page = support::step_note(&repo, &by, Some(&trail), "customers page", at + 1);
    let badge = support::step_note(&repo, &by, Some(&page), "badge", at + 2);
    let seal = support::step_note(&repo, &by, Some(&page), "seal", at + 3);
    support::mark_note(&repo, &by, "done", &seal.id, "proof", at + 33);
    let stamp = support::step_note(&repo, &by, Some(&page), "stamp", at + 4);
    let logo = support::step_note(&repo, &by, Some(&trail), "logo", at + 5);
    support::mark_note(&repo, &by, "done", &logo.id, "proof", at + 65);
    let footer = support::step_note(&repo, &by, Some(&trail), "footer", at + 6);
    let ids = |text: &str| {
        text.replace("{t}", short(&trail.id))
            .replace("{r}", short(&pattern.id))
            .replace("{c}", short(&colours.id))
            .replace("{p}", short(&page.id))
            .replace("{b}", short(&badge.id))
            .replace("{s}", short(&seal.id))
            .replace("{m}", short(&stamp.id))
            .replace("{l}", short(&logo.id))
            .replace("{f}", short(&footer.id))
    };
    let lines = [
        "○   {t} build the CRM",
        "  3/7 steps · 1m 27s",
        "  ○   {f} footer",
        "  ○   {p} customers page · 1/3 steps · 30s",
        "    ○   {m} stamp",
        "    ○   {b} badge",
        "    ✓   {s} seal · 30s",
        "  ¶   {r} colours are tokens",
        "  ✓   {l} logo · 32s",
        "  ✓   {c} colours · 25s",
    ];
    assert_eq!(printed(&repo, &[short(&trail.id)]), lines.map(ids), "short");
    let full = [
        "○   {t} build the CRM",
        "  3/7 steps · 1m 27s",
        "  ○   {f} footer",
        "  ○   {p} customers page · 1/3 steps · 30s",
        "    ○   {m} stamp",
        "    ○   {b} badge",
        "    ✓   {s} seal · 30s",
        "        done     proof",
        "        took     30s",
        "  ¶   {r} colours are tokens",
        "  ✓   {l} logo · 32s",
        "      done     proof",
        "      took     32s",
        "  ✓   {c} colours · 25s",
        "      done     proof",
        "      took     25s",
    ];
    assert_eq!(
        printed(&repo, &[short(&trail.id), "--full"]),
        full.map(ids),
        "full"
    );
}

/// `tree <id>` prints the open steps of a level by the time each was made, newest first, and does not
/// lift a working step above a newer open one.
#[test]
fn tree_keeps_a_working_step_where_its_made_time_puts_it() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 600;
    let trail = support::step_note(&repo, &by, None, "build the CRM", at);
    let older = support::step_note(&repo, &by, Some(&trail), "older part", at + 1);
    support::mark_note(&repo, &by, "start", &older.id, "", at + 2);
    support::step_note(&repo, &by, Some(&trail), "newer part", at + 3);
    let lines = printed(&repo, &[short(&trail.id)]);
    let row = |what: &str| lines.iter().position(|l| l.contains(what)).unwrap();
    assert!(row("newer part") < row("older part"), "{lines:?}");
    assert!(!lines[row("older part")].starts_with("  ○"), "{lines:?}");
}

/// `tree <id> --full` prints the expect, for, not and ask me lines of a step in the label column, with
/// its proof and grade, and no ask me line whose value is none.
#[test]
fn tree_prints_the_expect_line_in_the_label_column() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 600;
    let trail = support::step_note(&repo, &by, None, "build the CRM", at);
    let logo = support::step_note(
        &repo,
        &by,
        Some(&trail),
        "logo\nexpect: the logo shows\nfor: a brand\nnot: a favicon\nask me: none",
        at + 5,
    );
    support::mark_note(&repo, &by, "done", &logo.id, "proof", at + 65);
    support::step_note(
        &repo,
        &by,
        Some(&trail),
        "badge\nask me: its colour",
        at + 6,
    );
    let lines = printed(&repo, &[short(&trail.id), "--full"]);
    assert!(
        lines.contains(&"      ask me   its colour".to_string()),
        "{lines:?}"
    );
    let at = lines.iter().position(|l| l.contains("logo")).unwrap();
    assert_eq!(
        lines[at + 1..at + 6],
        [
            "      expect   the logo shows",
            "      for      a brand",
            "      not      a favicon",
            "      done     proof",
            "      took     1m 0s"
        ]
    );
}

/// `tree <id> --full` prints each line of a step's text that starts with a key of one or two words of
/// letters, up to 12 characters, and a `:` with a space or nothing after it, with that key in the label
/// column, and each other line as plain text.
#[test]
fn tree_prints_any_key_of_a_step_in_the_label_column() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 600;
    let trail = support::step_note(&repo, &by, None, "build the CRM", at);
    support::step_note(
        &repo,
        &by,
        Some(&trail),
        "logo\nwhy: a brand\nRisk Note: none\nhttps://x.io\nat 10:30\nstep 1: run it\nmuch too long: no",
        at + 5,
    );
    let lines = printed(&repo, &[short(&trail.id), "--full"]);
    let at = lines.iter().position(|l| l.contains("logo")).unwrap();
    assert_eq!(
        lines[at + 1..at + 7],
        [
            "      why       a brand",
            "      risk note none",
            "      https://x.io",
            "      at 10:30",
            "      step 1: run it",
            "      much too long: no"
        ]
    );
}

/// `tree <id> --full` lines up the text of each label of a step after its longest label.
#[test]
fn tree_lines_up_the_labels_of_a_step_after_its_longest() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 600;
    let trail = support::step_note(&repo, &by, None, "build the CRM", at);
    let logo = support::step_note(
        &repo,
        &by,
        Some(&trail),
        "logo\nwhy: a brand\nhttps://x.io\nconstraints: no favicon",
        at + 5,
    );
    support::mark_note(&repo, &by, "done", &logo.id, "proof", at + 65);
    let lines = printed(&repo, &[short(&trail.id), "--full"]);
    let at = lines.iter().position(|l| l.contains("logo")).unwrap();
    assert_eq!(
        lines[at + 1..at + 6],
        [
            "      why         a brand",
            "      https://x.io",
            "      constraints no favicon",
            "      done        proof",
            "      took        1m 0s"
        ]
    );
}

/// `tree <id>` prints a stale step with the closed steps, after the notes.
#[test]
fn tree_prints_a_stale_step_with_the_closed_ones() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 2 * 3600;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    let scope = format!("trail:{}", trail.id);
    let project = (repo.project_id(), "repo");
    let pattern = support::stored_pattern(
        &repo,
        (&project.0, project.1),
        &by,
        (Some(&scope), None),
        "a rule",
        at,
    );
    let done = support::step_note(&repo, &by, Some(&trail), "done part", at + 1);
    support::mark_note(&repo, &by, "done", &done.id, "proof", at + 31);
    let left = support::step_note(&repo, &by, Some(&trail), "left part", at + 2);
    let lines = printed(&repo, &[short(&trail.id)]);
    assert_eq!(
        lines[2..],
        [
            format!("  ¶   {} a rule", short(&pattern.id)),
            format!("  ◌   {} left part", short(&left.id)),
            format!("  ✓   {} done part · 30s", short(&done.id)),
        ],
        "{lines:?}"
    );
}

/// `tree <id>` of an ask prints its notes on its block.
#[test]
fn tree_of_an_ask_prints_its_notes() {
    let repo = Repo::new();
    let ask = add(&repo, &["--", "customers page"]);
    let o = cli(
        &repo,
        &["step", "note", short(&ask), "--", "colours are tokens"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let pattern = out(&o).trim()[..8].to_string();
    let badge = add(&repo, &["--under", short(&ask), "--", "badge"]);
    assert_eq!(
        printed(&repo, &[short(&ask)]),
        [
            format!("○   {} customers page", short(&ask)),
            "  0/1 step".to_string(),
            format!("  ○   {} badge", short(&badge)),
            format!("  ¶   {pattern} colours are tokens"),
        ]
    );
}

/// `tree <id> --full` and `tree --next <id> --full` print a note on a step as `¶` under that step, after
/// the patterns it uses, and not under the trail; with no `--full`, under no step.
#[test]
fn tree_prints_a_steps_notes_under_it() {
    let repo = Repo::new();
    let ask = add(&repo, &["--", "customers page"]);
    let badge = add(&repo, &["--under", short(&ask), "--", "badge"]);
    let o = cli(
        &repo,
        &["step", "note", short(&badge), "--", "round corners"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let note = out(&o).trim()[..8].to_string();
    for next in [&[][..], &["--next"]] {
        let args = [next, &[short(&ask), "--full"]].concat();
        assert_eq!(
            printed(&repo, &args)[2..],
            [
                format!("  ○   {} badge", short(&badge)),
                format!("    ¶   {note} round corners"),
            ],
            "{args:?}"
        );
        let args = [next, &[short(&ask)]].concat();
        assert!(
            !printed(&repo, &args).iter().any(|l| l.contains(&note)),
            "{args:?}"
        );
    }
}

/// `tree --full` prints a done trail's proof and grade under its title, and no progress with no step
/// under it.
#[test]
fn tree_prints_a_done_trails_proof_and_grade() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 600;
    let trail = support::step_note(&repo, &by, None, "a small ask", at);
    support::mark_note(&repo, &by, "done", &trail.id, "proof", at + 45);
    assert_eq!(
        printed(&repo, &[short(&trail.id), "--full"]),
        [
            format!("✓   {} a small ask", short(&trail.id)),
            "  45s".to_string(),
            "    done     proof".to_string(),
            "    took     45s".to_string(),
        ]
    );
}

/// `tree --json` prints the trail, then one object a step with its links and state, and writes no
/// record.
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
    assert_eq!(lines[0]["trail"]["state"], "open");
    assert_eq!(lines[0]["trail"]["links"], serde_json::json!([]));
    assert_eq!(lines[1]["step"]["id"], first.as_str());
    assert_eq!(lines[1]["step"]["links"][0]["to"], trail.as_str());
    assert_eq!(lines[1]["step"]["state"], "open");
    assert_eq!(lines[2]["step"]["id"], second.as_str());
    assert_eq!(lines[2]["step"]["links"][0]["to"], first.as_str());
    assert_eq!(lines.len(), 3);
    assert_eq!(count(), before);
}

/// `tree` with no id starts with today's line and its counts, then its active trails with their projects,
/// then the others by project, the newest project first, each newest first, every one a block;
/// a dropped one's reason follows it; an id no step has is refused with `no-step`.
#[test]
fn tree_with_no_id_prints_today_by_project() {
    let repo = Repo::new();
    let dropped = add(&repo, &["--", "a dropped trail"]);
    let o = cli(
        &repo,
        &["step", "drop", short(&dropped), "--", "not needed"],
    );
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let ended = add(&repo, &["--", "an ended trail"]);
    let part = add(&repo, &["--under", short(&ended), "--", "its part"]);
    finish(&repo, short(&part));
    finish(&repo, short(&ended));
    let active = add(&repo, &["--", "an active trail"]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let shop = repo.other("shop");
    let o = repo.run(&shop, &["step", "add", "--", "a shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let lines = printed(&repo, &[]);
    let text = lines.join("\n");
    assert!(
        lines[0].contains(", ") && lines[0].ends_with(" · 2 active · 2 done"),
        "{text}"
    );
    assert_eq!(lines[1], "", "{text}");
    assert!(lines[2].ends_with(" a shop trail · shop"), "{text}");
    assert_eq!(
        lines[3],
        format!("○   {} an active trail · repo", short(&active))
    );
    assert_eq!(lines[4..6], ["", "repo"], "{text}");
    let at = |what: &str| lines.iter().position(|l| l.ends_with(what)).unwrap();
    assert!(at("an ended trail") < at("a dropped trail"), "{text}");
    let done = format!("  ✓   {} its part · ", short(&part));
    assert!(lines[at("an ended trail") + 2].starts_with(&done), "{text}");
    assert_eq!(lines[at("a dropped trail") + 1], "    dropped not needed");
    refusal(&cli(&repo, &["tree", "deadbeef", "--json"]), 1, "no-step");
}

/// `tree --all` prints every trail in the store, of every project, by day, newest first, one line each
/// with its progress and time.
#[test]
fn tree_all_prints_every_trail_of_every_project_by_day() {
    let repo = Repo::new();
    let by = support::account();
    let old = support::step_note(
        &repo,
        &by,
        None,
        "an old trail",
        support::now() - 2 * support::DAY,
    );
    support::mark_note(
        &repo,
        &by,
        "done",
        &old.id,
        "proof",
        support::now() - 2 * support::DAY,
    );
    let trail = add(&repo, &["--", "a new trail"]);
    let part = add(&repo, &["--under", short(&trail), "--", "its part"]);
    finish(&repo, short(&part));
    finish(&repo, short(&trail));
    let lines = printed(&repo, &["--all"]);
    let text = lines.join("\n");
    assert!(lines[0].ends_with(" · 0 active · 1 done"), "{text}");
    assert_eq!(lines[1..3], ["", "repo"], "{text}");
    let line = format!("✓   {} a new trail · 1/1 step · ", short(&trail));
    assert!(
        lines[3].starts_with(&line) && lines[3].ends_with('s'),
        "{text}"
    );
    assert!(lines[5].ends_with(" · 0 active · 1 done"), "{text}");
    assert_eq!(lines[6..8], ["", "repo"], "{text}");
    assert_eq!(
        lines[8],
        format!("✓   {} an old trail · 0s", short(&old.id))
    );
    assert_eq!(lines.len(), 9, "{text}");
    assert!(!text.contains("its part"), "{text}");
    let full = printed(&repo, &["--all", "--full"]).join("\n");
    assert!(full.contains("its part"), "{full}");
    let o = cli(&repo, &["tree", "--all", short(&trail), "--json"]);
    refusal(&o, 2, "usage");
}

/// The agent view prints a step's place whole, and cuts, wraps and colours nothing, whatever the
/// width.
#[test]
fn the_agent_view_prints_whole_lines_with_no_colour() {
    let repo = Repo::new();
    let title = "a trail whose words run well past thirty characters";
    let trail = add(&repo, &["--", title]);
    let pattern = "When you price a senior: use cents, because floats lost a cent on the receipt.";
    let o = cli(&repo, &["step", "note", short(&trail), "--", pattern]);
    assert!(o.status.success(), "{}", err(&o));
    let part = add(
        &repo,
        &[
            "--under",
            short(&trail),
            "--at",
            "shelf/prices.py:40",
            "--",
            "a part whose words run long",
        ],
    );
    let env = [("COLUMNS", "30"), ("CLICOLOR_FORCE", "1")];
    let o = repo.run(&repo.path(), &["tree", short(&trail)], &env, "");
    let text = out(&o);
    assert!(!text.contains('\x1b') && !text.contains('…'), "{text:?}");
    assert!(text.contains(title) && text.contains(pattern), "{text}");
    assert!(
        text.contains(&format!(
            "  ○   {} a part whose words run long · shelf/prices.py:40\n",
            short(&part)
        )),
        "{text}"
    );
}

/// `tree` prints a done step's `changed` tag under it as `~ <changed>`, and its flags after the mark: `↻`
/// on a revised step and its trail, then `~` when changed, which keep their states.
#[test]
fn tree_prints_what_changed_and_marks_a_revised_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", short(&trail), "--", "the part"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            short(&part),
            "--changed",
            "one file, not two",
            "--",
            "it works",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    add(&repo, &["--under", short(&trail), "--", "next"]);
    let lines = printed(&repo, &[short(&trail)]);
    let done = format!("  ✓~  {} the part · ", short(&part));
    assert!(lines[3].starts_with(&done), "{lines:?}");
    assert_eq!(lines[4], "      ~ one file, not two");
    let other = add(&repo, &["--", "another trail"]);
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--under",
            short(&other),
            "--replaces",
            short(&part),
            "--",
            "a better part",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let lines = printed(&repo, &[short(&trail)]);
    assert!(lines[0].starts_with("○↻  "), "{lines:?}");
    assert!(
        lines[3].starts_with(&format!("  ✓↻~ {}", short(&part))),
        "{lines:?}"
    );
    let json = objects(&repo, &["tree", short(&trail)]);
    let step = json
        .iter()
        .find(|o| o["step"]["id"] == part.as_str())
        .unwrap();
    assert_eq!(step["step"]["state"], "done", "{json:?}");
}

/// `tree <id>` and `tree --next <id>` start each id of a level in one column, whatever signs stand
/// after a mark.
#[test]
fn the_agent_view_starts_each_id_in_one_column() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", short(&trail), "--", "the part"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            short(&part),
            "--changed",
            "less",
            "--",
            "ok",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let other = add(&repo, &["--", "another trail"]);
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--under",
            short(&other),
            "--replaces",
            short(&part),
            "--",
            "a better part",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let plain = add(&repo, &["--under", short(&trail), "--", "plain"]);
    for args in [vec![short(&trail)], vec!["--next", short(&trail)]] {
        let lines = printed(&repo, &args);
        let column = |id: &str| {
            let line = lines.iter().find(|l| l.contains(short(id))).unwrap();
            line[..line.find(short(id)).unwrap()].chars().count()
        };
        assert_eq!(column(&trail), 4, "{args:?} {lines:#?}");
        assert_eq!(column(&part), 6, "{args:?} {lines:#?}");
        assert_eq!(column(&plain), 6, "{args:?} {lines:#?}");
    }
}

/// `tree <id>` prints a trail with no record from today, and `tree` with no id only today's.
#[test]
fn tree_with_no_id_shows_only_today_and_tree_with_an_id_prints_any_trail() {
    let repo = Repo::new();
    let by = support::account();
    let old = support::step_note(
        &repo,
        &by,
        None,
        "an old trail",
        support::now() - 2 * support::DAY,
    );
    let ended = add(&repo, &["--", "an ended trail"]);
    let part = add(&repo, &["--under", short(&ended), "--", "its part"]);
    finish(&repo, short(&part));
    finish(&repo, short(&ended));
    let text = out(&cli(&repo, &["tree"]));
    assert!(!text.contains("an old trail"), "{text}");
    assert!(text.contains("its part"), "{text}");
    let o = cli(&repo, &["tree", short(&old.id)]);
    assert!(out(&o).contains("an old trail"), "{}", out(&o));
}

/// `tree` with no id prints today's trails of every project, in any folder; with none, it names `tree
/// --all`.
#[test]
fn tree_with_no_id_prints_every_projects_trails_and_names_all_when_empty() {
    let repo = Repo::new();
    let o = cli(&repo, &["tree"]);
    assert_eq!(
        out(&o),
        "No active trails. Run `celis tree --all` to see every trail.\n"
    );
    let o = cli(&repo, &["tree", "--json"]);
    assert_eq!(out(&o), "");
    add(&repo, &["--", "a repo trail"]);
    let elsewhere = repo.other("elsewhere");
    let o = repo.run(&elsewhere, &["tree"], &[], "");
    assert!(out(&o).contains(" a repo trail · repo\n"), "{}", out(&o));
}

/// The tree prints an older trail's first ask as the trail when their texts match, and the ask's steps
/// one level up; `--json` prints both (`spec/04-trails.md`, "Trails").
#[test]
fn the_tree_folds_a_trails_twin_first_ask() {
    let repo = Repo::new();
    let (by, at) = (support::account(), support::now() - 60);
    let head = support::step_note(&repo, &by, None, "the trail", at);
    let ask = &support::step_note(&repo, &by, Some(&head), "the trail", at + 1).id;
    let trail = &head.id;
    let part = add(&repo, &["--under", short(ask), "--", "the part"]);
    let lines = printed(&repo, &[short(trail)]);
    assert!(!lines.join("\n").contains(short(ask)), "{lines:?}");
    assert_eq!(lines[2], format!("  ○   {} the part", short(&part)));
    let json = out(&cli(&repo, &["tree", short(trail), "--json"]));
    assert!(json.contains(ask.as_str()), "{json}");
}

/// `tree --demo` prints made-up trails of two projects with a step or a trail in each state, a change,
/// patterns and a dropped reason, `--all` a past day too, in a folder it removes, and writes nothing to
/// the store; with an id it refuses.
#[test]
fn tree_demo_prints_every_state_and_writes_nothing() {
    let repo = Repo::new();
    let temp = tempfile::tempdir().unwrap();
    let zone = noon();
    let env = [("TMPDIR", temp.path().to_str().unwrap()), ("TZ", &zone)];
    let o = repo.run(&repo.path(), &["tree", "--demo"], &env, "");
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let working = text.lines().filter(|l| l.starts_with("◐ ")).count();
    assert!(working >= 2, "{text}");
    for mark in ["○ ", "◐ ", "✓ ", "✓↻ ", "✗ ", "◌ "] {
        assert!(
            text.lines().any(|l| l.trim_start().starts_with(mark)),
            "{mark}: {text}"
        );
    }
    for part in ["\nshop\n", "\ncrm\n", "  § ", "  ¶ ", " ~ ", " dropped "] {
        assert!(text.contains(part), "{part:?}: {text}");
    }
    let all = out(&repo.run(&repo.path(), &["tree", "--demo", "--all"], &env, ""));
    let days = all.lines().filter(|l| l.contains(" active · ")).count();
    assert!(days >= 2, "{all}");
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    assert!(
        !repo.store().exists()
            || std::fs::read_dir(repo.store().join("shared"))
                .unwrap()
                .count()
                == 0
    );
    let o = repo.run(
        &repo.path(),
        &["tree", "--demo", "abcd", "--json"],
        &env,
        "",
    );
    refusal(&o, 2, "usage");
}

/// `tree <id> --graph` prints the trail as a graph: its path, its steps by state with its bar, done of
/// all and time worked, then each step right under it as a card in a chain from left to right in the
/// order they were added, a done one closed to its time, an open one with steps under it the bar of
/// those done, and the trail's notes.
#[test]
fn tree_graph_prints_the_trail_as_a_chain_of_cards() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 600;
    let trail = support::step_note(&repo, &by, None, "build the CRM", at);
    let scope = format!("trail:{}", trail.id);
    let project = (repo.project_id(), "repo");
    support::stored_pattern(
        &repo,
        (&project.0, project.1),
        &by,
        (Some(&scope), None),
        "colours are tokens",
        at,
    );
    let colours = support::step_note(&repo, &by, Some(&trail), "colours", at);
    support::mark_note(&repo, &by, "done", &colours.id, "proof", at + 90);
    let page = support::step_note(&repo, &by, Some(&trail), "customers page", at + 1);
    let badge = support::step_note(&repo, &by, Some(&page), "badge", at + 2);
    support::step_note(&repo, &by, Some(&badge), "badge icon", at + 3);
    let seal = support::step_note(&repo, &by, Some(&page), "seal", at + 4);
    support::mark_note(&repo, &by, "done", &seal.id, "proof", at + 33);
    support::step_note(&repo, &by, Some(&trail), "footer", at + 6);
    let printed: Vec<String> = printed(&repo, &[short(&trail.id), "--graph"])
        .iter()
        .map(|line| unnamed(line))
        .collect();
    let graph = [
        "  All trails  ›  📁 repo  ›  ○ build the CRM",
        "  ────────────────────────────────────────────────────────────────────────────────────────────────────────",
        "  ✓ 2 done · ○ 4 open                                               ━━━━━━━━━━━━━━━━━━━━━━━━  2/6 · 1m 26s",
        "",
        "  ╭─ ✓ <id> ─────────╮  ╭╌ ▸ ○ <id> ╌╌╌╌╌╌╌╌╮  ╭╌ ○ <id> ╌╌╌╌╌╌╌╌╌╮",
        "  │ colours              │  ╎ customers page        ╎  ╎ footer               ╎",
        "  │                  57s │┄▶╎ ━━━━━━━━━━━━ 1/3      ╎┄▶╎ open                 ╎",
        "  ╰──────────────────────╯  ╰╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╯  ╰╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╯",
        "",
        "  ╭─ § Rules 1 ──────────────────────────────────────────────────────────────────────────────────────────╮",
        "  │ § colours are tokens                                                                                 │",
        "  ╰──────────────────────────────────────────────────────────────────────────────────────────────────────╯",
    ];
    assert_eq!(printed, graph);
}

/// `line` with each id of 8 hex digits as `<id>`.
fn unnamed(line: &str) -> String {
    let words =
        line.split(' ').map(
            |w| match w.len() == 8 && w.chars().all(|c| c.is_ascii_hexdigit()) {
                true => "<id>",
                false => w,
            },
        );
    words.collect::<Vec<_>>().join(" ")
}

/// With `--next`, `tree --demo` prints made-up trails of two projects with a step or a trail in each state, a change,
/// patterns and a dropped reason, `--all` a past day too, in a folder it removes, and writes nothing to
/// the store; with an id it refuses.
#[test]
fn next_tree_demo_prints_every_state_and_writes_nothing() {
    let repo = Repo::new();
    let temp = tempfile::tempdir().unwrap();
    let zone = noon();
    let env = [("TMPDIR", temp.path().to_str().unwrap()), ("TZ", &zone)];
    let o = repo.run(&repo.path(), &["tree", "--next", "--demo"], &env, "");
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let working = text.lines().filter(|l| l.starts_with("⠏ ")).count();
    assert!(working >= 2, "{text}");
    for mark in ["○ ", "⠏ ", "✓ ", "✓↻ ", "✗ ", "◌ "] {
        assert!(
            text.lines().any(|l| l.trim_start().starts_with(mark)),
            "{mark}: {text}"
        );
    }
    for part in ["\nshop\n", "\ncrm\n", "  § ", "  ¶ ", " ~ ", " dropped "] {
        assert!(text.contains(part), "{part:?}: {text}");
    }
    let all = out(&repo.run(
        &repo.path(),
        &["tree", "--next", "--demo", "--all"],
        &env,
        "",
    ));
    let days = all.lines().filter(|l| l.contains(" active · ")).count();
    assert!(days >= 2, "{all}");
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    assert!(
        !repo.store().exists()
            || std::fs::read_dir(repo.store().join("shared"))
                .unwrap()
                .count()
                == 0
    );
    let o = repo.run(
        &repo.path(),
        &["tree", "--next", "--demo", "abcd", "--json"],
        &env,
        "",
    );
    refusal(&o, 2, "usage");
}
