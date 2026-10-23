//! The models `search` can use.

use std::path::{Path, PathBuf};

/// How a model turns the states of its tokens into one vector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Architecture {
    /// Gemma 3 layers that see every token, mean pooling, then two dense layers.
    Gemma,
    /// BERT layers and mean pooling.
    Bert,
}

/// A model `setup` can download from Hugging Face at a pinned revision, and how `search` uses it.
pub(crate) struct Model {
    pub id: &'static str,
    pub repo: &'static str,
    pub revision: &'static str,
    /// Each file with its sha256.
    pub files: &'static [(&'static str, &'static str)],
    pub architecture: Architecture,
    /// The size its vectors are cut to.
    pub dimensions: usize,
    pub query: &'static str,
    pub document: &'static str,
    pub size: &'static str,
    pub speed: &'static str,
    pub license: &'static str,
    pub recommended: bool,
    pub note: &'static str,
}

/// Every model, the recommended one first.
pub(crate) const MODELS: &[Model] = &[
    Model {
        id: "embeddinggemma",
        repo: "unsloth/embeddinggemma-300m",
        revision: "bfa3c846ac738e62aa61806ef9112d34acb1dc5a",
        files: &[
            (
                "config.json",
                "8f863f76e2d9c710cc833dc92efa898c9adfd41031c786507cc6b0e49c2e3e68",
            ),
            (
                "tokenizer.json",
                "6852f8d561078cc0cebe70ca03c5bfdd0d60a45f9d2e0e1e4cc05b68e9ec329e",
            ),
            (
                "model.safetensors",
                "cbf5a78393b6a033e0b8a63a57549964f7ed5c6fbeb4ba0694214f36123f2fd2",
            ),
            (
                "2_Dense/model.safetensors",
                "c327f2acb00149676ade24a75e11eb6ebbd367f9ee050267ba56829d2979f702",
            ),
            (
                "3_Dense/model.safetensors",
                "ffb6cc5162e11e2ce6bc2367e121ee3bbbc4e82e1ee26826bd7573d4948d81b8",
            ),
        ],
        architecture: Architecture::Gemma,
        dimensions: 256,
        query: "task: search result | query: ",
        document: "title: none | text: ",
        size: "1.2 GB",
        speed: "fast",
        license: "Gemma terms (https://ai.google.dev/gemma/terms)",
        recommended: true,
        note: "the best quality for its size, 100+ languages",
    },
    Model {
        id: "e5-small",
        repo: "intfloat/multilingual-e5-small",
        revision: "614241f622f53c4eeff9890bdc4f31cfecc418b3",
        files: &[
            (
                "config.json",
                "69137736cab8b8903a07fe8afaafdda25aac55415a12a55d1bffa9f581abf959",
            ),
            (
                "tokenizer.json",
                "0b44a9d7b51c3c62626640cda0e2c2f70fdacdc25bbbd68038369d14ebdf4c39",
            ),
            (
                "model.safetensors",
                "1a55775f53449dac10a2bcbc312469fac40b96d53198c407081a831f81c98477",
            ),
        ],
        architecture: Architecture::Bert,
        dimensions: 384,
        query: "query: ",
        document: "passage: ",
        size: "480 MB",
        speed: "fastest",
        license: "MIT",
        recommended: false,
        note: "lower quality",
    },
];

impl Model {
    pub fn named(id: &str) -> Option<&'static Model> {
        MODELS.iter().find(|m| m.id == id)
    }

    /// The model whose folder is in `home`'s models; none when no model was set up.
    pub fn chosen(home: &Path) -> Option<&'static Model> {
        MODELS.iter().find(|m| m.folder(home).is_dir())
    }

    /// The folder in `home` that holds this model's files.
    pub fn folder(&self, home: &Path) -> PathBuf {
        home.join("models").join(self.id)
    }

    /// `<id> <size> <speed> <license>[ recommended] <note>`, the model's line in the list.
    pub fn line(&self) -> String {
        let badge = if self.recommended { " recommended" } else { "" };
        format!(
            "{} {} {} {}{badge} {}",
            self.id, self.size, self.speed, self.license, self.note
        )
    }
}
