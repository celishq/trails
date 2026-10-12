//! CIP-14, "The past".

mod support;

use celis_trails::{Anchor, Decision, Mark, Scope, Step};
use celis_wire::Note;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use support::{account, add, cli, done_note, err, now, objects, out, put, step_note, Repo};

/// A done trail saying `text` in `folder`, with a done step under it, steered when `steered`.
fn trail(repo: &Repo, folder: &Path, text: &str, steered: bool) -> String {
    let run = |args: &[&str]| {
        let o = repo.run(folder, args, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        out(&o).trim().to_string()
    };
    let trail = run(&["step", "add", "--", text]);
    let part = run(&["step", "add", "--under", &trail[..8], "--", "the part"]);
    if steered {
        run(&["step", "steer", &part[..8], "--", "Use the shared table."]);
    }
    run(&["step", "done", &part[..8], "--as-planned", "--", "p"]);
    run(&["step", "done", &trail[..8], "--as-planned", "--", "p"]);
    std::thread::sleep(Duration::from_millis(1100));
    trail
}

/// `read --like` prints the done trails of every project like the text, 3 at most:
/// the highest score first, then the fewest steers and rework, then the newest
/// (`spec/trails/14-trails.md`, "The past").
#[test]
fn read_like_prints_the_done_trails_that_share_most_words_and_least_help() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let here = repo.path();
    let vendors = trail(&repo, &here, "Build the Vendors page", false);
    trail(&repo, &here, "Build the Income page", true);
    let admin = trail(&repo, &here, "Build the Admin page", true);
    let settings = trail(&repo, &shop, "Build the Settings page in the shop", false);
    trail(&repo, &here, "Fix the login", false);
    add(&repo, &["--", "Build the Reports page"]);
    let past: Vec<String> = objects(&repo, &["read", "--like", "Build the Settings page"])
        .into_iter()
        .filter_map(|v| v.get("past").map(|p| p["id"].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(past, [settings, vendors.clone(), admin]);
    let o = cli(&repo, &["read", "--like", "Build the Vendors page"]);
    assert!(out(&o).contains(&format!(
        "{} [repo] Build the Vendors page · took ",
        &vendors[..8]
    )));
}

/// `read --like` weighs each word by how few asks and trails hold it, so one rare shared word outranks
/// two common ones. It finds a done ask under a trail that ended with no `done`, and prints it on one line
/// (`spec/trails/14-trails.md`, "The past").
#[test]
fn read_like_ranks_rare_words_first_and_finds_done_asks() {
    let repo = Repo::new();
    let by = account();
    let t = now() - 3 * 24 * 60 * 60;
    let done = |under: Option<&Note>, text: &str, at: u64| {
        let note = step_note(&repo, &by, under, text, at);
        done_note(&repo, &by, &note, at + 1);
        note
    };
    for (n, text) in [
        "celis tree columns",
        "celis tree colours",
        "celis tree layout",
    ]
    .into_iter()
    .enumerate()
    {
        done(None, text, t + 10 * n as u64);
    }
    let ended = step_note(&repo, &by, None, "Polish the terminal", t + 40);
    let watch = done(Some(&ended), "watch redraws on resize", t + 41);
    let text = out(&cli(&repo, &["read", "--like", "celis tree watch"]));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with(&watch.id[..8]), "{text}");
    assert_eq!(lines.len(), 3, "{text}");
}

/// Each prompt of `spec/fixtures/past.json` finds its trail first, or no past work when it names none
/// (`spec/trails/14-trails.md`, "The past").
#[test]
fn read_like_finds_each_fixture_prompts_trail() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../spec/fixtures/past.json")).unwrap();
    let repo = Repo::new();
    let by = account();
    let anchor = Anchor {
        project: repo.project_id(),
        name: "repo".to_string(),
    };
    let text = |v: &Value, key: &str| v[key].as_str().map(str::to_string);
    let mut names = Vec::new();
    let at = now() - 3 * 24 * 60 * 60;
    for (n, trail) in fixture["trails"].as_array().unwrap().iter().enumerate() {
        let at = at + 10 * n as u64;
        let step = Step {
            anchor: anchor.clone(),
            under: None,
            fixes: None,
            replaces: None,
            places: trail["places"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p.as_str().unwrap().to_string())
                .collect(),
            check: None,
            session: None,
            content: text(trail, "title").unwrap(),
        };
        let note = step.record(&by, at).unwrap();
        put(&repo, &note);
        let done = Mark::Done {
            proof: text(trail, "proof").unwrap(),
            changed: text(trail, "changed"),
        };
        put(&repo, &done.record(&by, &note.id, at + 1).unwrap());
        if let Some(pitfall) = text(trail, "pitfall") {
            let rule = Decision {
                anchor: anchor.clone(),
                scope: Scope::Trail(note.id.clone()),
                replaces: None,
                content: pitfall,
            };
            put(&repo, &rule.record(&by, at + 2).unwrap());
        }
        names.push((text(trail, "name").unwrap(), note.id));
    }
    for prompt in fixture["prompts"].as_array().unwrap() {
        let like = text(prompt, "text").unwrap();
        let first = objects(&repo, &["read", "--like", &like])
            .into_iter()
            .find_map(|v| v.get("past").map(|p| p["id"].as_str().unwrap().to_string()));
        let found =
            first.and_then(|id| names.iter().find(|(_, n)| *n == id).map(|(k, _)| k.clone()));
        assert_eq!(found, text(prompt, "finds"), "{}: {like}", prompt["kind"]);
    }
}

