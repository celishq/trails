use super::Glob;

#[test]
fn a_star_matches_within_one_segment() {
    let glob = Glob::new("src/*.rs");
    assert!(glob.matches("src/main.rs"));
    assert!(!glob.matches("src/command/rules.rs"));
    assert!(!glob.matches("main.rs"));
}

#[test]
fn a_double_star_matches_across_segments() {
    let glob = Glob::new("**/*.py");
    assert!(glob.matches("a.py"));
    assert!(glob.matches("shelf/prices/a.py"));
    assert!(!glob.matches("a.pyc"));
    assert!(Glob::new("docs/**").matches("docs/a/b.md"));
    assert!(Glob::new("src/**/tests.rs").matches("src/tests.rs"));
    assert!(Glob::new("src/**/tests.rs").matches("src/a/b/tests.rs"));
}

#[test]
fn a_question_mark_matches_one_character() {
    let glob = Glob::new("v?.txt");
    assert!(glob.matches("v1.txt"));
    assert!(!glob.matches("v10.txt"));
    assert!(!Glob::new("a?b").matches("a/b"));
}

#[test]
fn a_literal_matches_only_its_own_path() {
    let glob = Glob::new("./shelf/prices.py");
    assert!(glob.matches("shelf/prices.py"));
    assert!(!glob.matches("shelf/prices.pyc"));
}

#[test]
fn other_characters_match_themselves() {
    assert!(Glob::new("a+b(1)*.md").matches("a+b(1)x.md"));
    assert!(!Glob::new("a.*").matches("abc"));
}
