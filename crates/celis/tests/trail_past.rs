//! CIP-14, "The past".

mod support;

use celis_wire::Note;
use std::path::Path;
use std::time::Duration;
use support::{account, add, cli, done_note, err, now, objects, out, step_note, Repo};

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

/// `read --like` prints the done trails of every project that share a word with the text, 3 at most:
/// the most shared words first, then the fewest steers and rework, then the newest
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
