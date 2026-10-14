//! CIP-02, "Tree": `trail tree`.

mod support;

use support::{add, cli, err, finish, objects, out, refusal, Repo};

fn short(id: &str) -> &str {
    &id[..8]
}

/// The lines `tree --full <args>` prints, each without its details, and each time in ` · ` parts or
/// after a `took` label as `T`, since times vary.
fn printed(repo: &Repo, args: &[&str]) -> Vec<String> {
    let mut all = vec!["tree", "--full"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(|l| untimed(&bare(l))).collect()
}

/// `line` without the details after its text: from the first two spaces after its guides and label.
fn bare(line: &str) -> String {
    let text = line.len() - line.trim_start_matches([' ', '│']).len();
    let label = ["done    ", "took    ", "changed "]
        .iter()
        .find(|l| line[text..].starts_with(*l))
        .map_or(0, |l| l.len());
    match line[text + label..].find("  ") {
        Some(end) => line[..text + label + end].to_string(),
        None => line.trim_end().to_string(),
    }
}

/// `line` with each time, a ` · ` part or the text after `took`, as `T`.
fn untimed(line: &str) -> String {
    let time = |part: &str| {
        !part.is_empty()
            && part.split(' ').all(|w| {
                w.len() > 1
                    && w.is_ascii()
                    && w[..w.len() - 1].bytes().all(|b| b.is_ascii_digit())
                    && w.ends_with(['s', 'm', 'h', 'd'])
            })
    };
    if let Some(n) = line.find("took    ") {
        return format!("{}took    T", &line[..n]);
    }
    let parts: Vec<&str> = line.split(" · ").collect();
    let parts: Vec<&str> = parts
        .iter()
        .map(|p| match time(p) {
            true => "T",
            false => p,
        })
        .collect();
    parts.join(" · ")
}

/// `tree` prints the trail as a block: its title, then its project, progress, worked time and rules,
/// then the rules with `--full`, then the steps under it, oldest first, each followed by its own one
/// guide down, with the proof and grade of a done one (`spec/02-cli.md`, "Tree").
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
    add(&repo, &["--under", short(&page), "--", "badge"]);
    let other = add(&repo, &["--", "other"]);
    add(&repo, &["--under", short(&other), "--", "elsewhere"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            short(&colours),
            "--as-planned",
            "--",
            "lint failed, passes",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        printed(&repo, &[short(&trail)]),
        [
            "○ build the CRM".to_string(),
            "  repo · 1 of 3 · T · § 1 rule".to_string(),
            format!("  § {} colours are tokens", short(&rule)),
            String::new(),
            "  ✓ colours".to_string(),
            "    done    lint failed, passes".to_string(),
            "    took    T".to_string(),
            "  ○ customers page".to_string(),
            "  │ ○ badge".to_string(),
        ]
    );
}

/// `tree <id>` of an ask counts its trail's rules on its block (`spec/02-cli.md`, "Tree").
#[test]
fn tree_of_an_ask_counts_its_trails_rules() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "build the CRM"]);
    let scope = format!("trail:{}", short(&trail));
    let o = cli(
        &repo,
        &["decide", "--scope", &scope, "--", "colours are tokens"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let ask = add(&repo, &["--under", short(&trail), "--", "customers page"]);
    add(&repo, &["--under", short(&ask), "--", "badge"]);
    let o = cli(&repo, &["tree", short(&ask)]);
    let lines: Vec<String> = out(&o).lines().map(|l| untimed(&bare(l))).collect();
    assert_eq!(
        lines,
        [
            "○ customers page",
            "  repo · 0 of 1 · § 1 rule",
            "",
            "  ○ badge"
        ]
    );
}

/// `tree` prints a done trail's proof and grade under its title, and no progress with no step under it
/// (`spec/02-cli.md`, "Tree").
#[test]
fn tree_prints_a_done_trails_proof_and_grade() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "a small ask"]);
    finish(&repo, short(&trail));
    assert_eq!(
        printed(&repo, &[short(&trail)]),
        [
            "✓ a small ask",
            "  repo · T",
            "  done    failed before, passes now",
            "  took    T",
        ]
    );
}

