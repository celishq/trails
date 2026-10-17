//! CIP-01, "Projects".

mod support;

use celis_nostr::Note;
use support::Repo;

fn add_at(repo: &Repo, folder: &std::path::Path, url: &str) -> Note {
    let out = repo.run(
        folder,
        &["step", "add", "--at", url, "--", &format!("work at {url}")],
        &[],
        "",
    );
    assert!(out.status.success(), "{}", support::err(&out));
    let id = support::out(&out).trim().to_string();
    let dir = repo.store().join("shared");
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .find(|n| n.id == id)
        .expect("the step is stored")
}

fn anchor(note: &Note) -> String {
    note.tag("I").unwrap().to_string()
}

/// Outside git, `step add --at <URL>` names the project `figma:<file key>` for a figma.com /file/ or
/// /design/ URL and `web:<host>` for any other URL; two steps at one Figma file from two folders name the
/// same project (`spec/01-records.md`, "Projects").
#[test]
fn outside_git_a_url_names_the_project() {
    let repo = Repo::plain();
    let one = tempfile::tempdir().unwrap();
    let two = tempfile::tempdir().unwrap();

    let file = add_at(
        &repo,
        one.path(),
        "https://www.figma.com/file/Ab12/Onboarding",
    );
    let design = add_at(
        &repo,
        two.path(),
        "https://figma.com/design/Ab12/Other-name",
    );
    let web = add_at(&repo, one.path(), "https://example.com/some/page");

    assert_eq!(anchor(&file), "project:figma:Ab12");
    assert_eq!(anchor(&design), anchor(&file));
    assert_eq!(anchor(&web), "project:web:example.com");
}

/// Outside git, `step add --at <URL>` names the project too: the file name segment of a Figma URL, else the
/// host (`spec/01-records.md`, "Projects").
#[test]
fn outside_git_a_url_gives_the_project_its_name() {
    let repo = Repo::plain();
    let folder = tempfile::tempdir().unwrap();

    let figma = add_at(
        &repo,
        folder.path(),
        "https://www.figma.com/file/Ab12/Onboarding",
    );
    let web = add_at(&repo, folder.path(), "https://example.com/some/page");

    assert_eq!(figma.tag("project"), Some("Onboarding"));
    assert_eq!(web.tag("project"), Some("example.com"));
}

/// Outside git, two steps at one Figma file whose URLs hold two file names carry one project id and each
/// its own project name (`spec/01-records.md`, "Projects").
#[test]
fn one_figma_file_under_two_names_is_one_project_with_two_names() {
    let repo = Repo::plain();
    let folder = tempfile::tempdir().unwrap();

    let first = add_at(
        &repo,
        folder.path(),
        "https://www.figma.com/file/Ab12/Onboarding",
    );
    let second = add_at(
        &repo,
        folder.path(),
        "https://www.figma.com/design/Ab12/Checkout",
    );

    assert_eq!(anchor(&first), anchor(&second));
    assert_eq!(first.tag("project"), Some("Onboarding"));
    assert_eq!(second.tag("project"), Some("Checkout"));
}
