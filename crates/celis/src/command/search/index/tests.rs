use super::Index;

#[test]
fn a_saved_index_opens_with_its_vectors() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("index/m.bin");
    let mut index = Index::open(&file, "m", 2);
    index.insert("keep prices in cents", vec![0.6, 0.8]);
    index.save(&file).unwrap();
    let opened = Index::open(&file, "m", 2);
    assert_eq!(opened.get("keep prices in cents"), Some(&[0.6, 0.8][..]));
    assert_eq!(opened.get("other"), None);
}

#[test]
fn an_index_of_another_model_or_a_bad_file_opens_empty() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("m.bin");
    let mut index = Index::open(&file, "m", 2);
    index.insert("a", vec![1.0, 0.0]);
    index.save(&file).unwrap();
    assert_eq!(Index::open(&file, "other", 2).get("a"), None);
    assert_eq!(Index::open(&file, "m", 3).get("a"), None);
    let mut bytes = std::fs::read(&file).unwrap();
    bytes.pop();
    std::fs::write(&file, bytes).unwrap();
    assert_eq!(Index::open(&file, "m", 2).get("a"), None);
}

#[test]
fn keep_drops_the_texts_not_given() {
    let mut index = Index::open(std::path::Path::new("/nonexistent"), "m", 1);
    index.insert("a", vec![1.0]);
    index.insert("b", vec![1.0]);
    index.keep(["a"]);
    assert!(index.get("a").is_some());
    assert_eq!(index.get("b"), None);
}
