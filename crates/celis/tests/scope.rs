//! CIP-01, "Scope".

mod support;

use celis_nostr::Note;
use support::{account, cli, decision_tags, err, now, out, put, refusal, Repo};

fn decide(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// A reader gives a `me` decision in every project, and a decision with any other scope only in its own,
/// so only a `me` decision can be replaced from another project (`spec/01-decisions.md`, "Scope").
#[test]
fn a_me_decision_holds_in_every_project_and_others_only_in_their_own() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let mine = decide(&repo, &["--scope", "me", "--", "Use plain words."]);
    let here = decide(&repo, &["--", "Keep prices in cents."]);
    let placed = decide(&repo, &["--scope", "at:src/a.rs", "--", "Round down."]);
    for id in [&here, &placed] {
        let o = repo.run(
            &shop,
            &["decide", "--json", "--replaces", &id[..8], "--", "x"],
            &[],
            "",
        );
        refusal(&o, 1, "no-decision");
    }
    let o = repo.run(
        &shop,
        &["decide", "--replaces", &mine[..8], "--", "x"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
}

/// A reader counts a decision with no `scope` tag as `project`, and skips one whose scope it does not
/// know (`spec/01-decisions.md`, "Scope").
#[test]
fn no_scope_tag_is_the_project_and_an_unknown_scope_is_skipped() {
    let repo = Repo::new();
    let id = repo.project_id();
    let here = decide(&repo, &["--", "Keep prices in cents."]);
    let record = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap())
        .find(|t| t.contains(&here))
        .unwrap();
    assert!(!record.contains("\"scope\""), "{record}");
    let mut tags = decision_tags((&id, "repo"));
    tags.push(vec!["scope".into(), "team:design".into()]);
    let unknown = Note::write(&account(), 1111u16, now(), tags, "Team rule.").unwrap();
    put(&repo, &unknown);
    let text = out(&cli(&repo, &["grep", " rule "]));
    assert!(
        text.contains(&format!("rule {} live project Keep", &here[..8])),
        "{text}"
    );
    assert!(!text.contains("Team rule."), "{text}");
}
