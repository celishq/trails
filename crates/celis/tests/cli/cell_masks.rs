//! CIP-09, "Masks": `celis cell on` and `celis cell off`.

use crate::support;

use support::cell::{decide, membership, theirs};
use support::{account, cli, err, kept, now, out, refusal, stored_pattern, Repo};

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
