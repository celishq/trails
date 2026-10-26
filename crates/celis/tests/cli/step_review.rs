//! The review `done` prints.

use crate::support;

use celis_nostr::Note;
use support::{account, add, cli, err, mark_note, now, out, put, step_note, Repo};

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

/// `done` on an ask prints the end to fill and give the person: a line for each decision, each step whose
/// proof is the agent's own claim, what no check proves outside its places, each changed step, the new
/// intent and each choice, then a total of its steps, proofs by kind and minutes; an old `by judge` counts
/// as judged (`spec/03-steps.md`, "Closing").
#[test]
fn done_on_an_ask_prints_the_end_to_fill() {
    let repo = Repo::new();
    let by = &account();
    let began = now() - 10 * 60 - 30;
    let ask = step_note(&repo, by, None, "the ask", began);
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
    let tags: [&[&str]; 4] = [
        &["L", "celis.step"],
        &["l", "done", "celis.step"],
        &["e", &judged],
        &["by", "judge"],
    ];
    let tags: Vec<Vec<String>> = tags
        .iter()
        .map(|t| t.iter().map(|v| v.to_string()).collect())
        .collect();
    put(
        &repo,
        &Note::write(by, 1985u16, now(), tags, "met").unwrap(),
    );
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
            "- [!] **Not proven:** <what no check proves, and each file git diff --stat shows changed outside a.rs, b.rs>".to_string(),
            "- [+] **I chose:** claim: it took the cache · <the pattern that decided it, or none>".to_string(),
            "- [+] **I chose:** <each choice you took yourself> · <the pattern that decided it, or none>".to_string(),
            "- [+] **Learned:** <each pattern you wrote, as its id and When> · <the guessed why, or none>".to_string(),
            String::new(),
            "celis · 4 steps · 1 checked · 1 judged · 1 person · 1 not proven · 10m".to_string(),
        ]
    );
}

/// The end of an ask whose steps share one kind of proof totals them as all of that kind.
#[test]
fn an_end_whose_steps_share_one_proof_totals_them_as_all() {
    let repo = Repo::new();
    let ask = add(&repo, &["--check", "true", "--", "the ask"]);
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
    format!("celis: before you answer, learn from {a}. Write a pattern with celis pattern for each choice of the work that will come back, such as those below; a wish or a correction that says always, never or from now on is one too. Its When names the work, never the person or the project: --scope says where it holds. Its Because says why the choice is good for the work; \"the person asked\" is not a why. Take it from the person's words or the work; when neither gives it, write your best guess, and name it on the Learned line for the person to confirm. A reaction to a result, such as \"this one\" or \"too busy\", is a pattern of the person's taste: --good the result they liked and --bad the one they did not, each a step id, a file or a URL. Give --path for the files it acts on, --within for the larger pattern it completes, --scope me when it holds in any project, and --replaces <id> for a pattern that misled you or a near copy. Write nothing when the work taught nothing. What holds only for this trail is a note: celis step note {a}.")
}

/// `done` of an ask begins with the learn brief: each `changed` of the ask and of its steps, each note of
/// its trail and each prompt on it since the ask began, the one that began it first
/// (`spec/02-cli.md`, "Step").
#[test]
fn done_on_an_ask_begins_with_the_learn_brief() {
    let repo = Repo::new();
    let by = &account();
    let began = now() - 60;
    let ask = step_note(&repo, by, None, "the ask", began);
    mark_note(&repo, by, "prompt", &ask.id, "make it red", began - 9);
    mark_note(&repo, by, "prompt", &ask.id, "make it blue", began - 1);
    mark_note(&repo, by, "prompt", &ask.id, "and bigger", began + 1);
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

/// `done` of a step that fixes done work ends with the learn brief, with what the fix changed, before
/// the next move; `done` of another step gives none (`spec/02-cli.md`, "Step").
#[test]
fn done_of_a_fix_of_done_work_ends_with_the_learn_brief() {
    let repo = Repo::new();
    let ask = add(&repo, &["--", "the ask"]);
    let a = &ask[..8];
    let first = add(&repo, &["--under", a, "--", "the blurb"]);
    assert!(!done(&repo, &first, &["--as-planned"])
        .iter()
        .any(|l| l.starts_with("celis: before you answer")));
    let fix = add(
        &repo,
        &[
            "--under",
            a,
            "--replaces",
            &first[..8],
            "--",
            "a shorter blurb",
        ],
    );
    let lines = done(&repo, &fix, &["--changed", "three short sentences"]);
    let brief = lines
        .iter()
        .position(|l| l.starts_with("celis: before you answer"))
        .unwrap();
    assert_eq!(lines[brief], learn(&fix[..8]));
    assert_eq!(
        lines[brief + 1],
        format!("  changed {} three short sentences", &fix[..8])
    );
    assert!(
        lines
            .last()
            .unwrap()
            .starts_with("celis: it went otherwise"),
        "{lines:?}"
    );
}

/// `done --by` takes only `person` (`spec/03-steps.md`, "Closing"; `spec/02-cli.md`, "Step").
#[test]
fn done_by_takes_only_person() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    for who in ["robot", "judge"] {
        let args = [
            "step",
            "done",
            &trail[..8],
            "--as-planned",
            "--by",
            who,
            "--",
            "p",
        ];
        let o = cli(&repo, &args);
        assert_eq!(o.status.code(), Some(2), "{who}: {}", err(&o));
    }
    done(&repo, &trail, &["--as-planned", "--by", "person"]);
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

/// `done` of an ask with no `--by` closes it while git shows a changed file.
#[test]
fn done_of_an_ask_with_changed_files_closes_it() {
    let repo = Repo::new();
    let ask = add(&repo, &["--", "Round the prices\nexpect: cents"]);
    repo.write("new.py", "x = 1\n");
    done(&repo, &ask, &["--as-planned"]);
}

/// With no git, the `--file` of a `done` count as its changed files, and `done` prints the patterns of
/// those files (`spec/02-cli.md`, "Step").
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
    let ask = add(&repo, &["--", "Round the prices"]);
    let piece = add(&repo, &["--under", &ask[..8], "--", "piece"]);
    repo.write("new.py", "x = 1\n");
    let said = done(&repo, &piece, &["--as-planned", "--file", "new.py"]).join("\n");
    assert!(said.contains(&pattern[..8]), "{said}");
}
