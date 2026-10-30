use super::*;

/// The notice names both versions when they differ.
#[test]
fn a_notice_names_both_versions_when_they_differ() {
    let notice = notice("1.4.0", "1.3.0").unwrap();
    assert!(notice.starts_with("celis 1.4.0 is installed, and the celis plugin is 1.3.0."));
}

/// No notice when the versions match, one is a build from the repo, or one is not known.
#[test]
fn no_notice_when_the_versions_match_or_one_is_a_repo_build() {
    for (installed, plugin) in [
        ("1.4.0", "1.4.0"),
        ("0.0.0", "1.4.0"),
        ("1.4.0", "0.0.0"),
        ("1.4.0", ""),
    ] {
        assert_eq!(notice(installed, plugin), None, "{installed} {plugin}");
    }
}

/// Each agent reads the text in its own shape.
#[test]
fn each_agent_reads_the_text_in_its_shape() {
    assert_eq!(shape("claude", "SessionStart", "a \"b\""), "a \"b\"");
    assert_eq!(
        shape("cursor", "SessionStart", "a"),
        r#"{"additional_context":"a"}"#
    );
    let gemini: Value = serde_json::from_str(&shape("gemini", "BeforeAgent", "a")).unwrap();
    assert_eq!(gemini["hookSpecificOutput"]["hookEventName"], "BeforeAgent");
    assert_eq!(gemini["hookSpecificOutput"]["additionalContext"], "a");
    let antigravity: Value =
        serde_json::from_str(&shape("antigravity", "SessionStart", "a")).unwrap();
    assert_eq!(antigravity["injectSteps"][0]["ephemeralMessage"], "a");
}

/// A run of `celis` in a new project, with its own store.
struct Desk {
    home: tempfile::TempDir,
    project: tempfile::TempDir,
}

impl Desk {
    fn new() -> Desk {
        Desk {
            home: tempfile::tempdir().unwrap(),
            project: tempfile::tempdir().unwrap(),
        }
    }

    /// The command of `words`, in the folder `folder`.
    fn command(&self, words: &[&str], folder: &Path) -> Command {
        let home = self.home.path().display().to_string();
        Command {
            args: words.iter().map(|w| w.to_string()).collect(),
            env: [("CELIS_HOME".to_string(), home)].into(),
            folder: folder.to_path_buf(),
        }
    }

    /// What the session start in the project gives `agent`, from a hook that runs in `folder`.
    fn start(&self, agent: &str, folder: &Path) -> String {
        let json = json!({"session_id": "s1", "cwd": self.project.path()});
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/celis");
        self.command(&[], folder).start(agent, &json, &root)
    }
}

/// The session start gives the index of the patterns in scope in the project the JSON names, under one
/// heading line, after the card (`spec/02-cli.md`, "Hooks").
#[test]
fn the_session_start_gives_the_index_of_the_project() {
    let desk = Desk::new();
    let words = [
        "pattern",
        "--path",
        "*.svg",
        "--",
        "When a logo is drawn: keep it flat.",
    ];
    let write = desk.command(&words, desk.project.path());
    assert_eq!(write.run(&mut Vec::new(), &mut Vec::new()), 0);
    let mut index = Vec::new();
    let read = desk.command(&["patterns"], desk.project.path());
    assert_eq!(read.run(&mut index, &mut Vec::new()), 0);
    let index = String::from_utf8(index).unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let text = desk.start("claude", elsewhere.path());
    let given = format!("\n\nThe patterns in scope here:\n{}", index.trim_end());
    assert!(text.contains(&given), "{text}");
    assert!(text.find("How to do an ask").unwrap() < text.find(&given).unwrap());
}

/// The session start gives no index while the store holds no pattern (`spec/02-cli.md`, "Hooks").
#[test]
fn the_session_start_gives_no_index_without_a_pattern() {
    let desk = Desk::new();
    let text = desk.start("claude", desk.project.path());
    assert!(text.starts_with("How to do an ask"), "{text}");
    assert!(!text.contains("The patterns in scope here"), "{text}");
}

/// The first session start with no pattern in scope offers `/celis:learn` on the person's past chats,
/// and no later one does.
#[test]
fn the_first_session_start_without_a_pattern_offers_to_learn_once() {
    let desk = Desk::new();
    let first = desk.start("claude", desk.project.path());
    assert!(first.contains("offer them /celis:learn"), "{first}");
    let second = desk.start("claude", desk.project.path());
    assert!(!second.contains("/celis:learn"), "{second}");
}

/// A session start with a pattern in scope does not offer `/celis:learn`.
#[test]
fn a_session_start_with_a_pattern_does_not_offer_to_learn() {
    let desk = Desk::new();
    let words = ["pattern", "--", "When a logo is drawn: keep it flat."];
    let write = desk.command(&words, desk.project.path());
    assert_eq!(write.run(&mut Vec::new(), &mut Vec::new()), 0);
    let text = desk.start("claude", desk.project.path());
    assert!(!text.contains("/celis:learn"), "{text}");
}