/// `tree --json` prints the trail, then one object a step with its links and state, and writes no
/// record (`spec/02-cli.md`, "Tree").
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

/// `tree` with no id prints the active trails after `ACTIVE`, newest first, a blank line between two
/// blocks, and refuses an id no step has with `no-step` (`spec/02-cli.md`, "Tree").
#[test]
fn tree_with_no_id_prints_every_trail_newest_first() {
    let repo = Repo::new();
    let first = add(&repo, &["--", "first trail"]);
    add(&repo, &["--under", short(&first), "--", "a part"]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    add(&repo, &["--", "second trail"]);
    let text = out(&cli(&repo, &["tree"]));
    let lines: Vec<String> = text.lines().map(bare).collect();
    assert_eq!(
        lines,
        [
            "ACTIVE",
            "○ second trail",
            "  repo",
            "",
            "○ first trail",
            "  repo · 0 of 1",
            "",
            "  ○ a part",
        ],
        "{text}"
    );
    refusal(&cli(&repo, &["tree", "deadbeef", "--json"]), 1, "no-step");
}

/// `tree` with no id prints the active trails after `ACTIVE` and today's others after `DONE`, one line
/// each with its project, steps, rules, worked time and id; a dropped one's reason follows it
/// (`spec/02-cli.md`, "Tree").
#[test]
fn tree_with_no_id_groups_active_and_done_trails() {
    let repo = Repo::new();
    let dropped = add(&repo, &["--", "a dropped trail"]);
    let o = cli(
        &repo,
        &["step", "drop", short(&dropped), "--", "not needed"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let ended = add(&repo, &["--", "an ended trail"]);
    let scope = format!("trail:{}", short(&ended));
    let o = cli(&repo, &["decide", "--scope", &scope, "--", "a rule"]);
    assert!(o.status.success(), "{}", err(&o));
    let part = add(&repo, &["--under", short(&ended), "--", "its part"]);
    finish(&repo, short(&part));
    add(&repo, &["--", "an active trail"]);
    let text = out(&cli(&repo, &["tree"]));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "ACTIVE", "{text}");
    assert_eq!(lines[1], "○ an active trail", "{text}");
    assert_eq!(lines[3], "", "{text}");
    assert_eq!(lines[4], "DONE", "{text}");
    let line = |what: &str| lines.iter().position(|l| l.contains(what)).unwrap();
    let ended_line = lines[line("an ended trail")];
    assert!(ended_line.starts_with("✓ an ended trail  "), "{text}");
    assert!(ended_line.contains("  repo  1 step · 1 rule  "), "{text}");
    assert!(ended_line.ends_with(short(&ended)), "{text}");
    let at = line("a dropped trail");
    assert!(lines[at].starts_with("✗ a dropped trail"), "{text}");
    assert_eq!(lines[at + 1], "  not needed", "{text}");
    assert!(!text.contains("its part"), "{text}");
}

/// `text` without its colour codes.
fn plain(text: &str) -> String {
    let mut kept = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => {
                chars.by_ref().find(|c| *c == 'm');
            }
            c => kept.push(c),
        }
    }
    kept
}

