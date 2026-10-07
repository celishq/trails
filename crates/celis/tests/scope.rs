//! CIP-10, "Scope".

mod support;

use celis_cairns::{Project, Store};
use celis_wire::Note;
use std::process::Command;
use support::Repo;

/// Every clone and every worktree of one repo has the id of its first commit, and the name of the main
/// checkout's folder (`spec/cairns/10-cairns.md`, "Scope").
#[test]
fn every_clone_and_every_worktree_of_one_repo_must_give_the_same_project_id() {
    let repo = Repo::new();
    let first = repo.git(&["rev-list", "--max-parents=0", "HEAD"]);
    let tree = repo.worktree("feat-a");
    let clone = tempfile::tempdir().unwrap();
    let copy = clone.path().join("copy");
    let cloned = Command::new("git")
        .args([
            "clone",
            "-q",
            repo.path().to_str().unwrap(),
            copy.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(cloned.success());
    for folder in [repo.path(), repo.path().join("sub"), tree, copy] {
        std::fs::create_dir_all(&folder).unwrap();
        assert_eq!(Project::at(&folder).id, first, "{}", folder.display());
    }
    assert_eq!(Project::at(&repo.worktree("feat-b")).name, "repo");
}

/// Outside git, the working folder's path is the project id, and its name is the project name
/// (`spec/cairns/10-cairns.md`, "Scope").
#[test]
fn outside_git_the_project_id_is_the_path_of_the_working_folder() {
    let outside = tempfile::tempdir().unwrap();
    let folder = outside.path().join("shop");
    std::fs::create_dir_all(&folder).unwrap();
    let project = Project::at(&folder);
    assert_eq!(project.id, folder.display().to_string());
    assert_eq!(project.name, "shop");
}

/// `--project` names the project, and the id stays the first commit (`spec/cairns/10-cairns.md`,
/// "Scope").
#[test]
fn the_name_must_not_change_the_project_id() {
    let repo = Repo::new();
    let args = [
        "add",
        "--task",
        "t",
        "--project",
        "shelf",
        "--",
        "Before you run make, set CI=1.",
    ];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let note = written(&repo);
    assert_eq!(values(&note, "project"), ["shelf"]);
    assert_eq!(
        values(&note, "I"),
        [format!("project:{}", repo.project_id())]
    );
}

/// `add --project <name>` carries the name as the cairn's project name.
///
/// (`spec/cairns/11-cli.md`, "Commands")
#[test]
fn add_with_project_carries_the_name_as_the_cairns_project_name() {
    let repo = Repo::new();
    let args = [
        "add",
        "--task",
        "t",
        "--project",
        "shelf",
        "--",
        "Before you run make, set CI=1.",
    ];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(values(&written(&repo), "project"), ["shelf"]);
}

/// The one cairn in the store.
fn written(repo: &Repo) -> Note {
    let records = Store::open(&repo.home()).unwrap().records();
    let mut cairns = records.into_iter().filter(|n| n.kind == 1111);
    cairns.next().unwrap()
}

/// The values of the tags of `note` named `name`.
fn values<'a>(note: &'a Note, name: &str) -> Vec<&'a str> {
    note.tags
        .iter()
        .filter(|t| t[0] == name)
        .map(|t| t[1].as_str())
        .collect()
}
