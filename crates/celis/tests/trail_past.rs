//! CIP-14, "The past".

mod support;

use std::path::Path;
use std::time::Duration;
use support::{add, cli, err, objects, out, Repo};

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
    run(&["step", "done", &part[..8], "--", "p"]);
    run(&["step", "done", &trail[..8], "--", "p"]);
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
        "{} [repo] Build the Vendors page · reached alone",
        &vendors[..8]
    )));
}
