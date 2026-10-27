//! CIP-01, "Projects".

use crate::support;

use celis_nostr::Note;
use celis_store::{Home, Project};
use std::process::Command;
use support::Repo;

/// Every clone and every worktree of one repo has the id of its first commit, and the name of the main
/// checkout's folder (`spec/01-records.md`, "Projects").
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
        assert_eq!(
            Project::at(&folder).anchor.project,
            first,
            "{}",
            folder.display()
        );
    }
    assert_eq!(Project::at(&repo.worktree("feat-b")).anchor.name, "repo");
}

/// Outside git, the working folder's path is the project id, and its name is the project name
/// (`spec/01-records.md`, "Projects").
#[test]
fn outside_git_the_project_id_is_the_path_of_the_working_folder() {
    let outside = tempfile::tempdir().unwrap();
    let folder = outside.path().join("shop");
    std::fs::create_dir_all(&folder).unwrap();
    let project = Project::at(&folder);
    assert_eq!(project.anchor.project, folder.display().to_string());
    assert_eq!(project.anchor.name, "shop");
}

/// A clone in a folder of another name has that name, and the id stays the first commit
/// (`spec/01-records.md`, "Projects").
#[test]
fn the_name_must_not_change_the_project_id() {
    let repo = Repo::new();
    let clone = tempfile::tempdir().unwrap();
    let copy = clone.path().join("shelf");
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
    let args = ["step", "add", "--", "Run make."];
    let o = repo.run(&copy, &args, &[], "");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let note = written(&repo);
    assert_eq!(values(&note, "project"), ["shelf"]);
    assert_eq!(
        values(&note, "I"),
        [format!("project:{}", repo.project_id())]
    );
}

/// The one trail in the store.
fn written(repo: &Repo) -> Note {
    let records = Home::open(&repo.home()).unwrap().records();
    records.into_iter().find(|n| n.kind == 1111).unwrap()
}

/// The values of the tags of `note` named `name`.
fn values<'a>(note: &'a Note, name: &str) -> Vec<&'a str> {
    note.tags
        .iter()
        .filter(|t| t[0] == name)
        .map(|t| t[1].as_str())
        .collect()
}
