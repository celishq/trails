use super::Failure;

#[test]
fn usage_exits_2_and_names_the_help_page() {
    let failure = Failure::usage("bad flag").see("celis trail help");
    assert_eq!((failure.name.as_str(), failure.code), ("usage", 2));
    assert_eq!(failure.message, "bad flag. See `celis trail help`.");
}

#[test]
fn an_unknown_word_names_the_nearest_known_one_within_two_edits() {
    let failure = Failure::unknown("lsit", ["list", "add"]);
    assert_eq!(failure.code, 2);
    assert!(failure.message.contains("did you mean \"list\"?"));
    let far = Failure::unknown("frobnicate", ["list", "add"]);
    assert_eq!(far.message, "no command \"frobnicate\"");
}

#[test]
fn any_other_error_exits_1_with_its_text() {
    let failure = Failure::from(anyhow::anyhow!("disk full"));
    assert_eq!((failure.name.as_str(), failure.code), ("error", 1));
    assert!(failure.message.contains("disk full"));
}
