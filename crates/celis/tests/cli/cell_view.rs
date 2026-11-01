//! CIP-09, "The view".

use crate::support;

use celis_nostr::Note;
use support::cell::{decide, membership, their_folder, theirs};
use support::{account, cli, err, kept, now, out, pattern_tags, put, refusal, Repo};

const FIXTURE: &str = include_str!("../../../../spec/fixtures/cells.json");

/// The tag names of `tags`, with the label of an `l` tag, as the fixture gives them.
fn shape(tags: &[Vec<String>]) -> Vec<String> {
    let name = |t: &Vec<String>| match t[0].as_str() {
        "l" | "L" => format!("{} {}", t[0], t.last().unwrap()),
        name => name.to_string(),
    };
    tags.iter().map(name).collect()
}

/// `cell add` and `cell on` write the membership and the mask in the form the fixture gives
/// (`spec/09-cells.md`, "Membership", "Masks").
#[test]
fn the_records_match_the_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    let tags =
        |v: &serde_json::Value| -> Vec<Vec<String>> { serde_json::from_value(v.clone()).unwrap() };
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    assert!(cli(&repo, &["cell", "add", "west", &a]).status.success());
    assert!(cli(&repo, &["cell", "on", "west"]).status.success());
    assert!(cli(&repo, &["cell", "on", "west", "--scope", "me"])
        .status
        .success());
    let membership = &kept(&repo, 1985, "west")[0];
    assert_eq!(
        shape(&membership.tags),
        shape(&tags(&fixture["membership"]))
    );
    let mut masks = kept(&repo, 1985, "on");
    masks.sort_by_key(|m| m.tag("scope").is_some());
    assert_eq!(
        shape(&masks[0].tags),
        shape(&tags(&fixture["mask"]["project"]))
    );
    assert_eq!(shape(&masks[1].tags), shape(&tags(&fixture["mask"]["me"])));
}

/// In each case of the fixture, `patterns` shows the patterns of its view, and only those
/// (`spec/09-cells.md`, "The view").
#[test]
fn patterns_shows_the_view_of_each_case() {
    let fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let repo = Repo::new();
        let here = repo.project_id();
        let by = |who: &serde_json::Value| match who.as_str().unwrap() {
            "mine" => account(),
            _ => theirs(),
        };
        let mut at = now() - 1000;
        let mut ids: Vec<(String, String)> = vec![];
        for pattern in case["patterns"].as_array().unwrap() {
            let letter = pattern["id"].as_str().unwrap();
            let project = match pattern["project"].as_str() {
                Some(other) => other.to_string(),
                None => here.clone(),
            };
            let mut tags = pattern_tags((&project, "repo"));
            if let Some(old) = pattern["replaces"].as_str() {
                let old = &ids.iter().find(|(l, _)| l == old).unwrap().1;
                tags.push(vec!["replaces".into(), old.clone()]);
            }
            at += 1;
            let text = format!("When case {letter} holds\nDo: {letter}");
            let note = Note::write(&by(&pattern["by"]), 1111u16, at, tags, text).unwrap();
            put(&repo, &note);
            ids.push((letter.to_string(), note.id));
        }
        let id = |letter: &str| ids.iter().find(|(l, _)| l == letter).unwrap().1.clone();
        for cell in case["cells"].as_array().unwrap() {
            let named: Vec<String> = cell["patterns"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| id(l.as_str().unwrap()))
                .collect();
            let named: Vec<&str> = named.iter().map(String::as_str).collect();
            membership(
                &repo,
                &by(&cell["by"]),
                cell["name"].as_str().unwrap(),
                &named,
            );
        }
        for mask in case["masks"].as_array().unwrap() {
            let author = case["cells"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["name"] == mask["cell"])
                .map(|c| by(&c["by"]).pubkey().hex())
                .unwrap();
            let state = mask["l"].as_str().unwrap();
            let mut tags = vec![
                vec!["L".into(), "celis.mask".into()],
                vec!["l".into(), state.into(), "celis.mask".into()],
                vec!["cell".into(), mask["cell"].as_str().unwrap().into()],
                vec!["p".into(), author],
            ];
            match mask["scope"].as_str().unwrap() {
                "me" => tags.push(vec!["scope".into(), "me".into()]),
                _ => tags.extend(pattern_tags((&here, "repo")).into_iter().take(5)),
            }
            at += 1;
            put(
                &repo,
                &Note::write(&by(&mask["by"]), 1985u16, at, tags, "").unwrap(),
            );
        }
        let shown = out(&cli(&repo, &["patterns"]));
        let mut seen: Vec<String> = ids
            .iter()
            .filter(|(_, id)| shown.contains(&id[..8]))
            .map(|(l, _)| l.clone())
            .collect();
        seen.sort();
        let view: Vec<&str> = case["view"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(seen, view, "{name}:\n{shown}");
    }
}

