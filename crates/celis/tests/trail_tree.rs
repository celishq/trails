//! CIP-14, "The commands": `trail tree`.

mod support;

use support::{add, cli, err, finish, objects, out, refusal, Repo};

fn short(id: &str) -> &str {
    &id[..8]
}

/// The lines `tree --full <args>` prints, each row without its details and each grade line cut to
/// `grade`, since times vary.
fn printed(repo: &Repo, args: &[&str]) -> Vec<String> {
    let mut all = vec!["tree", "--full"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
        .lines()
        .map(|l| match l.find("took ") {
            Some(n) if l[..n].trim_matches([' ', '│']).is_empty() => format!("{}grade", &l[..n]),
            _ => bare(l),
        })
        .collect()
}

/// `line` without the details after a row's title: from the first two spaces after its id.
fn bare(line: &str) -> String {
    let id = line
        .char_indices()
        .find(|(n, _)| {
            line.get(*n..*n + 8)
                .is_some_and(|id| id.bytes().all(|b| b.is_ascii_hexdigit()))
                && line[*n..]
                    .get(8..)
                    .is_some_and(|rest| rest.starts_with(' '))
        })
        .map(|(n, _)| n + 8);
    match id.and_then(|at| line[at..].find("  ").map(|n| at + n)) {
        Some(end) => line[..end].to_string(),
        None => line.trim_end().to_string(),
    }
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
            format!("● {} build the CRM", short(&trail)),
            format!("      rule {} colours are tokens", short(&rule)),
            format!("  ├─ ✓ {} colours", short(&colours)),
            "  │        done lint failed, passes".to_string(),
            "  │        grade".to_string(),
            format!("  └─ ○ {} customers page", short(&page)),
            format!("     └─ ○ {} badge", short(&badge)),
        ]
    );
}

/// `tree <id>` of an ask prints its trail's rules under its line (`spec/trails/14-trails.md`, "The
/// commands").
#[test]
fn tree_of_an_ask_prints_its_trails_rules() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "build the CRM"]);
    let scope = format!("trail:{}", short(&trail));
    let o = cli(
        &repo,
        &["decide", "--scope", &scope, "--", "colours are tokens"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rule = out(&o).trim().to_string();
    let ask = add(&repo, &["--under", short(&trail), "--", "customers page"]);
    let badge = add(&repo, &["--under", short(&ask), "--", "badge"]);
    assert_eq!(
        printed(&repo, &[short(&ask)]),
        [
            format!("● {} customers page", short(&ask)),
            format!("      rule {} colours are tokens", short(&rule)),
            format!("  └─ ○ {} badge", short(&badge)),
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
            format!("✓ {} a small ask", short(&trail)),
            "      done failed before, passes now".to_string(),
            "      grade".to_string(),
        ]
    );
}

/// `tree --json` prints the trail, then one object a step with its links and state, and writes no
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
    assert_eq!(lines[0]["trail"]["links"], serde_json::json!([]));
    assert_eq!(lines[1]["step"]["id"], first.as_str());
    assert_eq!(lines[1]["step"]["links"][0]["to"], trail.as_str());
    assert_eq!(lines[1]["step"]["state"], "open");
    assert_eq!(lines[2]["step"]["id"], second.as_str());
    assert_eq!(lines[2]["step"]["links"][0]["to"], first.as_str());
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
        .filter_map(|l| Some(l.strip_prefix("● ")?[..8].to_string()))
        .collect();
    assert_eq!(heads, [short(&second), short(&first)]);
    refusal(&cli(&repo, &["tree", "deadbeef", "--json"]), 1, "no-step");
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

/// With `CLICOLOR_FORCE=1`, `tree` draws branch lines, marks, underlined ids and dim columns at the right
/// edge, with no start time when a title gets short; with no id, each trail names its project; `NO_COLOR` turns
/// it off (`spec/trails/14-trails.md`, "The commands").
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
    assert!(
        lines[0].starts_with(&format!("● {} the trail ", &trail[..8])),
        "{lines:?}"
    );
    assert!(lines[1].starts_with("      rule "), "{lines:?}");
    assert!(
        lines[2].starts_with(&format!("  ├─ ○ {} done part ", &done[..8])),
        "{lines:?}"
    );
    assert!(
        lines[3].starts_with(&format!("  │  └─ ◐ {} deeper part ", &open[..8])),
        "{lines:?}"
    );
    assert!(
        lines[4].starts_with(&format!("  └─ ○ {} the fix ", &fix[..8])),
        "{lines:?}"
    );
    assert_eq!(lines.len(), 5, "{lines:?}");
    let column = |line: &str, text: &str| line.find(text).map(|n| line[..n].chars().count());
    assert_eq!(column(&lines[2], "prices.py:40"), column(&lines[4], "a.py"));
    for line in [&lines[0], &lines[3]] {
        assert_eq!(line.chars().count(), 80, "{lines:?}");
    }
    let took = |line: &str| line.rfind("s  ").map(|n| line[..n].chars().count());
    assert_eq!(took(&lines[0]), took(&lines[3]), "{lines:?}");
    let clock = |line: &str| {
        let end: Vec<char> = line.chars().rev().take(6).collect();
        end.len() == 6 && end[2] == ':' && end[5] == ' '
    };
    assert!(clock(&lines[0]), "{lines:?}");
    let narrow = [("CLICOLOR_FORCE", "1"), ("COLUMNS", "50")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8]], &narrow, "");
    let lines: Vec<String> = plain(&out(&o)).lines().map(str::to_string).collect();
    assert!(lines[0].contains(" the trail "), "{lines:?}");
    assert!(!lines.iter().any(|l| clock(l)), "{lines:?}");
    let o = repo.run(&repo.path(), &["tree"], &forced, "");
    let first = plain(&out(&o)).lines().next().unwrap_or("").to_string();
    assert!(
        first.starts_with(&format!("● {} repo the trail", &trail[..8])),
        "{first}"
    );
    let off = [("CLICOLOR_FORCE", "1"), ("NO_COLOR", "1")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8]], &off, "");
    assert!(!out(&o).contains('\x1b'), "{}", out(&o));
    assert!(
        out(&o).starts_with(&format!("● {}", &trail[..8])),
        "{}",
        out(&o)
    );
}

