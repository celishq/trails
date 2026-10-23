use super::Vectors;

#[test]
fn saved_vectors_open_again() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("index/m.bin");
    let mut vectors = Vectors::open(&file, "m", 2);
    vectors.insert("keep prices in cents", vec![0.6, 0.8]);
    vectors.save(&file).unwrap();
    let opened = Vectors::open(&file, "m", 2);
    assert_eq!(opened.get("keep prices in cents"), Some(&[0.6, 0.8][..]));
    assert_eq!(opened.get("other"), None);
}

#[test]
fn vectors_of_another_model_or_a_bad_file_open_empty() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("m.bin");
    let mut vectors = Vectors::open(&file, "m", 2);
    vectors.insert("a", vec![1.0, 0.0]);
    vectors.save(&file).unwrap();
    assert_eq!(Vectors::open(&file, "other", 2).get("a"), None);
    assert_eq!(Vectors::open(&file, "m", 3).get("a"), None);
    let mut bytes = std::fs::read(&file).unwrap();
    bytes.pop();
    std::fs::write(&file, bytes).unwrap();
    assert_eq!(Vectors::open(&file, "m", 2).get("a"), None);
}

#[test]
fn keep_drops_the_texts_not_given() {
    let mut vectors = Vectors::open(std::path::Path::new("/nonexistent"), "m", 1);
    vectors.insert("a", vec![1.0]);
    vectors.insert("b", vec![1.0]);
    vectors.keep(["a"]);
    assert!(vectors.get("a").is_some());
    assert_eq!(vectors.get("b"), None);
}
