//! CIP-03, "Steps".

mod support;

use celis_nostr::Note;
use support::Repo;

fn run(repo: &Repo, args: &[&str]) -> std::process::Output {
    repo.run(&repo.path(), args, &[], "")
}

fn stored(repo: &Repo) -> Vec<Note> {
    let dir = repo.store().join("shared");
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .collect()
}

fn tag_values<'a>(note: &'a Note, name: &str) -> Vec<&'a str> {
    note.tags
        .iter()
        .filter(|t| t[0] == name)
        .map(|t| t[1].as_str())
        .collect()
}

/// `step add` writes a kind 1111 record with content, the anchor `project:<id>`, the project name, and
/// the label `step` in `celis.step`; with `--at` and `--under` it carries one `at` and one `under`
/// (`spec/trails/03-steps.md`, "Steps").
#[test]
fn step_add_writes_a_kind_1111_record_with_its_tags() {
    let repo = Repo::plain();
    let first = run(&repo, &["step", "add", "--", "first work"]);
    assert!(first.status.success(), "{}", support::err(&first));
    let needed = support::out(&first).trim().to_string();
    let second = run(
        &repo,
        &[
            "step",
            "add",
            "--at",
            "a/b.rs:3",
            "--under",
            &needed,
            "--",
            "second work",
        ],
    );
    assert!(second.status.success(), "{}", support::err(&second));
    let id = support::out(&second).trim().to_string();

    let notes = stored(&repo);
    let plain = notes.iter().find(|n| n.id == needed).expect("first stored");
    let full = notes.iter().find(|n| n.id == id).expect("second stored");
    assert_eq!(plain.kind, 1111);
    assert_eq!(plain.content, "first work");
    assert!(tag_values(plain, "task").is_empty());
    assert_eq!(
        tag_values(plain, "I"),
        [format!("project:{}", repo.project_id())]
    );
    assert_eq!(tag_values(plain, "project"), ["repo"]);
    assert_eq!(tag_values(plain, "L"), ["celis.step"]);
    assert!(plain.tags.contains(&vec![
        "l".to_string(),
        "step".to_string(),
        "celis.step".to_string()
    ]));
    assert!(tag_values(plain, "at").is_empty());
    assert_eq!(tag_values(full, "at"), ["a/b.rs:3"]);
    assert_eq!(tag_values(full, "under"), [needed.as_str()]);
}

/// A reader counts a kind 1111 record as a step only with the label `step` in `celis.step`, and ignores
/// unknown tags (`spec/trails/03-steps.md`, "Steps").
#[test]
fn a_reader_counts_a_record_as_a_step_only_with_the_label() {
    let repo = Repo::plain();
    let by = &support::account();
    let anchor = format!("project:{}", repo.project_id());
    let build = |label: &str, content: &str| {
        let tags: Vec<Vec<String>> = [
            vec!["I", anchor.as_str()],
            vec!["K", "project"],
            vec!["i", anchor.as_str()],
            vec!["k", "project"],
            vec!["project", "repo"],
            vec!["unknown", "x"],
            vec!["L", "celis.step"],
            vec!["l", label, "celis.step"],
        ]
        .into_iter()
        .map(|t| t.into_iter().map(String::from).collect())
        .collect();
        let note = Note::write(by, 1111u16, support::now() - 60, tags, content).unwrap();
        support::put(&repo, &note);
        note
    };
    let step = build("step", "a real step");
    let other = build("other", "not a step");

    let all = run(&repo, &["past"]);
    let text = support::out(&all);
    assert!(text.contains(&step.id[..8]), "{text}");
    assert!(!text.contains(&other.id[..8]), "{text}");
}

/// A step's text line holds only its title, its first line; the full tree prints the other lines under
/// it (`spec/trails/03-steps.md`, "Steps").
#[test]
fn a_steps_line_holds_its_title_and_the_full_tree_its_detail() {
    let repo = Repo::new();
    let trail = support::add(&repo, &["--", "the trail"]);
    support::add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--",
            "Round down\nThe test expects 7.99.",
        ],
    );
    let short = support::out(&run(&repo, &["tree", &trail[..8]]));
    assert!(short.lines().any(|l| l.ends_with(" Round down")), "{short}");
    assert!(!short.contains("7.99"), "{short}");
    let full = support::out(&run(&repo, &["tree", &trail[..8], "--full"]));
    let lines: Vec<&str> = full.lines().collect();
    assert!(lines[1].ends_with(" Round down"), "{full}");
    assert_eq!(lines[2], "                The test expects 7.99.", "{full}");
}
