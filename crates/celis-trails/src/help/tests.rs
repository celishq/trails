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
    assert_eq!(Verb::of(&["read"]).map(|v| v.command), Some("read"));
    assert!(Verb::of(&["step"]).is_none());
}