/// `grep` gives a pattern of a cell that is on with ` cell:<name>`, and leaves out one of a cell that
/// is off (`spec/02-cli.md`, "Grep").
#[test]
fn grep_names_the_cell_that_is_on_and_leaves_out_one_that_is_off() {
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    assert!(cli(&repo, &["cell", "add", "west", &a]).status.success());
    let shown = out(&cli(&repo, &["grep", "soups"]));
    assert!(!shown.contains(&a[..8]), "{shown}");
    assert!(cli(&repo, &["cell", "on", "west"]).status.success());
    let shown = out(&cli(&repo, &["grep", "soups"]));
    assert!(
        shown.contains(&format!("{} live {{project}} cell:west", &a[..8])),
        "{shown}"
    );
}

/// `cells` gives each cell with its account, its live patterns and whether it is on here, and
/// `patterns --cell` gives the patterns of one cell, on or off (`spec/02-cli.md`, "Cells").
#[test]
fn cells_lists_each_cell_and_patterns_cell_shows_its_patterns() {
    let repo = Repo::new();
    assert_eq!(out(&cli(&repo, &["cells"])).trim(), "no cell yet");
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    let b = decide(&repo, "When a dish is served\nDo: serve it hot");
    assert!(cli(&repo, &["cell", "add", "west", &a, &b])
        .status
        .success());
    let npub = account().pubkey().npub();
    let line = format!("west {} 2 patterns off", &npub[5..13]);
    assert_eq!(out(&cli(&repo, &["cells"])).trim(), line);
    let shown = out(&cli(&repo, &["patterns", "--cell", "west"]));
    assert!(
        shown.contains(&a[..8]) && shown.contains(&b[..8]),
        "{shown}"
    );
    assert!(!out(&cli(&repo, &["patterns"])).contains(&a[..8]));
    assert!(cli(&repo, &["cell", "on", "west"]).status.success());
    assert!(out(&cli(&repo, &["cells"]))
        .trim()
        .ends_with("2 patterns on"));
    let o = cli(&repo, &["patterns", "--cell", "east", "--json"]);
    refusal(&o, 1, "no-cell");
}

/// Your pattern that replaces an imported pattern shows while the cell is on, and leaves with the cell
/// when it is off (`spec/09-cells.md`, "The view").
#[test]
fn your_replacement_of_an_imported_pattern_shows_only_while_its_cell_is_on() {
    let repo = Repo::new();
    let (_, theirs) = their_folder(&repo, "east");
    assert!(cli(&repo, &["cell", "import", "from-them", "--yes"])
        .status
        .success());
    let o = cli(
        &repo,
        &["pattern", "--replaces", &theirs.id[..8], "--", "x"],
    );
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(cli(&repo, &["cell", "on", "east"]).status.success());
    let o = cli(
        &repo,
        &[
            "pattern",
            "--replaces",
            &theirs.id[..8],
            "--",
            "Name: Rice last\nWhen a meal of rice is set\nDo: serve the rice last",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let mine = out(&o).trim().to_string();
    let shown = out(&cli(&repo, &["patterns"]));
    assert!(
        shown.contains(&mine[..8]) && !shown.contains(&theirs.id[..8]),
        "{shown}"
    );
    assert!(cli(&repo, &["cell", "off", "east"]).status.success());
    let shown = out(&cli(&repo, &["patterns"]));
    assert!(
        !shown.contains(&mine[..8]) && !shown.contains(&theirs.id[..8]),
        "{shown}"
    );
}
