//! The brief `setup` prints to install the plugin into each agent on the machine.

use crate::support;

use std::path::Path;
use support::{err, out, Repo};

/// A folder of empty files named `names`, to stand on PATH for the agents' commands.
fn bin(names: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for name in names {
        std::fs::write(dir.path().join(name), "").unwrap();
    }
    dir
}

/// What `celis setup` prints with only `path` on PATH; it must succeed.
fn setup(repo: &Repo, path: &Path) -> String {
    let path = path.to_str().unwrap();
    let o = repo.run(&repo.path(), &["setup"], &[("PATH", path)], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// For each agent on PATH, `setup` names the commands that add the plugin, what only the person can
/// do, and the status line that shows it works; an agent not on PATH is left out.
#[test]
fn setup_names_the_commands_for_each_agent_on_path() {
    let repo = Repo::new();
    let path = bin(&["claude", "codex"]);
    let shown = setup(&repo, path.path());
    for line in [
        "  claude plugin marketplace add celishq/trails",
        "  claude plugin install celis@celis",
        "  codex plugin marketplace add celishq/trails",
        "  codex plugin add celis@celis",
        "  - Restart each agent above, so that the plugin loads.",
        "  - In Codex, trust the celis hooks with /hooks.",
        "  - In the first session, celis offers to learn patterns from your past chats.",
    ] {
        assert!(shown.lines().any(|l| l == line), "{line:?} in {shown}");
    }
    assert!(!shown.contains("gemini"), "{shown}");
    assert!(shown.contains("celis status line"), "{shown}");
    assert!(!shown.contains("setup again"), "{shown}");
}

/// With no agent it knows on PATH, `setup` says so and points to the install steps.
#[test]
fn setup_with_no_agent_on_path_points_to_the_install_steps() {
    let repo = Repo::new();
    let path = bin(&[]);
    let shown = setup(&repo, path.path());
    assert!(shown.starts_with("celis "), "{shown}");
    assert!(!shown.starts_with("celis celis"), "{shown}");
    assert!(
        shown.contains("https://github.com/celishq/trails#install"),
        "{shown}"
    );
}
