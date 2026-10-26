//! The review `done` prints.

use crate::support;

use support::{account, add, cli, err, mark_note, now, out, step_note, Repo};

/// The stderr lines of `celis step done <id> <how> -- proof`, which must succeed.
fn done(repo: &Repo, id: &str, how: &[&str]) -> Vec<String> {
    let mut args = vec!["step", "done", &id[..8]];
    args.extend(how);
    args.extend(["--", "proof"]);
    let o = cli(repo, &args);
    assert!(o.status.success(), "{}", err(&o));
    err(&o).lines().map(str::to_string).collect()
}

/// `done` as planned prints the step's expect line and the open steps beside it, then asks for the
/// next step under its `under`.
#[test]
fn done_as_planned_reviews_the_step_and_names_the_next_move() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the ask"]);
    let piece = add(
        &repo,
        &["--under", &ask[..8], "--", "piece\nexpect: it prints 3"],
    );
    let next = add(&repo, &["--under", &ask[..8], "--", "next"]);
    let (a, p, n) = (&ask[..8], &piece[..8], &next[..8]);
    assert_eq!(
        done(&repo, &piece, &["--as-planned"]),
        [
            format!("celis: {p} is done; compare the result with its plan:"),
            "  expect  it prints 3".to_string(),
            format!("  open    {n} next"),
            format!("celis: take an open step beside it, add one step under {a}, or close {a} when its work is done."),
        ]
    );
}

/// `done --changed` prints what changed, and asks to reshape the plan before the next step.
#[test]
fn done_changed_asks_to_reshape_the_plan() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the ask"]);
    let piece = add(&repo, &["--under", &ask[..8], "--", "piece"]);
    let (a, p) = (&ask[..8], &piece[..8]);
    assert_eq!(
        done(&repo, &piece, &["--changed", "it printed 4"]),
        [
            format!("celis: {p} is done; compare the result with its plan:"),
            format!("  changed {p} it printed 4"),
            format!("celis: it went otherwise than planned; replace or drop each open step it voids, then add one step under {a}."),
        ]
    );
}

/// A trail's review names no next move.
#[test]
fn a_trails_review_names_no_next_move() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    assert_eq!(
        done(&repo, &trail, &["--as-planned"]),
        [format!(
            "celis: {} is done; compare the result with its plan:",
            &trail[..8]
        )]
    );
}

/// `done` on an ask prints the end to fill and give the person: a line for each decision, each step whose
/// proof is the agent's own claim, what the judge could not check outside its places, each changed step,
/// the new intent and each choice, then a total of its steps, proofs by kind and minutes.
#[test]
fn done_on_an_ask_prints_the_end_to_fill() {
    let repo = Repo::new();
    let by = &account();
    let began = now() - 10 * 60 - 30;
    let trail = step_note(&repo, by, None, "the trail", began);
    let ask = step_note(&repo, by, Some(&trail), "the ask", began);
    let a = &ask.id[..8];
    let checked = add(
        &repo,
        &[
            "--under", a, "--at", "a.rs", "--check", "true", "--", "checked",
        ],
    );
    let judged = add(&repo, &["--under", a, "--at", "b.rs", "--", "judged"]);
    let claim = add(&repo, &["--under", a, "--", "claim"]);
    let gone = add(&repo, &["--under", a, "--", "gone"]);
    let edit = cli(
        &repo,
        &["step", "edit", a, "--", "the ask\nexpect: it ends"],
    );
    assert!(edit.status.success(), "{}", err(&edit));
    done(&repo, &checked, &["--as-planned"]);
    done(&repo, &judged, &["--as-planned", "--by", "judge"]);
    done(&repo, &claim, &["--changed", "it took the cache"]);
    assert!(
        cli(&repo, &["step", "drop", &gone[..8], "--", "not wanted"])
            .status
            .success()
    );
    assert_eq!(
        done(&repo, &ask.id, &["--as-planned", "--by", "person"]),
        [
            learn(a),
            format!("  changed {} it took the cache", &claim[..8]),
            format!("celis: the ask {a} is done. When you have kept what it taught, answer the person with the lines below alone: fill each <…>, and remove each line you do not need."),
            String::new(),
            "**<what was done, in one line>**".to_string(),
            String::new(),
            "- [?] **Needs you:** <each decision only the person can take>".to_string(),
            "- [!] **Not proven:** claim".to_string(),
            "- [!] **Not proven:** <what the judge could not check, and each file git diff --stat shows changed outside a.rs, b.rs>".to_string(),
            "- [+] **I chose:** claim: it took the cache · <the pattern that decided it, or none>".to_string(),
            "- [+] **I chose:** <each choice you took yourself> · <the pattern that decided it, or none>".to_string(),
            "- [+] **Learned:** <each pattern you wrote, as its id and When>".to_string(),
            String::new(),
            "celis · 4 steps · 1 checked · 1 judged · 1 person · 1 not proven · 10m".to_string(),
        ]
    );
}