/// With `CLICOLOR_FORCE=1`, `tree` draws guides, marks, underlined ids and details at the right edge,
/// in columns; `NO_COLOR` turns the colour off and keeps the lines (`spec/02-cli.md`, "Tree").
#[test]
fn tree_draws_the_styled_tree_when_colour_is_forced() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let done = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--at",
            "shelf/prices.py:40",
            "--",
            "done part",
        ],
    );
    let open = add(&repo, &["--under", &done[..8], "--", "deeper part"]);
    let o = cli(
        &repo,
        &[
            "decide",
            "--scope",
            &format!("trail:{}", &trail[..8]),
            "--",
            "a rule",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["step", "start", &open[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let fix = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--at",
            "shelf/a.py",
            "--",
            "the fix",
        ],
    );
    let forced = [("CLICOLOR_FORCE", "1"), ("COLUMNS", "80")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8]], &forced, "");
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    assert!(text.contains("\x1b[4;"), "{text}");
    let lines: Vec<String> = plain(&text).lines().map(str::to_string).collect();
    assert_eq!(lines[0], "◐ the trail", "{lines:?}");
    assert!(
        lines[1].starts_with("  repo · 0 of 3 · ") && lines[1].contains(" · § 1 rule  "),
        "{lines:?}"
    );
    assert_eq!(lines[2], "", "{lines:?}");
    assert!(lines[3].starts_with("  ○ done part  "), "{lines:?}");
    assert!(lines[4].starts_with("  │ ◐ deeper part  "), "{lines:?}");
    assert!(lines[5].starts_with("  ○ the fix  "), "{lines:?}");
    assert_eq!(lines.len(), 6, "{lines:?}");
    let column = |line: &str, text: &str| line.find(text).map(|n| line[..n].chars().count());
    assert_eq!(column(&lines[3], "prices.py:40"), column(&lines[5], "a.py"));
    for (line, id) in [(1, &trail), (3, &done), (4, &open), (5, &fix)] {
        assert_eq!(lines[line].chars().count(), 80, "{lines:?}");
        assert!(lines[line].ends_with(&id[..8]), "{lines:?}");
    }
    let o = repo.run(&repo.path(), &["tree"], &forced, "");
    let text = plain(&out(&o));
    assert!(text.starts_with("ACTIVE\n◐ the trail\n"), "{text}");
    let off = [("CLICOLOR_FORCE", "1"), ("NO_COLOR", "1")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8]], &off, "");
    assert!(!out(&o).contains('\x1b'), "{}", out(&o));
    assert!(out(&o).starts_with("◐ the trail\n"), "{}", out(&o));
}

/// The full tree starts each line under a step at the column of its title, after its label, and wraps a
/// long line and a rule at the width (`spec/02-cli.md`, "Tree").
#[test]
fn the_full_tree_aligns_and_wraps_the_lines_under_a_step() {
    let repo = Repo::new();
    let long =
        "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen";
    let trail = add(&repo, &["--", &format!("the trail\n{long}")]);
    let rule = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &rule, "--", long]);
    assert!(o.status.success(), "{}", err(&o));
    let rule = out(&o).trim()[..8].to_string();
    let first = add(
        &repo,
        &["--under", &trail[..8], "--", &format!("first\n{long}")],
    );
    std::thread::sleep(std::time::Duration::from_millis(1100));
    add(&repo, &["--under", &trail[..8], "--", "second"]);
    let o = cli(
        &repo,
        &["step", "done", &first[..8], "--as-planned", "--", long],
    );
    assert!(o.status.success(), "{}", err(&o));

    let width = [("COLUMNS", "60")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8], "--full"], &width, "");
    assert!(o.status.success(), "{}", err(&o));
    let lines: Vec<String> = out(&o).lines().map(str::to_string).collect();
    for line in &lines {
        assert!(line.chars().count() <= 60, "{lines:?}");
    }
    let block = |lead: &str, rest: &str| -> String {
        let at = lines
            .iter()
            .position(|l| l.strip_prefix(lead).is_some_and(|t| t.starts_with("one")));
        let at = at.unwrap_or_else(|| panic!("{lead:?}: {lines:?}"));
        let mut text = lines[at][lead.len()..].to_string();
        for line in &lines[at + 1..] {
            match line.strip_prefix(rest) {
                Some(more) if long.starts_with(&format!("{text} {more}")) => {
                    text = format!("{text} {more}")
                }
                _ => break,
            }
        }
        assert!(text.len() > 60 - lead.len(), "{lead}: {lines:?}");
        text
    };
    assert_eq!(block("  ", "  "), long);
    let label = format!("  § {rule} ");
    assert_eq!(block(&label, &" ".repeat(label.chars().count())), long);
    assert_eq!(block("    ", "    "), long);
    assert_eq!(block("    done    ", "            "), long);
}

/// The short tree counts a trail's rules and leaves out their text, and the full tree prints it whole
/// (`spec/02-cli.md`, "Tree").
#[test]
fn the_short_tree_counts_rules_and_leaves_out_their_text() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let rule = "When you price: use cents, because floats lost a cent.";
    let scope = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    let short = out(&cli(&repo, &["tree", &trail[..8]]));
    assert!(short.contains(" · § 1 rule"), "{short}");
    assert!(!short.contains("When you price"), "{short}");
    let full = out(&cli(&repo, &["tree", &trail[..8], "--full"]));
    assert!(full.contains(rule), "{full}");
}

