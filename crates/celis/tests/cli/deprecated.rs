//! What celis does with a name that older plugins give.

use crate::support;

use support::{add, cli, err, Repo};

/// Each deprecated flag is taken and named as deprecated on stderr, first, but with `--json`
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_deprecated_flag_is_taken_with_a_notice() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &[
            "pattern",
            "--binds",
            "--cell",
            "cli",
            "--",
            "When you test: set A=1.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let notices =
        "celis: --binds is deprecated and ignored\ncelis: --cell is deprecated and ignored\n";
    assert!(err(&o).starts_with(notices), "{}", err(&o));
    let pattern = support::out(&o).lines().next().unwrap().to_string();
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--new",
            "--recalled",
            &pattern[..8],
            "--",
            "Test the app",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let notices =
        "celis: --new is deprecated and ignored\ncelis: --recalled is deprecated and ignored\n";
    assert!(err(&o).starts_with(notices), "{}", err(&o));
    let step = add(&repo, &["--", "Run the tests"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &step[..8],
            "--as-planned",
            "--by",
            "judge",
            "--",
            "tests pass",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        err(&o).starts_with("celis: --by judge is deprecated and ignored\n"),
        "{}",
        err(&o)
    );
    let o = cli(
        &repo,
        &[
            "pattern",
            "--json",
            "--binds",
            "--",
            "When you lint: run ruff.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(!err(&o).contains("deprecated"), "{}", err(&o));
}
