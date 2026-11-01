//! What celis does with a secret in a record or a kept file.

use crate::support;

use support::{add, cli, err, refusal, Repo};

const KEY: &str = "sk-live-7f3a9c2e81d4b06a";

/// The records in the store.
fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared"))
        .map(|d| d.count())
        .unwrap_or(0)
}

/// A pattern, a step, a note and a proof that hold a secret are refused with `secret`, and write
/// nothing (`spec/01-records.md`, "The account").
#[test]
fn a_record_that_holds_a_secret_is_refused() {
    let repo = Repo::new();
    let text = format!("When the app calls the API: send API_KEY={KEY}, because it signs in.");
    refusal(
        &cli(&repo, &["pattern", "--json", "--", &text]),
        1,
        "secret",
    );
    let title = format!("Put {KEY} in .env");
    refusal(
        &cli(&repo, &["step", "add", "--json", "--", &title]),
        1,
        "secret",
    );
    assert_eq!(records(&repo), 0);
    let step = add(&repo, &["--", "Read the API key from .env"]);
    let note = [
        "step",
        "note",
        "--json",
        &step[..8],
        "--",
        "token ghp_0123456789abcdefghijABCDEFGHIJ012345",
    ];
    refusal(&cli(&repo, &note), 1, "secret");
    let proof = [
        "step",
        "done",
        "--json",
        &step[..8],
        "--as-planned",
        "--",
        "AWS key AKIAIOSFODNN7EXAMPLE works",
    ];
    refusal(&cli(&repo, &proof), 1, "secret");
    assert_eq!(records(&repo), 1);
}

/// A kept file that holds a secret is refused with `secret`, and the step stays open
/// (`spec/01-records.md`, "The account").
#[test]
fn a_proof_file_that_holds_a_secret_is_refused() {
    let repo = Repo::new();
    repo.write(".env", &format!("API_KEY={KEY}\n"));
    repo.write(
        "config.py",
        "import os\n\n\ndef api_key():\n    return os.environ['API_KEY']\n",
    );
    let step = add(&repo, &["--", "Read the API key from .env"]);
    let done = |file: &str| {
        let args = [
            "step",
            "done",
            "--json",
            &step[..8],
            "--as-planned",
            "--file",
            file,
            "--",
            "api_key() reads it",
        ];
        cli(&repo, &args)
    };
    let o = done(".env");
    refusal(&o, 1, "secret");
    assert!(err(&o).contains("API_KEY in .env"), "{}", err(&o));
    assert_eq!(records(&repo), 1);
    let o = done("config.py");
    assert!(o.status.success(), "{}", err(&o));
}

/// A name of a secret by where it lives is no secret (`spec/01-records.md`, "The account").
#[test]
fn a_secret_named_by_where_it_lives_is_kept() {
    let repo = Repo::new();
    for text in [
        "When the app calls the API: read API_KEY in .env with os.environ['API_KEY'], because it signs in.",
        "When a password is checked: compare its hash, because password_hash_v2 is slow on purpose.",
    ] {
        let o = cli(&repo, &["pattern", "--", text]);
        assert!(o.status.success(), "{}", err(&o));
    }
}

/// The prompt hook keeps no prompt that holds a secret, and says so (`spec/02-cli.md`, "Hooks").
#[test]
fn the_prompt_hook_keeps_no_prompt_that_holds_a_secret() {
    let repo = Repo::new();
    let json =
        serde_json::json!({"session_id": "s1", "prompt": format!("Put API_KEY={KEY} in .env")});
    let o = support::prompt(&repo, &repo.path(), &[], &json.to_string());
    assert!(o.status.success(), "{}", err(&o));
    assert!(err(&o).contains("holds a secret"), "{}", err(&o));
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--session", "s1", "--", "Read the key"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(records(&repo), 1);
}