/// The end of an ask whose steps share one kind of proof totals them as all of that kind.
#[test]
fn an_end_whose_steps_share_one_proof_totals_them_as_all() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(
        &repo,
        &["--under", &trail[..8], "--check", "true", "--", "the ask"],
    );
    let part = add(
        &repo,
        &["--under", &ask[..8], "--check", "true", "--", "part"],
    );
    done(&repo, &part, &["--as-planned"]);
    let lines = done(&repo, &ask, &["--as-planned"]);
    assert!(
        lines.contains(&"celis · 2 steps · all checked · 0m".to_string()),
        "{lines:?}"
    );
}

/// The head of the learn brief of the ask `a`.
fn learn(a: &str) -> String {
    format!("celis: before you answer, learn from the ask {a}. Write a pattern with celis pattern for each choice of the ask that will come back, such as those below. A wish or a correction that says always, never or from now on is a pattern whose forces say why. A reaction to a result, such as \"this one\" or \"too busy\", keeps the person's taste with --good and --bad. Give --path for the files it acts on, --within for the larger pattern it completes, --scope me when it holds in any project, and --replaces <id> for a pattern that misled you or a near copy. Write nothing when the ask taught nothing. What holds only for this trail is a note: celis step note {a}.")
}

/// `done` of an ask begins with the learn brief: each `changed` of the ask and of its steps, each note of
/// its trail and each prompt on its trail since the ask began, the one that began it first
/// (`spec/02-cli.md`, "Step").
#[test]
fn done_on_an_ask_begins_with_the_learn_brief() {
    let repo = Repo::new();
    let by = &account();
    let began = now() - 60;
    let trail = step_note(&repo, by, None, "the trail", began - 10);
    mark_note(&repo, by, "prompt", &trail.id, "make it red", began - 9);
    mark_note(&repo, by, "prompt", &trail.id, "make it blue", began - 1);
    let ask = step_note(&repo, by, Some(&trail), "the ask", began);
    mark_note(&repo, by, "prompt", &trail.id, "and bigger", began + 1);
    let a = &ask.id[..8];
    let part = add(&repo, &["--under", a, "--", "part"]);
    done(&repo, &part, &["--changed", "it took the cache"]);
    let note = cli(
        &repo,
        &[
            "step",
            "note",
            a,
            "--",
            "keep the old table\nuntil the release",
        ],
    );
    assert!(note.status.success(), "{}", err(&note));
    let lines = done(
        &repo,
        &ask.id,
        &["--changed", "it went blue", "--by", "person"],
    );
    let end = lines
        .iter()
        .position(|l| l.starts_with("celis: the ask "))
        .unwrap();
    assert_eq!(
        lines[..end],
        [
            learn(a),
            format!("  changed {} it took the cache", &part[..8]),
            format!("  changed {a} it went blue"),
            "  note    keep the old table".to_string(),
            "          until the release".to_string(),
            "  asked   make it blue".to_string(),
            "  asked   and bigger".to_string(),
        ]
    );
}

/// `done --by` takes only `judge` or `person` (`spec/03-steps.md`, "Closing").
#[test]
fn done_by_takes_judge_or_person() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &trail[..8],
            "--as-planned",
            "--by",
            "robot",
            "--",
            "p",
        ],
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `done` of a step prints each pattern of the files git shows changed, and leaves out the rest
/// (`spec/02-cli.md`, "Step").
#[test]
fn done_prints_the_patterns_of_the_changed_files() {
    let repo = Repo::new();
    let python = cli(
        &repo,
        &[
            "pattern",
            "--path",
            "*.py",
            "--",
            "When Python changes: run ruff.",
        ],
    );
    let rust = cli(
        &repo,
        &[
            "pattern",
            "--path",
            "*.rs",
            "--",
            "When Rust changes: run fmt.",
        ],
    );
    let (python, rust) = (out(&python), out(&rust));
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the ask"]);
    let piece = add(&repo, &["--under", &ask[..8], "--", "piece"]);
    repo.write("new.py", "x = 1\n");
    let said = done(&repo, &piece, &["--as-planned"]).join("\n");
    assert!(said.contains(&python[..8]), "{said}");
    assert!(!said.contains(&rust[..8]), "{said}");
}

