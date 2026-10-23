//! Finding texts by meaning: the models celis can use, and the index of the texts a model has read.

mod encoder;
mod gemma;
mod index;
mod model;
mod vectors;

pub use index::Index;
pub use model::{Model, MODELS};
