//! CIP-01, "Links".

mod support;

use celis_nostr::{Account, Note};
use std::path::Path;
use support::{err, now, out, step_tags, Repo};

/// The records in the `.json` files of `folder`.
fn records(folder: &Path) -> Vec<Note> {
    std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok())
        .collect()
}

fn ids(folder: &Path) -> Vec<String> {
    let mut ids: Vec<String> = records(folder).into_iter().map(|n| n.id).collect();
    ids.sort();
    ids
}

/// Runs `celis args` in the main checkout and returns its stdout; it must succeed.
fn run(repo: &Repo, args: &[&str]) -> String {
    let o = repo.run(&repo.path(), args, &[], "");
    assert!(o.status.success(), "{args:?}: {}", err(&o));
    out(&o)
}

fn linked(repo: &Repo) -> std::path::PathBuf {
    repo.path().join(".celis/trails")
}

/// A step of the repo, signed by a stranger, written into the link folder alone.
fn strangers_step(repo: &Repo) -> Note {
    let stranger = Account::new(&[2u8; 32]).unwrap();
    let tags = step_tags((&repo.project_id(), "repo"));
    let note = Note::write(&stranger, 1111u16, now(), tags, "Ben's step").unwrap();
    std::fs::create_dir_all(linked(repo)).unwrap();
    let name = format!("{}-{}.json", note.created_at, note.id);
    std::fs::write(
        linked(repo).join(name),
        serde_json::to_string(&note).unwrap(),
    )
    .unwrap();
    note
}

/// `link` creates `.celis/trails/` and `.celis/trails/README.md`, and copies the project's records out of the
/// store.
#[test]
fn link_creates_the_folder_and_copies_the_projects_records_out() {
    let repo = Repo::new();
    run(&repo, &["step", "add", "--", "Keep prices in cents."]);
    let shown = run(&repo, &["link"]);
    let copied = ids(&linked(&repo));
    assert!(!copied.is_empty());
    assert_eq!(copied, ids(&repo.store().join("shared")));
    assert_eq!(
        shown.trim(),
        format!(".celis/trails · {} out · 0 in", copied.len())
    );
    let readme = std::fs::read_to_string(repo.path().join(".celis/trails/README.md")).unwrap();
    assert!(readme.contains("celis"), "{readme}");
    let listed = std::fs::read_to_string(repo.home().join("links")).unwrap();
    assert_eq!(listed, format!("{}\t.celis/trails\n", repo.project_id()));
}

/// A second `link` copies nothing, and `--json` prints the folder and both counts.
#[test]
fn a_second_link_copies_nothing() {
    let repo = Repo::new();
    run(&repo, &["step", "add", "--", "Keep prices in cents."]);
    run(&repo, &["link"]);
    let shown = run(&repo, &["link", "--json"]);
    let json: serde_json::Value = serde_json::from_str(shown.trim()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"folder": ".celis/trails", "out": 0, "in": 0})
    );
    let listed = std::fs::read_to_string(repo.home().join("links")).unwrap();
    assert_eq!(listed.lines().count(), 1, "{listed}");
}

/// A link holds no `me` decision and no record of another project, and holds a status record by the
/// step it names (`spec/01-records.md`, "Links").
#[test]
fn a_link_holds_only_the_projects_records_and_no_me_decision() {
    let repo = Repo::new();
    let mine = run(
        &repo,
        &[
            "decide",
            "--scope",
            "me",
            "--",
            "Write plain commit messages.",
        ],
    );
    let kept = run(&repo, &["decide", "--", "Keep prices in cents."]);
    let step = run(&repo, &["step", "add", "--", "Round the price down."]);
    let step = step.lines().next().unwrap().to_string();
    let started = run(&repo, &["step", "start", &step]);
    let other = repo.other("crm");
    let elsewhere = repo.run(&other, &["decide", "--", "Use the shared table."], &[], "");
    run(&repo, &["link"]);
    let held = ids(&linked(&repo));
    assert!(!held.contains(&mine.trim().to_string()), "{held:?}");
    assert!(
        !held.contains(&out(&elsewhere).trim().to_string()),
        "{held:?}"
    );
    assert!(held.contains(&kept.trim().to_string()), "{held:?}");
    assert!(held.contains(&step), "{held:?}");
    let start = records(&linked(&repo))
        .into_iter()
        .any(|n| n.kind == 1985 && n.tag("e") == Some(step.as_str()));
    assert!(start, "{started}");
}