/// When no step is open under the ask, `done` of its last step prints the judge brief: the ask, its
/// intent lines, the patterns of the changed files, each once, the changed files, and its check
/// (`spec/02-cli.md`, "Step").
#[test]
fn done_of_the_last_step_prints_the_judge_brief() {
    let repo = Repo::new();
    let pattern = cli(
        &repo,
        &[
            "pattern",
            "--path",
            "*.py",
            "--",
            "When Python changes: run ruff.",
        ],
    );
    let pattern = out(&pattern);
    let trail = add(&repo, &["--", "the trail"]);
    let intent = "Round the prices\nexpect: prices show cents\nfor: the shop\nnot: the cart";
    let ask = add(
        &repo,
        &["--under", &trail[..8], "--check", "true", "--", intent],
    );
    let piece = add(&repo, &["--under", &ask[..8], "--", "piece"]);
    repo.write("new.py", "x = 1\n");
    let said = done(&repo, &piece, &["--as-planned"]).join("\n");
    assert!(said.contains("celis:judge"), "{said}");
    for line in [
        "  ask     Round the prices",
        "  expect  prices show cents",
        "  for     the shop",
        "  not     the cart",
        "  file    new.py",
        "  check   true",
    ] {
        assert!(said.contains(line), "{line}: {said}");
    }
    assert!(
        said.contains(&format!("  pattern {}", &pattern[..8])),
        "{said}"
    );
    assert_eq!(said.matches(&pattern[..8]).count(), 1, "{said}");
}

/// `pattern` with no scope says that the pattern holds in the project, and names `--scope me` and
/// `step note`.
#[test]
fn pattern_with_no_scope_names_the_scopes() {
    let repo = Repo::new();
    let o = cli(&repo, &["pattern", "--", "When Python changes: run ruff."]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(err(&o).contains("--scope me"), "{}", err(&o));
    assert!(err(&o).contains("celis step note"), "{}", err(&o));
    let o = cli(
        &repo,
        &[
            "pattern",
            "--scope",
            "me",
            "--",
            "When you push: run the tests.",
        ],
    );
    assert!(!err(&o).contains("--scope me"), "{}", err(&o));
}

/// `done` of an ask with no `--by`, while git shows a changed file, refuses with `unjudged`, writes
/// nothing, and prints the judge brief with the patterns of the changed files, once, and the trail's
/// notes; `--by judge` closes it (`spec/02-cli.md`, "Step").
#[test]
fn done_of_an_ask_with_changed_files_needs_a_judge() {
    let repo = Repo::new();
    let pattern = cli(
        &repo,
        &[
            "pattern",
            "--path",
            "*.py",
            "--",
            "When Python changes: run ruff.",
        ],
    );
    let pattern = out(&pattern);
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--",
            "Round the prices\nexpect: cents",
        ],
    );
    let note = cli(
        &repo,
        &["step", "note", &trail[..8], "--", "keep the old table"],
    );
    assert!(note.status.success(), "{}", err(&note));
    repo.write("new.py", "x = 1\n");
    let args = [
        "step",
        "done",
        &ask[..8],
        "--as-planned",
        "--json",
        "--",
        "proof",
    ];
    let o = cli(&repo, &args);
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(err(&o).contains("\"error\":\"unjudged\""), "{}", err(&o));
    assert!(err(&o).contains("celis:judge"), "{}", err(&o));
    assert!(err(&o).contains("  expect  cents"), "{}", err(&o));
    assert_eq!(err(&o).matches(&pattern[..8]).count(), 1, "{}", err(&o));
    assert!(
        err(&o).contains("  note    keep the old table"),
        "{}",
        err(&o)
    );
    let by = [
        "step",
        "done",
        &ask[..8],
        "--as-planned",
        "--by",
        "judge",
        "--",
        "met",
    ];
    let o = cli(&repo, &by);
    assert!(o.status.success(), "{}", err(&o));
}

