//! CIP-02, "Past".

mod support;

use support::{account, cli, err, now, out, stored_rule, tiny_model, Repo, DAY};

/// `past` prints the live rules in scope 50 a page, and the first page names the others
/// (`spec/02-cli.md`, "Past").
#[test]
fn past_prints_the_rules_fifty_a_page() {
    let repo = Repo::plain();
    let id = repo.project_id();
    for n in 0..120 {
        let text = format!("When rule {n} holds: keep it.");
        stored_rule(
            &repo,
            (id.as_str(), "repo"),
            &account(),
            (None, None),
            &text,
            now() - DAY,
        );
    }
    let page = |args: &[&str]| {
        let o = repo.run(&repo.path(), args, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        out(&o)
    };
    let first = page(&["past", "--", "a new report"]);
    assert!(first.starts_with("rules, page 1 of 3\n"), "{first}");
    assert_eq!(first.matches("When rule").count(), 50);
    assert!(
        first.contains("\npast\n  celis search has no model"),
        "{first}"
    );
    let last = first.lines().last().unwrap();
    assert!(
        last.contains("read pages 2 to 3 with celis past --page <n>"),
        "{last}"
    );
    assert_eq!(
        page(&["past", "--page", "2"]).matches("When rule").count(),
        50
    );
    let third = page(&["past", "--page", "3"]);
    assert_eq!(third.matches("When rule").count(), 20);
    assert!(!third.contains("past\n"), "{third}");
    for args in [&["past", "--page", "4"][..], &["past", "--", " "]] {
        let o = repo.run(&repo.path(), args, &[], "");
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", out(&o));
    }
}

/// The first page shows the lines of the trails most alike to the ask (`spec/02-cli.md`, "Past").
#[test]
fn past_shows_the_trails_most_alike() {
    let repo = Repo::new();
    tiny_model(&repo);
    let o = cli(&repo, &["step", "add", "--new", "--", "release linux"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["past", "--", "release linux"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let past = text.split("\npast\n").nth(1).unwrap_or_default();
    assert!(past.contains(" trail "), "{text}");
    assert!(past.contains("release linux"), "{text}");
}
