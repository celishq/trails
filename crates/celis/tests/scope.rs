//! CIP-10, "Scope".

mod support;

use celis_wire::Note;
use support::{account, cli, decision_tags, err, now, objects, out, put, Repo};

/// The ids of the decisions `read --json` prints in `folder`.
fn decisions(repo: &Repo, folder: &std::path::Path) -> Vec<String> {
    let o = repo.run(folder, &["read", "--json"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter_map(|v| Some(v.get("decision")?["id"].as_str()?.to_string()))
        .collect()
}

fn decide(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// A reader gives a `me` decision in every project, and a decision with any other scope only in its own
/// (`spec/trails/10-decisions.md`, "Scope").
#[test]
fn a_me_decision_holds_in_every_project_and_others_only_in_their_own() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let mine = decide(&repo, &["--scope", "me", "--", "Use plain words."]);
    let here = decide(&repo, &["--", "Keep prices in cents."]);
    let placed = decide(&repo, &["--scope", "at:src/a.rs", "--", "Round down."]);
    assert_eq!(decisions(&repo, &shop), std::slice::from_ref(&mine));
    let local = decisions(&repo, &repo.path());
    for id in [&mine, &here, &placed] {
        assert!(local.contains(id), "{local:?}");
    }
}

/// A reader counts a decision with no `scope` tag as `project`, and skips one whose scope it does not
/// know (`spec/trails/10-decisions.md`, "Scope").
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
    let shown = objects(&repo, &["read"]);
    let scopes: Vec<(&str, &str)> = shown
        .iter()
        .filter_map(|v| {
            let d = v.get("decision")?;
            Some((d["id"].as_str()?, d["scope"].as_str()?))
        })
        .collect();
    assert_eq!(scopes, [(here.as_str(), "project")]);
}