/// The short tree prints a done step's `changed` tag under it as `~ <changed>`, and marks a revised step
/// `↻` (`spec/02-cli.md`, "Tree").
#[test]
fn the_short_tree_prints_what_changed_and_marks_a_revised_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &part[..8],
            "--changed",
            "one file, not two",
            "--",
            "it works",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    add(&repo, &["--under", &trail[..8], "--", "next"]);
    let text = out(&cli(&repo, &["tree", &trail[..8]]));
    let lines: Vec<String> = text.lines().map(bare).collect();
    assert_eq!(lines[3], "  ✓ the part", "{text}");
    assert_eq!(lines[4], "    ~ one file, not two", "{text}");
    let other = add(&repo, &["--", "another trail"]);
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--under",
            &other[..8],
            "--replaces",
            &part[..8],
            "--",
            "a better part",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&cli(&repo, &["tree", &trail[..8]]));
    assert!(
        text.lines().any(|l| l.starts_with("  ↻ the part")),
        "{text}"
    );
}

/// `tree <id>` prints a trail with no record from today, and an ended one whole (`spec/02-cli.md`,
/// "Tree").
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
    let part = add(&repo, &["--under", &ended[..8], "--", "its part"]);
    finish(&repo, &part[..8]);
    let text = out(&cli(&repo, &["tree"]));
    assert!(!text.contains("an old trail"), "{text}");
    let o = cli(&repo, &["tree", &ended[..8]]);
    assert!(out(&o).contains("its part"), "{}", out(&o));
    let o = cli(&repo, &["tree", &old.id[..8]]);
    assert!(out(&o).contains("an old trail"), "{}", out(&o));
}

/// `tree` prints the short tree, with no line under a step, and `--full` adds the proof and grade; with
/// `COLUMNS`, the short tree fits each line to that width: it wraps a block's title, cuts a step's
/// title with `…` and keeps its time (`spec/02-cli.md`, "Tree").
#[test]
fn tree_prints_the_short_tree_and_the_full_tree_with_full() {
    let repo = Repo::new();
    let title = "a trail whose words run well past thirty characters";
    let trail = add(&repo, &["--", title]);
    let part = add(
        &repo,
        &["--under", &trail[..8], "--", "a part whose words run long"],
    );
    finish(&repo, &part[..8]);
    let short = out(&cli(&repo, &["tree", &trail[..8]]));
    assert_eq!(short.lines().count(), 4, "{short}");
    let full = out(&cli(&repo, &["tree", &trail[..8], "--full"]));
    assert!(
        full.lines().any(|l| l.trim_start().starts_with("done ")),
        "{full}"
    );
    let o = repo.run(
        &repo.path(),
        &["tree", &trail[..8]],
        &[("COLUMNS", "30")],
        "",
    );
    let cut = out(&o);
    assert!(cut.lines().all(|l| l.chars().count() <= 30), "{cut}");
    let lines: Vec<&str> = cut.lines().collect();
    let head: Vec<&str> = lines
        .iter()
        .take_while(|l| !l.starts_with("  repo"))
        .map(|l| l.trim_start_matches("✓ ").trim())
        .collect();
    assert_eq!(head.join(" "), title, "{cut}");
    let step = lines.last().unwrap();
    assert!(step.contains('…') && step.ends_with('s'), "{cut}");
}