/// The styled full tree starts each line under a step at the column of its title, ends a label one
/// space before it, and wraps a long line at the width, keeping the branch line
/// (`spec/trails/14-trails.md`, "The commands").
#[test]
fn the_styled_full_tree_aligns_and_wraps_the_lines_under_a_step() {
    let repo = Repo::new();
    let long =
        "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen";
    let trail = add(&repo, &["--", &format!("the trail\n{long}")]);
    let rule = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &rule, "--", long]);
    assert!(o.status.success(), "{}", err(&o));
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

    let forced = [("CLICOLOR_FORCE", "1"), ("COLUMNS", "60")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8], "--full"], &forced, "");
    assert!(o.status.success(), "{}", err(&o));
    let lines: Vec<String> = plain(&out(&o)).lines().map(str::to_string).collect();
    for line in &lines {
        assert!(line.chars().count() <= 60, "{lines:?}");
    }
    let block = |from: &str, lead: &str, rest: &str| -> String {
        let at = lines.iter().position(|l| l.starts_with(from));
        let at = at.unwrap_or_else(|| panic!("{from:?}: {lines:?}"));
        let mut words = vec![lines[at][lead.len()..].to_string()];
        for line in &lines[at + 1..] {
            match line.strip_prefix(rest) {
                Some(more) if !more.starts_with(' ') => words.push(more.to_string()),
                _ => break,
            }
        }
        assert!(words.len() > 1, "{from}: {lines:?}");
        words.join(" ")
    };
    let under_trail = " ".repeat("● 12345678 ".chars().count());
    assert_eq!(
        block(&format!("{under_trail}one"), &under_trail, &under_trail),
        long
    );
    let label = format!("{}rule ", " ".repeat(6));
    let ruled = block(&label, &label, &under_trail);
    assert!(ruled.ends_with(long), "{ruled}");
    let rail = format!("  │  {}", " ".repeat("✓ 12345678 ".chars().count()));
    assert_eq!(block(&format!("{rail}one"), &rail, &rail), long);
    let done = format!(
        "  │{}done ",
        " ".repeat(rail.chars().count() - "  │done ".chars().count())
    );
    assert_eq!(block(&done, &done, &rail), long);
}

/// The short styled tree prints a rule without its reason, and the full tree prints it whole
/// (`spec/trails/14-trails.md`, "The commands").
#[test]
fn the_short_styled_tree_leaves_out_a_rules_reason() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let rule = "When you price: use cents, because floats lost a cent.";
    let scope = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    let forced = [("CLICOLOR_FORCE", "1"), ("COLUMNS", "100")];
    let short = plain(&out(&repo.run(
        &repo.path(),
        &["tree", &trail[..8]],
        &forced,
        "",
    )));
    assert!(short.contains("When you price: use cents\n"), "{short}");
    assert!(!short.contains("because"), "{short}");
    let full = repo.run(&repo.path(), &["tree", &trail[..8], "--full"], &forced, "");
    assert!(plain(&out(&full)).contains(rule), "{}", out(&full));
}

