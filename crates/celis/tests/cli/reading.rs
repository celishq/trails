//! CIP-05, "Reading".

use crate::support;

use support::{account, now, place_ids, stored_pattern, Repo, DAY};

fn short(id: &str) -> String {
    id[..8].to_string()
}

/// A reader gives only live patterns of the working folder's project at a place
/// (`spec/05-patterns.md`, "Reading").
#[test]
fn a_reader_gives_only_live_patterns_of_the_working_folders_project() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let shop = repo.other("shop");
    let old = stored_pattern(
        &repo,
        project,
        &account(),
        (Some("at:src/a.rs"), None),
        "old",
        now() - 4 * DAY,
    );
    let new = stored_pattern(
        &repo,
        project,
        &account(),
        (Some("at:src/a.rs"), Some(&old.id)),
        "new",
        now() - 3 * DAY,
    );
    let foreign = stored_pattern(
        &repo,
        ("elsewhere", "shop"),
        &account(),
        (Some("at:src/a.rs"), None),
        "foreign",
        now() - 2 * DAY,
    );
    let at = ["--at", "src/a.rs"];
    assert_eq!(place_ids(&repo, &repo.path(), &at), [short(&new.id)]);
    let in_shop = place_ids(&repo, &shop, &at);
    assert!(!in_shop.contains(&short(&new.id)), "{in_shop:?}");
    assert!(!in_shop.contains(&short(&foreign.id)), "{in_shop:?}");
}

/// A pattern at a file is at that file whatever line its `at` names, and a folder holds what lies
/// under it (`spec/05-patterns.md`, "Reading").
#[test]
fn a_pattern_at_a_file_is_at_that_file_whatever_line_and_a_folder_holds_what_lies_under_it() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    for file in [
        "src/x/a.rs",
        "src/x/ab.rs",
        "src/x/deep/b.rs",
        "src/xy/c.rs",
    ] {
        repo.write(file, "fn main() {}\n");
    }
    let at = |place: &str, n: u64| {
        short(
            &stored_pattern(
                &repo,
                project,
                &account(),
                (Some(&format!("at:{place}")), None),
                place,
                now() - n * DAY,
            )
            .id,
        )
    };
    let line3 = at("src/x/a.rs:3", 6);
    let line99 = at("src/x/a.rs:99", 5);
    let bare = at("src/x/a.rs", 4);
    let deep = at("src/x/deep/b.rs:1", 3);
    let sibling = at("src/xy/c.rs:1", 2);
    let other = at("src/x/ab.rs:1", 1);
    let folder = repo.path();
    let sorted = |mut ids: Vec<String>| {
        ids.sort();
        ids
    };
    assert_eq!(
        place_ids(&repo, &folder, &["--at", "src/x/a.rs"]),
        sorted(vec![bare.clone(), line99.clone(), line3.clone()])
    );
    assert_eq!(
        place_ids(&repo, &folder, &["--at", "src/x"]),
        sorted(vec![other, deep, bare, line99, line3])
    );
    assert_eq!(place_ids(&repo, &folder, &["--at", "src/xy"]), [sibling]);
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

/// A pattern whose scope is `at:` of the top folder is a pattern of the project with no path: it holds
/// in the whole project, and no changed file brings it (`spec/05-patterns.md`, "Reading").
#[test]
fn a_pattern_at_the_top_folder_has_no_path() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn main() {}\n");
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let mut tops = vec![];
    for (n, scope) in ["at:.", "at:./", "at:.:3"].iter().enumerate() {
        let made = now() - (n as u64 + 2) * DAY;
        let note = stored_pattern(&repo, project, &account(), (Some(scope), None), "top", made);
        tops.push(short(&note.id));
    }
    let found = place_ids(&repo, &repo.path(), &["--at", "src/a.rs"]);
    assert!(found.iter().all(|f| !tops.contains(f)), "{found:?}");
    let o = repo.run(&repo.path(), &["grep", " pattern "], &[], "");
    let shown = support::out(&o);
    for top in &tops {
        let line = shown.lines().find(|l| l.contains(top.as_str())).unwrap();
        assert!(line.ends_with("live {project} top"), "{shown}");
    }
}
