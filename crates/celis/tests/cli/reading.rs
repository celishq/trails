//! CIP-05, "Paths", and CIP-01, "Places".

use crate::support;

use support::{account, now, place_ids, tagged_pattern, Repo, DAY};

fn short(id: &str) -> String {
    id[..8].to_string()
}

/// The `path` tags of a pattern at `src/a.rs`.
fn paths() -> Vec<Vec<String>> {
    vec![vec!["path".to_string(), "src/a.rs".to_string()]]
}

/// A reader gives only live patterns of the working folder's project at a place
/// (`spec/05-patterns.md`, "Paths").
#[test]
fn a_reader_gives_only_live_patterns_of_the_working_folders_project() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let shop = repo.other("shop");
    let old = tagged_pattern(
        &repo,
        project,
        &account(),
        (None, None),
        paths(),
        "old",
        now() - 4 * DAY,
    );
    let new = tagged_pattern(
        &repo,
        project,
        &account(),
        (None, Some(&old.id)),
        paths(),
        "new",
        now() - 3 * DAY,
    );
    let foreign = tagged_pattern(
        &repo,
        ("elsewhere", "shop"),
        &account(),
        (None, None),
        paths(),
        "foreign",
        now() - 2 * DAY,
    );
    let at = ["--at", "src/a.rs"];
    assert_eq!(place_ids(&repo, &repo.path(), &at), [short(&new.id)]);
    let in_shop = place_ids(&repo, &shop, &at);
    assert!(!in_shop.contains(&short(&new.id)), "{in_shop:?}");
    assert!(!in_shop.contains(&short(&foreign.id)), "{in_shop:?}");
}

/// A place resolves against the working folder and reads as a path from the project's top folder, so
/// `./a`, a full path and a path from a subfolder name one place (`spec/01-records.md`, "Places").
#[test]
fn a_place_reads_as_a_path_from_the_projects_top_folder() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn main() {}\n");
    let root = repo.path();
    let src = root.join("src");
    let o = repo.run(
        &root,
        &["pattern", "--path", "./src/a.rs", "--", "kept"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", support::err(&o));
    let id = short(support::out(&o).trim());
    let full = root.join("src/a.rs").display().to_string();
    assert_eq!(place_ids(&repo, &root, &["--at", &full]), [id.as_str()]);
    assert_eq!(place_ids(&repo, &src, &["--at", "a.rs"]), [id.as_str()]);
    assert_eq!(place_ids(&repo, &src, &["--at", "../src"]), [id.as_str()]);
    let pattern = support::labelled(&repo, "pattern").pop().unwrap();
    assert_eq!(pattern.tag("path"), Some("src/a.rs"));
}
