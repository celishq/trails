//! CIP-04, "The past".

mod support;

use celis_nostr::Note;
use celis_trails::{Anchor, Step};
use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use support::{account, cli, done_note, err, now, objects, out, put, step_note, Repo};

/// A done trail saying `text` in `folder`, with a done step under it.
fn trail(repo: &Repo, folder: &Path, text: &str) -> String {
    let run = |args: &[&str]| {
        let o = repo.run(folder, args, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        out(&o).trim().to_string()
    };
    let trail = run(&["step", "add", "--", text]);
    let part = run(&["step", "add", "--under", &trail[..8], "--", "the part"]);
    run(&["step", "done", &part[..8], "--as-planned", "--", "p"]);
    run(&["step", "done", &trail[..8], "--as-planned", "--", "p"]);
    std::thread::sleep(Duration::from_millis(1100));
    trail
}

/// The ids of the past objects `celis past --json` prints, in order.
fn listed(repo: &Repo) -> Vec<String> {
    objects(repo, &["past"])
        .into_iter()
        .filter_map(|v| v.get("past").map(|p| p["id"].as_str().unwrap().to_string()))
        .collect()
}

/// `past` lists the done asks and reached trails of every project, newest first. It leaves out open work
/// and a trail that ended with no `done` (`spec/trails/04-trails.md`, "The past").
#[test]
fn past_lists_done_asks_and_reached_trails_of_every_project_newest_first() {
    let repo = Repo::new();
    let by = account();
    let t = now() - 3 * 24 * 60 * 60;
    let shop = Anchor {
        project: celis_trails::Project::at(&repo.other("shop")).id,
        name: "shop".to_string(),
    };
    let reached = Step {
        anchor: shop,
        under: None,
        replaces: None,
        uses: Vec::new(),
        places: vec!["Export.swift".to_string()],
        check: None,
        session: None,
        content: "Add the export button".to_string(),
    }
    .record(&by, t)
    .unwrap();
    put(&repo, &reached);
    done_note(&repo, &by, &reached, t + 1);
    let ended = step_note(&repo, &by, None, "Polish the terminal", t + 10);
    let watch = step_note(&repo, &by, Some(&ended), "watch redraws on resize", t + 11);
    done_note(&repo, &by, &watch, t + 12);
    step_note(&repo, &by, Some(&ended), "tree columns", t + 13);
    step_note(&repo, &by, None, "Fix the login", now() - 60);
    assert_eq!(listed(&repo), [watch.id, reached.id]);
}

/// `past` leaves out a trail with an ask of its own text, so the ask stands for it
/// (`spec/trails/04-trails.md`, "The past").
#[test]
fn past_lists_the_ask_in_place_of_a_trail_of_its_text() {
    let repo = Repo::new();
    let by = account();
    let t = now() - 24 * 60 * 60;
    let trail: Note = step_note(&repo, &by, None, "Build the Vendors page", t);
    let ask = step_note(&repo, &by, Some(&trail), "Build the Vendors page", t + 1);
    done_note(&repo, &by, &ask, t + 2);
    done_note(&repo, &by, &trail, t + 3);
    assert_eq!(listed(&repo), [ask.id]);
}

/// `past` prints each past work on one line: its id, project, title and place, its grade, and its day
/// (`spec/trails/04-trails.md`, "The past").
#[test]
fn past_prints_one_tagged_line_for_each_past_work() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let settings = trail(&repo, &shop, "Build the Settings page");
    let o = cli(&repo, &["past"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let line = text
        .lines()
        .find(|l| l.starts_with(&settings[..8]))
        .unwrap_or_else(|| panic!("{text}"));
    assert!(
        line.contains(" [shop] Build the Settings page · took "),
        "{line}"
    );
    let day = objects(&repo, &["past"])[0]["past"]["day"].clone();
    assert!(line.ends_with(day.as_str().unwrap()), "{line}");
}

/// `past --json` prints each past work as a `past` object with its tags
/// (`spec/trails/04-trails.md`, "The past").
#[test]
fn past_json_carries_each_past_works_tags() {
    let repo = Repo::new();
    let id = trail(&repo, &repo.path(), "Dark theme for the settings page");
    let past: Vec<Value> = objects(&repo, &["past"])
        .into_iter()
        .filter_map(|v| v.get("past").cloned())
        .filter(|p| p["id"] == id.as_str())
        .collect();
    let mut keys: Vec<&String> = past[0].as_object().unwrap().keys().collect();
    keys.sort();
    let want = [
        "content",
        "created_at",
        "day",
        "grade",
        "id",
        "places",
        "project",
        "state",
    ];
    assert_eq!(keys, want);
    assert_eq!(past[0]["project"], "repo");
}
