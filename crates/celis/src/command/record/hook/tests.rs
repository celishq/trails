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
