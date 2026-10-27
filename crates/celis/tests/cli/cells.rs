//! CIP-09: cells.

use crate::support;

use celis_nostr::{Account, Note};
use support::{account, cli, err, kept, now, out, put, refusal, stored_pattern, Repo};

fn decide(repo: &Repo, text: &str) -> String {
    let o = cli(repo, &["pattern", "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// Another account, whose patterns and cells a test puts in the store.
fn theirs() -> Account {
    Account::new(&[7u8; 32]).unwrap()
}

/// A membership of `by` that puts `ids` in the cell `name`, put in the store.
fn membership(repo: &Repo, by: &Account, name: &str, ids: &[&str]) -> Note {
    let mut tags = vec![
        vec!["L".into(), "celis.cell".into()],
        vec!["l".into(), name.into(), "celis.cell".into()],
    ];
    tags.extend(ids.iter().map(|id| vec!["e".to_string(), id.to_string()]));
    let note = Note::write(by, 1985u16, now(), tags, "").unwrap();
    put(repo, &note);
    note
}

/// `cell add` writes one membership that names each pattern whole (`spec/09-cells.md`, "Membership").
#[test]
fn cell_add_writes_one_membership_that_names_each_pattern_whole() {
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    let b = decide(&repo, "When a dish is served\nDo: serve it hot");
    let o = cli(&repo, &["cell", "add", "eating-east", &a[..8], &b[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let written = kept(&repo, 1985, "eating-east");
    assert_eq!(written.len(), 1);
    let note = &written[0];
    assert_eq!(out(&o).trim(), note.id);
    assert_eq!(note.pubkey, account().pubkey());
    let named: Vec<&str> = note
        .tags
        .iter()
        .filter(|t| t[0] == "e")
        .map(|t| t[1].as_str())
        .collect();
    assert_eq!(named, [a.as_str(), b.as_str()]);
    assert!(note.tags.contains(&vec!["L".into(), "celis.cell".into()]));
}

/// `cell add` refuses a bad name or no id as a usage error, a pattern of another account with
/// `no-pattern`, and a name that a cell of another account holds with `cell-taken` (`spec/02-cli.md`,
/// "Cells").
#[test]
fn cell_add_refuses_a_bad_name_a_pattern_not_yours_and_a_taken_name() {
    let repo = Repo::new();
    let mine = decide(&repo, "When a menu is laid out\nDo: put soups first");
    let other = stored_pattern(
        &repo,
        (&repo.project_id(), "repo"),
        &theirs(),
        (None, None),
        "When rice is cooked\nDo: rinse it",
        now(),
    );
    membership(&repo, &theirs(), "eating-west", &[&other.id]);
    for args in [
        &["cell", "add", "Eating", &mine[..8]][..],
        &["cell", "add", "-east", &mine[..8]],
        &["cell", "add", "eating-east"],
    ] {
        let mut args = args.to_vec();
        args.push("--json");
        refusal(&cli(&repo, &args), 2, "usage");
    }
    let o = cli(
        &repo,
        &["cell", "add", "eating-east", &other.id[..8], "--json"],
    );
    refusal(&o, 1, "no-pattern");
    let o = cli(&repo, &["cell", "add", "eating-west", &mine[..8], "--json"]);
    refusal(&o, 1, "cell-taken");
    assert!(kept(&repo, 1985, "eating-east").is_empty());
}

/// `cell on` writes a mask of the working folder's project, and `cell off --scope me` one of every
/// project (`spec/09-cells.md`, "Masks").
#[test]
fn cell_on_and_off_write_a_mask_of_the_project_or_of_every_project() {
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    assert!(cli(&repo, &["cell", "add", "eating-east", &a[..8]])
        .status
        .success());
    let o = cli(&repo, &["cell", "on", "eating-east"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["cell", "off", "eating-east", "--scope", "me"]);
    assert!(o.status.success(), "{}", err(&o));
    let on = &kept(&repo, 1985, "on")[0];
    let off = &kept(&repo, 1985, "off")[0];
    let author = account().pubkey().hex();
    for mask in [on, off] {
        assert_eq!(mask.tag("L"), Some("celis.mask"));
        assert_eq!(mask.tag("cell"), Some("eating-east"));
        assert_eq!(mask.tag("p"), Some(author.as_str()));
    }
    let anchor = format!("project:{}", repo.project_id());
    assert_eq!(on.tag("I"), Some(anchor.as_str()));
    assert_eq!(on.tag("scope"), None);
    assert_eq!(off.tag("scope"), Some("me"));
    assert_eq!(off.tag("I"), None);
}

/// `cell on` refuses a name no cell holds with `no-cell`, a name two cells hold with `cell-taken`, and
/// another scope as a usage error (`spec/02-cli.md`, "Cells").
#[test]
fn cell_on_refuses_an_unknown_name_a_shared_name_and_another_scope() {
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    assert!(cli(&repo, &["cell", "add", "eating-east", &a[..8]])
        .status
        .success());
    let o = cli(&repo, &["cell", "on", "eating-west", "--json"]);
    refusal(&o, 1, "no-cell");
    let o = cli(
        &repo,
        &["cell", "on", "eating-east", "--scope", "project", "--json"],
    );
    refusal(&o, 2, "usage");
    let other = stored_pattern(
        &repo,
        (&repo.project_id(), "repo"),
        &theirs(),
        (None, None),
        "When rice is cooked\nDo: rinse it",
        now(),
    );
    membership(&repo, &theirs(), "eating-east", &[&other.id]);
    let o = cli(&repo, &["cell", "off", "eating-east", "--json"]);
    refusal(&o, 1, "cell-taken");
    assert!(kept(&repo, 1985, "on").is_empty());
}

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
            let mut tags = support::pattern_tags((&project, "repo"));
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
                _ => tags.extend(support::pattern_tags((&here, "repo")).into_iter().take(5)),
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

/// `cell off` right after `cell on` turns the cell off, also within one second (`spec/09-cells.md`,
/// "Masks").
#[test]
fn cell_off_right_after_cell_on_turns_the_cell_off() {
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    assert!(cli(&repo, &["cell", "add", "west", &a]).status.success());
    for _ in 0..5 {
        assert!(cli(&repo, &["cell", "on", "west"]).status.success());
        assert!(cli(&repo, &["cell", "off", "west"]).status.success());
        assert!(out(&cli(&repo, &["cells"])).trim().ends_with(" off"));
    }
}

/// The number of record files in the store.
fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared")).map_or(0, Iterator::count)
}

/// `cell export` writes the cell's live patterns and one new membership that names them to
/// `cell.jsonl`, the blob of each file example to `blobs/`, and `cell.md` with its links and credits,
/// and writes no record (`spec/09-cells.md`, "Export").
#[test]
fn cell_export_writes_the_records_the_blobs_and_a_page_and_no_record() {
    use sha2::Digest;
    let repo = Repo::new();
    repo.write("plate.png", "a plate");
    let a = decide(
        &repo,
        "Name: Soups first\nWhen a menu is laid out\nDo: put soups first",
    );
    let step = support::add(&repo, &["--", "Lay out the menu"]);
    support::finish(&repo, &step);
    let o = cli(
        &repo,
        &[
            "pattern",
            "--good",
            "plate.png",
            "--bad",
            "https://example.com/menu",
            "--good",
            &step,
            "--",
            "Name: Hot dishes\nWhen a dish is served\nDo: serve it hot",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let b = out(&o).trim().to_string();
    assert!(cli(&repo, &["cell", "add", "east", &a, &b])
        .status
        .success());
    let before = records(&repo);
    let o = cli(&repo, &["cell", "export", "east", "--to", "shared/east"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(records(&repo), before);
    let folder = repo.path().join("shared/east");
    let lines = std::fs::read_to_string(folder.join("cell.jsonl")).unwrap();
    let notes: Vec<Note> = lines
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(notes.len(), 3);
    assert!(notes.iter().all(Note::verifies));
    assert_eq!([&notes[0].id, &notes[1].id], [&a, &b]);
    let membership = &notes[2];
    assert_eq!(membership.tag("L"), Some("celis.cell"));
    let named: Vec<&str> = membership
        .tags
        .iter()
        .filter(|t| t[0] == "e")
        .map(|t| t[1].as_str())
        .collect();
    assert_eq!(named, [a.as_str(), b.as_str()]);
    let sha = format!("{:x}", sha2::Sha256::digest(b"a plate"));
    let blob = format!("blobs/{sha}.png");
    assert_eq!(std::fs::read(folder.join(&blob)).unwrap(), b"a plate");
    let page = std::fs::read_to_string(folder.join("cell.md")).unwrap();
    for part in [
        "# east",
        "## Soups first",
        "## Hot dishes",
        &format!("- good: ![plate.png]({blob})"),
        "- bad: <https://example.com/menu>",
        "https://celis.ai",
        &account().pubkey().npub(),
    ] {
        assert!(page.contains(part), "{part}:\n{page}");
    }
    assert!(!page.contains(&step[..8]), "{page}");
}

/// `cell export` refuses a cell of another account and a name no cell holds with `no-cell`
/// (`spec/02-cli.md`, "Cells").
#[test]
fn cell_export_refuses_a_cell_not_yours() {
    let repo = Repo::new();
    let other = stored_pattern(
        &repo,
        (&repo.project_id(), "repo"),
        &theirs(),
        (None, None),
        "When rice is cooked\nDo: rinse it",
        now(),
    );
    membership(&repo, &theirs(), "west", &[&other.id]);
    for name in ["west", "east"] {
        let o = cli(&repo, &["cell", "export", name, "--json"]);
        refusal(&o, 1, "no-cell");
    }
    assert!(!repo.path().join("west").exists());
}

/// A folder that `theirs` exported: one pattern on another project, with a file example, and the
/// membership of `name` that names it.
fn their_folder(repo: &Repo, name: &str) -> (std::path::PathBuf, Note) {
    let folder = repo.path().join("from-them");
    std::fs::create_dir_all(folder.join("blobs")).unwrap();
    let sha = {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(b"a bowl"))
    };
    let pubkey = theirs().pubkey().hex();
    let mut tags = support::pattern_tags(("q", "kitchen"));
    tags.push(vec![
        "good".into(),
        format!("blossom:{sha}.png?as={pubkey}&sz=6"),
        "bowl.png".into(),
    ]);
    let text = "Name: Rice first\nWhen a meal of rice is set\nDo: serve the rice first";
    let pattern = Note::write(&theirs(), 1111u16, now() - 10, tags, text).unwrap();
    let tags = vec![
        vec!["L".to_string(), "celis.cell".to_string()],
        vec!["l".to_string(), name.to_string(), "celis.cell".to_string()],
        vec!["e".to_string(), pattern.id.clone()],
    ];
    let membership = Note::write(&theirs(), 1985u16, now() - 5, tags, "").unwrap();
    let lines = [&pattern, &membership].map(|n| serde_json::to_string(n).unwrap());
    std::fs::write(folder.join("cell.jsonl"), lines.join("\n") + "\n").unwrap();
    std::fs::write(folder.join(format!("blobs/{sha}.png")), "a bowl").unwrap();
    std::fs::write(folder.join("cell.md"), "# east\n").unwrap();
    (folder, pattern)
}

/// `cell import` shows the cell and keeps nothing; with `--yes` it keeps the records and the blob, and
/// the cell stays off (`spec/02-cli.md`, "Cells").
#[test]
fn cell_import_shows_the_cell_and_keeps_it_off_with_yes() {
    let repo = Repo::new();
    let (_, pattern) = their_folder(&repo, "east");
    let before = records(&repo);
    let o = cli(&repo, &["cell", "import", "from-them"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        out(&o).contains(&format!("{} Rice first", &pattern.id[..8])),
        "{}",
        out(&o)
    );
    assert_eq!(records(&repo), before);
    let o = cli(&repo, &["cell", "import", "from-them", "--yes"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(records(&repo), before + 2);
    assert!(out(&cli(&repo, &["cells"]))
        .trim()
        .ends_with("1 pattern off"));
    assert!(!out(&cli(&repo, &["patterns"])).contains(&pattern.id[..8]));
    let sha = {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(b"a bowl"))
    };
    assert!(repo.store().join("blobs").join(&sha).exists());
    assert!(cli(&repo, &["cell", "on", "east"]).status.success());
    assert!(out(&cli(&repo, &["patterns"])).contains(&pattern.id[..8]));
}

/// `cell import` refuses a folder whose record was edited, or whose pattern the membership does not
/// name, with `invalid`, and keeps nothing (`spec/09-cells.md`, "Export").
#[test]
fn cell_import_refuses_an_edited_record_and_a_pattern_out_of_the_cell() {
    let repo = Repo::new();
    let (folder, pattern) = their_folder(&repo, "east");
    let lines = std::fs::read_to_string(folder.join("cell.jsonl")).unwrap();
    let edited = lines.replace("serve the rice first", "serve the soup first");
    std::fs::write(folder.join("cell.jsonl"), &edited).unwrap();
    let before = records(&repo);
    let o = cli(&repo, &["cell", "import", "from-them", "--yes", "--json"]);
    refusal(&o, 1, "invalid");
    let stray = stored_pattern(
        &repo,
        ("q", "kitchen"),
        &theirs(),
        (None, None),
        "When soup is served\nDo: serve it hot",
        now(),
    );
    let mut lines: Vec<String> = lines.lines().map(str::to_string).collect();
    lines.insert(1, serde_json::to_string(&stray).unwrap());
    std::fs::write(folder.join("cell.jsonl"), lines.join("\n")).unwrap();
    let o = cli(&repo, &["cell", "import", "from-them", "--yes", "--json"]);
    refusal(&o, 1, "invalid");
    assert_eq!(records(&repo), before + 1);
    assert!(!out(&cli(&repo, &["cells"])).contains(&pattern.id[..8]));
}

/// An edit to `cell.md` changes nothing, and a cell whose name a cell of another account holds is
/// refused with `cell-taken` (`spec/09-cells.md`, "Export").
#[test]
fn cell_import_ignores_cell_md_and_refuses_a_taken_name() {
    let repo = Repo::new();
    let (folder, _) = their_folder(&repo, "east");
    std::fs::write(folder.join("cell.md"), "# something else\n").unwrap();
    assert!(cli(&repo, &["cell", "import", "from-them", "--yes"])
        .status
        .success());
    let repo = Repo::new();
    let a = decide(&repo, "When a menu is laid out\nDo: put soups first");
    assert!(cli(&repo, &["cell", "add", "east", &a]).status.success());
    their_folder(&repo, "east");
    let o = cli(&repo, &["cell", "import", "from-them", "--yes", "--json"]);
    refusal(&o, 1, "cell-taken");
}

/// Serves the files of `folder` over HTTP on a local port until the test ends; its base URL.
fn serve(folder: std::path::PathBuf) -> String {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = [0u8; 4096];
            let read = stream.read(&mut request).unwrap_or(0);
            let request = String::from_utf8_lossy(&request[..read]);
            let path = request
                .split(' ')
                .nth(1)
                .unwrap_or("/")
                .trim_start_matches('/');
            let reply = match std::fs::read(folder.join(path)) {
                Ok(body) => [
                    format!("HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n", body.len())
                        .into_bytes(),
                    body,
                ]
                .concat(),
                Err(_) => b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec(),
            };
            let _ = stream.write_all(&reply);
        }
    });
    format!("http://127.0.0.1:{port}")
}

/// `cell import <url>` fetches `cell.jsonl` and the blobs beside it, and takes a blob only when its
/// bytes hash to its name (`spec/09-cells.md`, "Export").
#[test]
fn cell_import_from_a_url_takes_only_blobs_that_match_their_hash() {
    let repo = Repo::new();
    let (folder, pattern) = their_folder(&repo, "east");
    let url = serve(folder.clone());
    let o = cli(&repo, &["cell", "import", &format!("{url}/"), "--yes"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains(&pattern.id[..8]), "{}", out(&o));
    let sha = {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(b"a bowl"))
    };
    assert!(repo.store().join("blobs").join(&sha).exists());
    let repo = Repo::new();
    let (folder, _) = their_folder(&repo, "east");
    std::fs::write(folder.join(format!("blobs/{sha}.png")), "not a bowl").unwrap();
    let url = serve(folder);
    let o = cli(
        &repo,
        &["cell", "import", &format!("{url}/cell.jsonl"), "--yes"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&cli(&repo, &["cells"])).contains("east"));
    let blobs = std::fs::read_dir(repo.store().join("blobs")).map_or(0, Iterator::count);
    assert_eq!(blobs, 0);
}