/// `read --like` puts the working project's past work first, and then the work at its `--at` places, when
/// the text alone ties them (`spec/trails/14-trails.md`, "The past").
#[test]
fn read_like_prefers_the_working_projects_and_places_work() {
    let repo = Repo::new();
    let by = account();
    let shop_folder = repo.other("shop");
    let shop = Anchor {
        project: celis_trails::Project::at(&shop_folder).id,
        name: "shop".to_string(),
    };
    let here = Anchor {
        project: repo.project_id(),
        name: "repo".to_string(),
    };
    let at = now() - 24 * 60 * 60;
    let trail = |anchor: &Anchor, title: &str, place: &str, at: u64| {
        let step = Step {
            anchor: anchor.clone(),
            under: None,
            fixes: None,
            replaces: None,
            places: vec![place.to_string()],
            check: None,
            session: None,
            content: title.to_string(),
        };
        let note = step.record(&by, at).unwrap();
        put(&repo, &note);
        let done = Mark::Done {
            proof: "done".to_string(),
            changed: None,
        };
        put(&repo, &done.record(&by, &note.id, at + 1).unwrap());
        note.id
    };
    let in_shop = trail(&shop, "Add the export button", "Export.swift", at);
    let in_repo = trail(&here, "Add the export button", "Export.swift", at + 10);
    let left = trail(&here, "Fix the table layout", "left/Table.swift", at + 20);
    let right = trail(&here, "Fix the table layout", "right/Grid.swift", at + 30);
    for n in 0..4 {
        trail(
            &here,
            &format!("Polish the menu {n}"),
            "Menu.swift",
            at + 40 + n,
        );
    }
    let first = |folder: &Path, args: &[&str]| {
        let mut all = vec!["read", "--json"];
        all.extend(args);
        let o = repo.run(folder, &all, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        out(&o)
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .find_map(|v| v.get("past").map(|p| p["id"].as_str().unwrap().to_string()))
    };
    let like = ["--like", "Add the export button"];
    assert_eq!(first(&shop_folder, &like), Some(in_shop));
    assert_eq!(first(&repo.path(), &like), Some(in_repo));
    let layout = ["--like", "Fix the table layout"];
    let at_left = [&layout[..], &["--at", "left/Table.swift"]].concat();
    let at_right = [&layout[..], &["--at", "right/Grid.swift"]].concat();
    assert_eq!(first(&repo.path(), &at_left), Some(left));
    assert_eq!(first(&repo.path(), &at_right), Some(right));
}

/// `read --like` appends what it showed, and `tree <id>` the step it opened, to
/// `$CELIS_HOME/retrieval.log`, and neither writes a record (`spec/trails/14-trails.md`, "The past").
#[test]
fn read_like_and_tree_append_to_the_retrieval_log() {
    let repo = Repo::new();
    let vendors = trail(&repo, &repo.path(), "Build the Vendors page", false);
    trail(&repo, &repo.path(), "Fix the login", false);
    let records = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .count();
    let o = cli(&repo, &["read", "--like", "Build the Vendors page"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["tree", &vendors[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let log = std::fs::read_to_string(repo.home().join("retrieval.log")).unwrap();
    let lines: Vec<Value> = log
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 2, "{log}");
    assert_eq!(lines[0]["shown"]["from"], "read");
    assert_eq!(lines[0]["shown"]["text"], "Build the Vendors page");
    assert_eq!(lines[0]["shown"]["ids"][0], vendors.as_str());
    assert_eq!(lines[1]["opened"], vendors.as_str());
    let after = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .count();
    assert_eq!(after, records);
}

/// `read --like --json` answers the request of `spec/fixtures/retrieve.json` with past objects of the
/// fixture's keys, and the request reads as a query (`spec/trails/14-trails.md`, "The past").
#[test]
fn read_like_json_matches_the_retrieval_contract() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../spec/fixtures/retrieve.json")).unwrap();
    let query: celis_trails::Query = serde_json::from_value(fixture["request"].clone()).unwrap();
    let repo = Repo::new();
    trail(
        &repo,
        &repo.path(),
        "Dark theme for the settings page",
        false,
    );
    trail(&repo, &repo.path(), "Fix the login form", false);
    let mut args = vec!["read", "--like", query.text.as_str()];
    for place in &query.places {
        args.extend(["--at", place.as_str()]);
    }
    let past: Vec<Value> = objects(&repo, &args)
        .into_iter()
        .filter_map(|v| v.get("past").cloned())
        .collect();
    let keys = |v: &Value| {
        let mut keys: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(past.len(), 1, "{past:?}");
    assert_eq!(keys(&past[0]), keys(&fixture["past"]));
    assert_eq!(past[0]["terms"], fixture["past"]["terms"]);
}