/// With no git, the `--file` of a `done` count as its changed files: an ask that names one needs a
/// judge, and gets the patterns of that file (`spec/02-cli.md`, "Step").
#[test]
fn the_files_of_a_done_count_as_changed_without_git() {
    let repo = Repo::new();
    std::fs::remove_dir_all(repo.path().join(".git")).unwrap();
    let pattern = cli(
        &repo,
        &[
            "pattern",
            "--path",
            "*.py",
            "--",
            "When Python changes: run ruff.",
        ],
    );
    let pattern = out(&pattern);
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "Round the prices"]);
    repo.write("new.py", "x = 1\n");
    let args = [
        "step",
        "done",
        &ask[..8],
        "--as-planned",
        "--file",
        "new.py",
        "--",
        "proof",
    ];
    let o = cli(&repo, &args);
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(err(&o).contains(&pattern[..8]), "{}", err(&o));
}

/// A step with steps under it has its own check, and the judge confirms its `expect`: `done` of its last
/// step prints its brief, and its own `done` with no `--by`, while git shows a changed file, refuses with
/// `unjudged` (`spec/03-steps.md`, "Closing"; `spec/02-cli.md`, "Step").
#[test]
fn a_step_that_holds_steps_needs_a_judge() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the shop"]);
    let intent = "Round the prices\nexpect: prices show cents";
    let part = add(
        &repo,
        &["--under", &ask[..8], "--check", "true", "--", intent],
    );
    let piece = add(&repo, &["--under", &part[..8], "--", "piece"]);
    repo.write("new.py", "x = 1\n");
    let said = done(&repo, &piece, &["--as-planned"]).join("\n");
    assert!(
        said.contains(&format!("before you close {} --by judge", &part[..8])),
        "{said}"
    );
    assert!(said.contains("  step    Round the prices"), "{said}");
    assert!(said.contains("  expect  prices show cents"), "{said}");
    assert!(said.contains("  check   true"), "{said}");
    let args = [
        "step",
        "done",
        &part[..8],
        "--as-planned",
        "--json",
        "--",
        "p",
    ];
    let o = cli(&repo, &args);
    assert!(err(&o).contains("\"error\":\"unjudged\""), "{}", err(&o));
    let by = [
        "step",
        "done",
        &part[..8],
        "--as-planned",
        "--by",
        "judge",
        "--",
        "met",
    ];
    assert!(cli(&repo, &by).status.success());
}

/// The judge brief gives the whole text of each live pattern the ask recalled and of each that a changed
/// file matches, each once, and no other pattern (`spec/02-cli.md`, "Step").
#[test]
fn the_judge_brief_gives_the_recalled_patterns_whole() {
    let repo = Repo::new();
    let decide = |args: &[&str], text: &str| {
        let mut all = vec!["pattern"];
        all.extend(args);
        all.extend(["--", text]);
        let o = cli(&repo, &all);
        assert!(o.status.success(), "{}", err(&o));
        out(&o).lines().next().unwrap().to_string()
    };
    let prices = decide(
        &[],
        "When prices change\nForces: floats are simple, but they lose cents\nDo: keep prices in cents\nBecause: floats lost a cent",
    );
    let python = decide(&["--path", "*.py"], "When Python changes: run ruff.");
    let other = decide(&[], "When a logo is drawn: keep it flat.");
    let ids = add(
        &repo,
        &[
            "--session",
            "s1",
            "--recalled",
            &prices[..8],
            "--recalled",
            &python[..8],
            "--",
            "Round the prices\nexpect: cents",
        ],
    );
    let ask = ids.lines().next().unwrap();
    repo.write("new.py", "x = 1\n");
    let o = cli(
        &repo,
        &["step", "done", &ask[..8], "--as-planned", "--", "proof"],
    );
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let said = err(&o);
    let brief = &said[said.find("celis: before you close").unwrap()..];
    let lines: Vec<&str> = brief.lines().collect();
    let at = |head: &str| lines.iter().filter(|l| l.starts_with(head)).count();
    assert_eq!(
        at(&format!(
            "  pattern {} project When prices change",
            &prices[..8]
        )),
        1,
        "{brief}"
    );
    assert_eq!(
        at("          Forces: floats are simple, but they lose cents"),
        1,
        "{brief}"
    );
    assert_eq!(at("          Because: floats lost a cent"), 1, "{brief}");
    assert_eq!(at(&format!("  pattern {}", &python[..8])), 1, "{brief}");
    assert!(!brief.contains(&other[..8]), "{brief}");
}
