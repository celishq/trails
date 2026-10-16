use super::*;

#[test]
fn a_long_usage_wraps_before_a_flag() {
    let page = Verb {
        command: "rate",
        usage: &["celis rate <id> [--note <a long note, long enough to pass the width>] [--other <another long flag value>] [--third <a third long flag value>]"],
        summary: "Rate a step.",
        args: &[],
        examples: &[],
    }
    .page();
    let lines: Vec<&str> = page.lines().collect();
    assert!(lines[1].starts_with("      ["), "{page}");
}

#[test]
fn the_longest_command_the_words_begin_with_names_the_verb() {
    assert_eq!(
        Verb::of(&["step", "add", "x"]).map(|v| v.command),
        Some("step add")
    );
    assert_eq!(Verb::of(&["grep"]).map(|v| v.command), Some("grep"));
    assert!(Verb::of(&["step"]).is_none());
}

/// An unknown word names the nearest known one within two edits (`spec/02-cli.md`, "Output and
/// errors").
#[test]
fn an_unknown_word_names_the_nearest_known_one_within_two_edits() {
    let failure = unknown("lsit", ["list", "add"]);
    assert_eq!(failure.code, 2);
    assert!(failure.message.contains("did you mean \"list\"?"));
    let far = unknown("frobnicate", ["list", "add"]);
    assert_eq!(far.message, "no command \"frobnicate\"");
}
