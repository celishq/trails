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