/// `tree` with no id prints today's trails of every project, in any folder, each with its project name;
/// with none, it names `tree --all` (`spec/02-cli.md`, "Tree").
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
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let shop = repo.other("shopping");
    let o = repo.run(&shop, &["step", "add", "--", "a shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let elsewhere = repo.other("elsewhere");
    let o = repo.run(&elsewhere, &["tree"], &[], "");
    let text = out(&o);
    let lines: Vec<String> = text.lines().map(bare).collect();
    assert_eq!(
        lines,
        [
            "ACTIVE",
            "○ a shop trail",
            "  shopping",
            "",
            "○ a repo trail",
            "  repo"
        ],
        "{text}"
    );
}

/// `tree --all` prints every trail in the store, of every project, newest first, one line each, each
/// day's trails after a line with the day (`spec/02-cli.md`, "Tree").
#[test]
fn tree_all_prints_every_trail_of_every_project_by_day() {
    let repo = Repo::new();
    let past = support::step_note(
        &repo,
        &support::account(),
        None,
        "an old trail",
        support::now() - 2 * support::DAY,
    );
    let shop = repo.other("shop");
    let o = repo.run(&shop, &["step", "add", "--", "a shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["tree"]);
    assert!(!out(&o).contains("an old trail"), "{}", out(&o));
    let o = cli(&repo, &["tree", "--all"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let lines: Vec<&str> = text.lines().collect();
    let at = |what: &str| lines.iter().position(|l| l.contains(what));
    let (shop, old) = (at("a shop trail").unwrap(), at(&past.id[..8]).unwrap());
    assert!(shop < old, "{text}");
    for trail in [shop, old] {
        assert!(lines[trail - 1].starts_with(char::is_alphabetic), "{text}");
    }
    assert!(lines[shop].starts_with("○ a shop trail  "), "{text}");
    assert!(lines[shop].contains("  shop  "), "{text}");
    refusal(
        &cli(&repo, &["tree", &past.id[..8], "--all", "--json"]),
        2,
        "usage",
    );
}

/// The tree prints a trail's first ask as the trail when their texts match, and the ask's steps one
/// level up; `--json` prints both (`spec/02-cli.md`, "Tree").
#[test]
fn the_tree_folds_a_trails_twin_first_ask() {
    let repo = Repo::new();
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(&repo.path(), &["step", "add", "--", "the trail"], &env, "");
    let ids: Vec<String> = out(&o).lines().map(str::to_string).collect();
    let (ask, trail) = (&ids[0], &ids[1]);
    let part = add(&repo, &["--under", &ask[..8], "--", "the part"]);
    let text = out(&cli(&repo, &["tree", &trail[..8]]));
    assert!(!text.contains(&ask[..8]), "{text}");
    assert!(
        text.lines()
            .any(|l| l.starts_with("  ○ the part  ") && l.ends_with(&part[..8])),
        "{text}"
    );
    let json = out(&cli(&repo, &["tree", &trail[..8], "--json"]));
    assert!(json.contains(ask.as_str()), "{json}");
}

/// In colour, a step whose newest record is less than 2 seconds old prints its title bold
/// (`spec/02-cli.md`, "Tree").
#[test]
fn the_tree_prints_a_fresh_step_bold() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
    finish(&repo, &part[..8]);
    let forced = [("CLICOLOR_FORCE", "1")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8]], &forced, "");
    assert!(out(&o).contains("\x1b[1mthe part"), "{:?}", out(&o));
    std::thread::sleep(std::time::Duration::from_millis(2100));
    let o = repo.run(&repo.path(), &["tree", &trail[..8]], &forced, "");
    assert!(out(&o).contains("\x1b[2mthe part"), "{:?}", out(&o));
}

/// Under 80 columns the short tree leaves out the ids; then, while a title would be cut below 20
/// characters, the first column, then the time (`spec/02-cli.md`, "Tree").
#[test]
fn a_narrow_tree_leaves_out_ids_then_projects_then_times() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "a trail with a long title of many words"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "its part"]);
    finish(&repo, &part[..8]);
    let at = |columns: &str| {
        let o = repo.run(&repo.path(), &["tree"], &[("COLUMNS", columns)], "");
        let text = out(&o);
        let line = text
            .lines()
            .find(|l| l.starts_with("✓ "))
            .unwrap()
            .to_string();
        line
    };
    let wide = at("100");
    assert!(
        wide.ends_with(&trail[..8]) && wide.contains(" repo "),
        "{wide}"
    );
    let narrow = at("70");
    assert!(
        !narrow.contains(&trail[..8]) && narrow.contains(" repo "),
        "{narrow}"
    );
    assert!(narrow.ends_with('s'), "{narrow}");
    let tight = at("38");
    assert!(!tight.contains(" repo ") && tight.ends_with('s'), "{tight}");
    let tiny = at("30");
    assert!(!tiny.ends_with('s') && tiny.contains("1 step"), "{tiny}");
}
