//! CIP-05, "Cells", and CIP-02, "Commands" and "Search".

mod support;

use serde_json::json;
use support::{add, cli, err, labelled, objects, out, Repo};

/// The whole id `celis pattern <args> -- <text>` prints.
fn decide(repo: &Repo, args: &[&str], text: &str) -> String {
    let mut all = vec!["pattern"];
    all.extend(args);
    all.extend(["--", text]);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// The `cell` tags of the stored pattern `id`.
fn cells(repo: &Repo, id: &str) -> Vec<String> {
    let notes = labelled(repo, "pattern");
    let note = notes.iter().find(|n| n.id == id).unwrap();
    let tags = note.tags.iter().filter(|t| t[0] == "cell");
    tags.map(|t| t[1].clone()).collect()
}

/// `pattern --cell` writes each cell once in its normal form, and a pattern that replaces it keeps them
/// (`spec/05-patterns.md`, "Cells").
#[test]
fn pattern_writes_each_cell_in_its_form_and_a_replacement_keeps_it() {
    let repo = Repo::new();
    let args = ["--cell", "CLI Help", "--cell", "cli-help", "--cell", "Grep"];
    let old = decide(&repo, &args, "When a flag gets help: keep it short.");
    assert_eq!(cells(&repo, &old), ["cli-help", "grep"]);
    let args = ["--replaces", &old[..8], "--cell", "flags", "--cell", "GREP"];
    let new = decide(
        &repo,
        &args,
        "When a flag gets help: keep it to 57 characters.",
    );
    assert_eq!(cells(&repo, &new), ["flags", "grep", "cli-help"]);
}

/// `pattern` refuses a cell with no letter or digit as a usage error (`spec/02-cli.md`, "Commands").
#[test]
fn pattern_refuses_an_empty_cell() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &["pattern", "--cell", " - ", "--", "When a flag: keep it."],
    );
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(labelled(&repo, "pattern").is_empty());
}

/// A pattern's line shows each of its cells, and its JSON names them (`spec/02-cli.md`, "Search").
#[test]
fn a_pattern_shows_its_cells() {
    let repo = Repo::new();
    let args = ["--cell", "cli-help", "--cell", "flags"];
    let id = decide(&repo, &args, "When a flag gets help: keep it short.");
    let said = out(&cli(&repo, &["search", "--", "a flag gets help"]));
    assert!(
        said.contains("  cell    cli-help\n  cell    flags\n"),
        "{said}"
    );
    let step = add(&repo, &["--new", "--uses", &id, "--", "t"]);
    let step = step.lines().next().unwrap();
    let shown = objects(&repo, &["step", "start", step]);
    let pattern = shown.iter().find_map(|v| v.get("pattern")).unwrap();
    assert_eq!(pattern["cells"], json!(["cli-help", "flags"]));
}

/// `grep` finds a pattern by its cell (`spec/02-cli.md`, "Grep").
#[test]
fn grep_finds_a_pattern_by_its_cell() {
    let repo = Repo::new();
    let id = decide(
        &repo,
        &["--cell", "CLI Help"],
        "When a flag gets help: keep it short.",
    );
    let said = out(&cli(&repo, &["grep", "-F", " cell:cli-help "]));
    assert!(said.contains(&id[..8]), "{said}");
}
