use super::Failure;

#[test]
fn any_other_error_exits_1_with_its_text() {
    let failure = Failure::from(anyhow::anyhow!("disk full"));
    assert_eq!((failure.name.as_str(), failure.code), ("error", 1));
    assert!(failure.message.contains("disk full"));
}

/// An error about an id or a step's state names the `celis` command that shows or fixes it.
#[test]
fn an_error_about_an_id_names_the_command_that_fixes_it() {
    for failure in [
        Failure::no_step("3fa2"),
        Failure::no_rule("3fa2"),
        Failure::no_trail("3fa2"),
        Failure::closed("3fa2"),
        Failure::open_steps("3fa2"),
    ] {
        assert!(failure.message.contains("`celis "), "{}", failure.message);
    }
}