/// When the link is on, each record the link holds is written to it as well
/// (`spec/01-records.md`, "Links").
#[test]
fn when_the_link_is_on_a_new_record_is_written_to_it() {
    let repo = Repo::new();
    run(&repo, &["link"]);
    let kept = run(&repo, &["decide", "--", "Keep prices in cents."]);
    let mine = run(
        &repo,
        &[
            "decide",
            "--scope",
            "me",
            "--",
            "Write plain commit messages.",
        ],
    );
    let held = ids(&linked(&repo));
    assert_eq!(held, vec![kept.trim().to_string()], "{mine}");
}

/// A link folder that this machine did not turn on is not read, and nothing is copied from it
/// (`spec/01-records.md`, "Links").
#[test]
fn a_link_that_is_not_on_is_not_read() {
    let repo = Repo::new();
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    let note = strangers_step(&repo);
    let shown = run(&repo, &["tree", "--all"]);
    assert!(!shown.contains("Ben's step"), "{shown}");
    assert!(!ids(&repo.store().join("shared")).contains(&note.id));
    assert_eq!(ids(&linked(&repo)), vec![note.id]);
}

/// When the link is on, a reader reads it, and copies into the store each record the store does not
/// have (`spec/01-records.md`, "Links").
#[test]
fn when_the_link_is_on_its_records_are_read_and_copied_in() {
    let repo = Repo::new();
    run(&repo, &["link"]);
    let note = strangers_step(&repo);
    let shown = run(&repo, &["tree", "--all"]);
    assert!(shown.contains("Ben's step"), "{shown}");
    assert!(ids(&repo.store().join("shared")).contains(&note.id));
    let other = Repo::new();
    std::fs::create_dir_all(linked(&other)).unwrap();
    let theirs = strangers_step(&other);
    let shown = run(&other, &["link"]);
    assert_eq!(shown.trim(), ".celis/trails · 0 out · 1 in");
    assert!(ids(&other.store().join("shared")).contains(&theirs.id));
}

