use super::Verified;

/// A list that names a file more than once, or holds a line it cannot read, is written again with one
/// line for each file, the last one it names.
#[test]
fn open_writes_the_list_again_with_one_line_a_file() {
    let home = tempfile::tempdir().unwrap();
    let file = home.path().join("verified");
    std::fs::write(&file, "a.json 1 1\nb.json 2 2\na.json 3 3\nbroken\n").unwrap();
    let verified = Verified::open(file.clone());
    let mut lines: Vec<String> = std::fs::read_to_string(&file)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    lines.sort();
    assert_eq!(lines, ["a.json 3 3", "b.json 2 2"]);
    assert_eq!(verified.known.len(), 2);
}
