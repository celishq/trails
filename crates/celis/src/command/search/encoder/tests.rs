use super::Encoder;
use crate::command::search::model::Model;

/// Compares each model in `$CELIS_TEST_MODELS/<id>` with the document vectors of
/// `$CELIS_TEST_MODELS/ref.json`; run with `--ignored`.
#[test]
#[ignore]
fn each_real_model_matches_its_reference() {
    let root = std::path::PathBuf::from(std::env::var("CELIS_TEST_MODELS").unwrap());
    let reference: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("ref.json")).unwrap()).unwrap();
    let texts: Vec<String> = serde_json::from_value(reference["texts"].clone()).unwrap();
    for (id, name) in [("embeddinggemma", "gemma"), ("e5-small", "e5")] {
        let model = Model::named(id).unwrap();
        let encoder = Encoder::load(model, &root.join(id)).unwrap();
        for (i, text) in texts.iter().enumerate() {
            let got = encoder.document(text).unwrap();
            let want: Vec<f32> =
                serde_json::from_value(reference["vectors"][name][i].clone()).unwrap();
            let want = &want[..model.dimensions];
            let length = want.iter().map(|x| x * x).sum::<f32>().sqrt();
            let cosine: f32 = got.iter().zip(want).map(|(a, b)| a * b / length).sum();
            assert!(cosine > 0.999, "{id} text {i}: cosine {cosine}");
        }
    }
}
