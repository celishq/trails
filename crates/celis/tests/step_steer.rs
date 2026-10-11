//! CIP-12, "Steering".

mod support;

use celis_wire::Note;
use serde_json::Value;
use std::process::Output;
use support::{account, err, now, out, put, refusal, Repo};

fn run(repo: &Repo, args: &[&str]) -> Output {
    repo.run(&repo.path(), args, &[], "")
}

/// The id `step add <args>` prints.
fn add(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["step", "add"];
    all.extend(args);
    let o = run(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// A steer does not close its step, and a reader ignores a steer with no content
/// (`spec/trails/12-steps.md`, "Steering").
#[test]
fn a_steer_does_not_close_its_step_and_one_with_no_content_is_ignored() {
    let repo = Repo::new();
    let id = add(&repo, &["--", "badge"]);
    let o = run(
        &repo,
        &["step", "steer", &id[..8], "--", "Use the shared Badge."],
    );
    assert!(o.status.success(), "{}", err(&o));
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "steer".into(), "celis.step".into()],
        vec!["e".into(), id.clone()],
    ];
    put(
        &repo,
        &Note::write(&account(), 1985u16, now(), tags, "").unwrap(),
    );
    let read = out(&run(&repo, &["read"]));
    assert!(read.contains(&id[..8]), "{read}");
    let o = run(
        &repo,
        &["step", "done", &id[..8], "--as-planned", "--", "done"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let tree = out(&run(&repo, &["tree", &id[..8], "--full"]));
    assert!(tree.contains(" · steered 1"), "{tree}");
}

/// `steer` refuses an unknown step with `no-step`, a closed step with `closed`, and empty text as a usage
/// error, and prints the id of the record it wrote (`spec/trails/12-steps.md`, "The commands").
#[test]
fn steer_refuses_unknown_and_closed_steps_and_empty_text_and_prints_its_id() {
    let repo = Repo::new();
    let open = add(&repo, &["--", "open"]);
    let closed = add(&repo, &["--", "closed"]);
    assert!(run(
        &repo,
        &["step", "done", &closed[..8], "--as-planned", "--", "p"]
    )
    .status
    .success());
    refusal(
        &run(&repo, &["step", "steer", "deadbeef", "--json", "--", "x"]),
        1,
        "no-step",
    );
    refusal(
        &run(&repo, &["step", "steer", &closed[..8], "--json", "--", "x"]),
        1,
        "closed",
    );
    let o = run(&repo, &["step", "steer", &open[..8], "--", " "]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    let o = run(&repo, &["step", "steer", &open[..8], "--json", "--", "x"]);
    assert!(o.status.success(), "{}", err(&o));
    let printed: Value = serde_json::from_str(out(&o).trim()).unwrap();
    assert_eq!(
        printed["id"].as_str().map(str::len),
        Some(64),
        "{}",
        out(&o)
    );
}