/// A worktree with no `.celis/trails/` writes no link, though its project is linked
/// (`spec/01-records.md`, "Links").
#[test]
fn a_checkout_without_the_folder_writes_no_link() {
    let repo = Repo::new();
    let tree = repo.worktree("feat-a");
    run(&repo, &["link"]);
    let o = repo.run(&tree, &["decide", "--", "Keep prices in cents."], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(!tree.join(".celis").exists());
}

/// When the top folder has `.celis/trails/` and the link is not on, `prompt` asks, once a session, to
/// ask the person whether to link, and names `celis link`; never on the prompt that asks for the past
/// agent.
#[test]
fn prompt_asks_to_link_a_project_whose_folder_is_not_linked() {
    let repo = Repo::new();
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    strangers_step(&repo);
    let hook = |session: &str| {
        let json =
            serde_json::json!({"session_id": session, "prompt": "Fix the cart."}).to_string();
        repo.run(&repo.path(), &["prompt"], &[], &json)
    };
    let first = err(&hook("s1"));
    assert!(first.contains("celis:past"), "{first}");
    assert!(!first.contains("celis link"), "{first}");
    run(
        &repo,
        &["step", "add", "--session", "s1", "--", "Fix the cart."],
    );
    let second = err(&hook("s1"));
    assert!(second.contains("celis link"), "{second}");
    assert!(second.contains("link"), "{second}");
    let third = err(&hook("s1"));
    assert!(!third.contains("celis link"), "{third}");
    run(&repo, &["link"]);
    hook("s2");
    let on = err(&hook("s2"));
    assert!(!on.contains("celis link"), "{on}");
}

/// Without `.celis/trails/`, `prompt` does not ask to link.
#[test]
fn prompt_does_not_ask_to_link_a_project_with_no_folder() {
    let repo = Repo::new();
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    let json = serde_json::json!({"session_id": "s1", "prompt": "Fix the cart."}).to_string();
    let said = err(&repo.run(&repo.path(), &["prompt"], &[], &json));
    assert!(!said.contains("celis link"), "{said}");
}

/// `unlink` takes the link out of `$CELIS_HOME/links` and keeps the folder; then nothing is
/// written to the folder or read from it.
#[test]
fn unlink_stops_the_sync_and_keeps_the_folder() {
    let repo = Repo::new();
    run(&repo, &["link"]);
    let shown = run(&repo, &["unlink"]);
    assert_eq!(shown.trim(), ".celis/trails · unlinked");
    let json = run(&repo, &["unlink", "--json"]);
    let json: serde_json::Value = serde_json::from_str(json.trim()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"folder": ".celis/trails", "unlinked": true})
    );
    let listed = std::fs::read_to_string(repo.home().join("links")).unwrap();
    assert!(!listed.contains(&repo.project_id()), "{listed}");
    assert!(repo.path().join(".celis/trails/README.md").exists());
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    assert!(ids(&linked(&repo)).is_empty());
    let note = strangers_step(&repo);
    let tree = run(&repo, &["tree", "--all"]);
    assert!(!tree.contains("Ben's step"), "{tree}");
    assert!(!ids(&repo.store().join("shared")).contains(&note.id));
}

/// With no store, `prompt` still asks to link a project whose link is not on, and creates no store.
#[test]
fn prompt_with_no_store_asks_to_link() {
    let repo = Repo::new();
    std::fs::create_dir_all(linked(&repo)).unwrap();
    let json = serde_json::json!({"session_id": "s1", "prompt": "Fix the cart."}).to_string();
    let said = err(&repo.run(&repo.path(), &["prompt"], &[], &json));
    assert!(said.contains("celis link"), "{said}");
    assert!(!repo.store().exists());
}

/// The README of `.celis/` says how to install celis.
#[test]
fn the_readme_says_how_to_install_celis() {
    let repo = Repo::new();
    run(&repo, &["link"]);
    let readme = std::fs::read_to_string(repo.path().join(".celis/trails/README.md")).unwrap();
    assert!(readme.contains("https://celis.ai/install.sh"), "{readme}");
}

/// `link <folder>` links a folder outside the repo by its full path; records go to it and come from it,
/// and `unlink <folder>` takes it out (`spec/01-records.md`, "Links").
#[test]
fn a_folder_outside_the_repo_links_by_its_full_path() {
    let repo = Repo::new();
    let drive = tempfile::tempdir().unwrap();
    let folder = std::fs::canonicalize(drive.path())
        .unwrap()
        .join("celis/shop");
    let place = folder.display().to_string();
    let kept = run(&repo, &["decide", "--", "Keep prices in cents."]);
    let shown = run(&repo, &["link", &place]);
    assert_eq!(shown.trim(), format!("{place} · 1 out · 0 in"));
    assert_eq!(ids(&folder), vec![kept.trim().to_string()]);
    assert!(folder.join("README.md").exists());
    assert!(!repo.path().join(".celis").exists());
    let listed = std::fs::read_to_string(repo.home().join("links")).unwrap();
    assert_eq!(listed, format!("{}\t{place}\n", repo.project_id()));
    let tree = repo.worktree("feat-a");
    let o = repo.run(&tree, &["decide", "--", "Round the price down."], &[], "");
    assert_eq!(ids(&folder).len(), 2, "{}", err(&o));
    run(&repo, &["unlink", &place]);
    assert!(std::fs::read_to_string(repo.home().join("links"))
        .unwrap()
        .is_empty());
}
