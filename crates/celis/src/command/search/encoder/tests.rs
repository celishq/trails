use super::Encoder;
use crate::command::search::model::Model;
use candle_core::{DType, Device};
use candle_nn::{VarBuilder, VarMap};
use candle_transformers::models::bert::{self, BertModel};
use std::path::Path;

/// A BERT model with random weights and a vocabulary of a few words, in `folder`.
fn tiny(folder: &Path) {
    let config = r#"{"vocab_size": 8, "hidden_size": 8, "num_hidden_layers": 1, "num_attention_heads": 2,
        "intermediate_size": 16, "hidden_act": "gelu", "hidden_dropout_prob": 0.0,
        "max_position_embeddings": 64, "type_vocab_size": 2, "initializer_range": 0.02,
        "layer_norm_eps": 1e-12, "pad_token_id": 0, "model_type": "bert"}"#;
    let tokenizer = r#"{"version": "1.0", "pre_tokenizer": {"type": "Whitespace"},
        "model": {"type": "WordLevel", "unk_token": "[UNK]",
        "vocab": {"[UNK]": 0, "passage:": 1, "query:": 2, "keep": 3, "prices": 4, "in": 5, "cents": 6, "rust": 7}}}"#;
    std::fs::write(folder.join("config.json"), config).unwrap();
    std::fs::write(folder.join("tokenizer.json"), tokenizer).unwrap();
    let map = VarMap::new();
    let vb = VarBuilder::from_varmap(&map, DType::F32, &Device::Cpu);
    BertModel::load(vb, &serde_json::from_str::<bert::Config>(config).unwrap()).unwrap();
    map.save(folder.join("model.safetensors")).unwrap();
}

#[test]
fn a_text_gives_the_same_unit_vector_each_time() {
    let folder = tempfile::tempdir().unwrap();
    tiny(folder.path());
    let encoder = Encoder::load(Model::named("e5-small").unwrap(), folder.path()).unwrap();
    let first = encoder.document("keep prices in cents").unwrap();
    let length: f32 = first.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert_eq!(first.len(), 8);
    assert!((length - 1.0).abs() < 1e-5);
    assert_eq!(first, encoder.document("keep prices in cents").unwrap());
    assert_ne!(first, encoder.document("rust").unwrap());
}

/// Compares each model in `$CELIS_TEST_MODELS/<id>` with the document vectors of
/// `$CELIS_TEST_MODELS/ref.json`; run with `--ignored`.
#[test]
#[ignore]
fn each_real_model_matches_its_reference() {
    let root = std::path::PathBuf::from(std::env::var("CELIS_TEST_MODELS").unwrap());
    let reference: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("ref.json")).unwrap()).unwrap();
    let texts: Vec<String> = serde_json::from_value(reference["texts"].clone()).unwrap();
    for (id, name) in [
        ("embeddinggemma", "gemma"),
        ("qwen3", "qwen"),
        ("e5-small", "e5"),
    ] {
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
