//! CIP-05, "The log".

mod support;

use support::{add, cli, Repo};

/// The lines of `$CELIS_HOME/log`; none when it does not exist.
fn logged(repo: &Repo) -> Vec<String> {
    std::fs::read_to_string(repo.home().join("log"))
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

/// A run that fails adds a line with its time, exit code, words and message, and leaves out the flags,
/// their values and the text after `--`; a run that succeeds adds none (`spec/05-reports.md`,
/// "The log").
#[test]
fn a_failed_run_adds_its_words_and_message_to_the_log() {
    let repo = Repo::new();
    add(&repo, &["--", "a task"]);
    assert_eq!(logged(&repo), Vec::<String>::new());
    let args = [
        "step",
        "done",
        "ffffffff",
        "--changed",
        "secret plan",
        "--",
        "secret proof",
    ];
    assert_eq!(cli(&repo, &args).status.code(), Some(1));
    let lines = logged(&repo);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let words: Vec<&str> = lines[0].splitn(4, ' ').collect();
    assert_eq!(words[0].len(), "2026-10-13".len(), "{lines:?}");
    assert_eq!(words[1].len(), "14:02".len(), "{lines:?}");
    assert_eq!(words[2], "1", "{lines:?}");
    assert!(
        words[3].starts_with("celis step done ffffffff · no step has the id ffffffff"),
        "{lines:?}"
    );
    assert!(!lines[0].contains("secret"), "{lines:?}");
    assert_eq!(cli(&repo, &["nope"]).status.code(), Some(2));
    assert_eq!(logged(&repo).len(), 2);
}
