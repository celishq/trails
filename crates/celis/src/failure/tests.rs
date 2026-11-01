use super::Failure;
use celis_store::Refusal;

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
        Failure::from(Refusal::NoStep("3fa2".to_string())),
        Failure::from(Refusal::NoPattern("3fa2".to_string())),
        Failure::from(Refusal::Closed("3fa2".to_string())),
        Failure::from(Refusal::OpenSteps("3fa2".to_string())),
    ] {
        assert!(failure.message.contains("`celis "), "{}", failure.message);
    }
}
