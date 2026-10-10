//! CIP-10, "Reading".

mod support;

use support::{account, decision, now, read_ids, Repo, DAY};

fn short(id: &str) -> String {
    id[..8].to_string()
}

/// A reader gives only live decisions of the working folder's project (`spec/trails/10-decisions.md`,
/// "Reading").
#[test]
fn a_reader_gives_only_live_decisions_of_the_working_folders_project() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let shop = repo.other("shop");
    let old = decision(
        &repo,
        project,
        &account(),
        (None, None),
        "old",
        now() - 4 * DAY,
    );
    let new = decision(
        &repo,
        project,
        &account(),
        (None, Some(&old.id)),
        "new",
        now() - 3 * DAY,
    );
    let foreign = decision(
        &repo,
        ("elsewhere", "shop"),
        &account(),
        (None, None),
        "foreign",
        now() - 2 * DAY,
    );
    assert_eq!(read_ids(&repo, &repo.path(), &[]), [short(&new.id)]);
    let in_shop = read_ids(&repo, &shop, &[]);
    assert!(!in_shop.contains(&short(&new.id)), "{in_shop:?}");
    assert!(!in_shop.contains(&short(&foreign.id)), "{in_shop:?}");
}

/// A decision at a file is at that file whatever line its `at` names, and a folder holds what lies
/// under it (`spec/trails/10-decisions.md`, "Reading").
#[test]
fn a_decision_at_a_file_is_at_that_file_whatever_line_and_a_folder_holds_what_lies_under_it() {
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
            &decision(
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
    assert_eq!(
        read_ids(&repo, &folder, &["--at", "src/x/a.rs"])[..3],
        [bare.clone(), line99.clone(), line3.clone()]
    );
    assert_eq!(
        read_ids(&repo, &folder, &["--at", "src/x"]),
        [other, deep, bare, line99, line3, sibling]
    );
}

/// A reader orders decisions by scope, `at` first, then `project`, then `me`, and in one scope newest
/// first, then by id (`spec/trails/10-decisions.md`, "Reading").
#[test]
fn a_reader_orders_decisions_by_scope_then_newest_first_then_by_id() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let make = |scope: Option<&str>, text: &str, at: u64| {
        short(&decision(&repo, project, &account(), (scope, None), text, at).id)
    };
    let mine = make(Some("me"), "m", now() - DAY);
    let same = now() - 3 * DAY;
    let mut tied: Vec<String> = ["a", "b", "c"]
        .iter()
        .map(|t| make(None, t, same))
        .collect();
    tied.sort();
    let newest = make(None, "n", now() - 2 * DAY);
    let placed = make(Some("at:src/a.rs"), "p", now() - 5 * DAY);
    let mut want = vec![placed, newest];
    want.extend(tied);
    want.push(mine);
    assert_eq!(read_ids(&repo, &repo.path(), &[]), want);
}

/// A place resolves against the working folder and reads as a path from the project's top folder, so
/// `./a`, a full path and a path from a subfolder name one place (`spec/trails/10-decisions.md`, "Places").
#[test]
fn a_place_reads_as_a_path_from_the_projects_top_folder() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn main() {}\n");
    let root = repo.path();
    let src = root.join("src");
    let o = repo.run(
        &root,
        &["decide", "--scope", "at:./src/a.rs:3", "--", "kept"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", support::err(&o));
    let id = short(support::out(&o).trim());
    let full = root.join("src/a.rs").display().to_string();
    assert_eq!(read_ids(&repo, &root, &["--at", &full]), [id.as_str()]);
    assert_eq!(read_ids(&repo, &src, &["--at", "a.rs"]), [id.as_str()]);
    assert_eq!(read_ids(&repo, &src, &["--at", "../src"]), [id.as_str()]);
    let o = repo.run(&root, &["read", "--json"], &[], "");
    assert!(
        support::out(&o).contains("\"at:src/a.rs:3\""),
        "{}",
        support::out(&o)
    );
}