/// `tree` with no id leaves out a trail with no record from today, and folds a trail that is not active
/// into its own line, which counts its steps; `tree <id>` prints either whole (`spec/trails/14-trails.md`,
/// "The commands").
#[test]
fn tree_with_no_id_folds_ended_trails_and_shows_only_today() {
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
    finish(&repo, &ended[..8]);
    let active = add(&repo, &["--", "an active trail"]);
    add(&repo, &["--under", &active[..8], "--", "a live part"]);
    let o = cli(&repo, &["tree"]);
    let text = out(&o);
    assert!(!text.contains("an old trail"), "{text}");
    let line = |what: &str| text.lines().find(|l| l.contains(what)).unwrap_or("");
    assert!(line("an active trail").starts_with("● "), "{text}");
    assert!(text.contains("a live part"), "{text}");
    let ended_line = line("an ended trail");
    assert!(
        ended_line.starts_with("✓ ") && ended_line.contains("  1 step  "),
        "{text}"
    );
    assert!(!text.contains("its part"), "{text}");
    let o = cli(&repo, &["tree", &ended[..8]]);
    assert!(out(&o).contains("its part"), "{}", out(&o));
    let o = cli(&repo, &["tree", &old.id[..8]]);
    assert!(out(&o).contains("an old trail"), "{}", out(&o));
}

/// `tree` prints the short tree, with no line under a step, and `--full` adds the proof and grade; with
/// `COLUMNS`, the short tree fits each line to that width, cutting a step's text with `…` and keeping its
/// details (`spec/trails/14-trails.md`, "The commands").
#[test]
fn tree_prints_the_short_tree_and_the_full_tree_with_full() {
    let repo = Repo::new();
    let trail = add(
        &repo,
        &["--", "a trail whose words run well past thirty characters"],
    );
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    finish(&repo, &part[..8]);
    let short = out(&cli(&repo, &["tree", &trail[..8]]));
    assert_eq!(short.lines().count(), 2, "{short}");
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
    let first = cut.lines().next().unwrap_or("");
    assert_eq!(first.chars().count(), 30, "{cut}");
    assert!(first.contains('…') && first.ends_with('s'), "{cut}");
    assert!(cut.lines().all(|l| l.chars().count() <= 30), "{cut}");
}

/// `tree` with no id prints today's trails of every project, in any folder, each with its project name
/// and no project line; with none, it names `tree --all` (`spec/trails/14-trails.md`, "The commands").
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
    let shop = repo.other("shop");
    let o = repo.run(&shop, &["step", "add", "--", "a shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let elsewhere = repo.other("elsewhere");
    let o = repo.run(&elsewhere, &["tree"], &[], "");
    let text = out(&o);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert!(lines[0].contains(" shop a shop trail"), "{text}");
    assert!(lines[1].contains(" repo a repo trail"), "{text}");
}

/// `tree --all` prints every trail in the store, of every project, newest first, each day's trails
/// after a line with the day (`spec/trails/14-trails.md`, "The commands").
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
    assert!(lines[shop].contains(" shop a shop trail"), "{text}");
    refusal(
        &cli(&repo, &["tree", &past.id[..8], "--all", "--json"]),
        2,
        "usage",
    );
}

/// The tree prints a trail's first ask on the trail's line when their texts match, and the ask's steps
/// one level up; `--json` prints both (`spec/trails/14-trails.md`, "The commands").
#[test]
fn the_tree_folds_a_trails_twin_first_ask() {
    let repo = Repo::new();
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(&repo.path(), &["step", "add", "--", "the trail"], &env, "");
    let ids: Vec<String> = out(&o).lines().map(str::to_string).collect();
    let (ask, trail) = (&ids[0], &ids[1]);
    let part = add(&repo, &["--under", &ask[..8], "--", "the part"]);
    let styled = [("CLICOLOR_FORCE", "1")];
    let o = repo.run(&repo.path(), &["tree", &trail[..8]], &styled, "");
    let text = out(&o);
    assert!(!text.contains(&ask[4..8]), "{text}");
    let line = text.lines().find(|l| l.contains("the part")).unwrap();
    assert!(line.starts_with("\u{1b}[2m  └─ "), "{text}");
    let plain = out(&cli(&repo, &["tree", &trail[..8]]));
    assert!(
        !plain.contains(&ask[..8]) && plain.contains(&part[..8]),
        "{plain}"
    );
    let json = out(&cli(&repo, &["tree", &trail[..8], "--json"]));
    assert!(json.contains(ask.as_str()), "{json}");
}
