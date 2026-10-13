//! CIP-03, "Steering".

mod support;

use celis_nostr::Note;
use std::process::Output;
use support::{account, err, now, out, put, Repo};

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
/// (`spec/trails/03-steps.md`, "Steering").
#[test]
fn a_steer_does_not_close_its_step_and_one_with_no_content_is_ignored() {
    let repo = Repo::new();
    let id = add(&repo, &["--", "badge"]);
    support::steer(&repo, &id, "Use the shared Badge.");
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "steer".into(), "celis.step".into()],
        vec!["e".into(), id.clone()],
    ];
    put(
        &repo,
        &Note::write(&account(), 1985u16, now(), tags, "").unwrap(),
    );
    let read = out(&run(&repo, &["past"]));
    assert!(read.contains(&id[..8]), "{read}");
    let o = run(
        &repo,
        &["step", "done", &id[..8], "--as-planned", "--", "done"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let tree = out(&run(&repo, &["tree", &id[..8], "--full"]));
    assert!(tree.contains(" · steered 1"), "{tree}");
}
