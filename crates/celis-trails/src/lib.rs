//! celis-trails — the trails of steps and the patterns agents leave for each other on one machine.
//!
//! A [`step`] is one part of the work, a [`trail`] is one task as its steps, and a [`pattern`] is what later
//! work must keep. [`Trails`] writes them, signed by the account, into the [`records`] store.

pub mod pattern;
pub mod records;
pub mod refusal;
pub mod step;
pub mod trail;
pub mod trails;

pub use pattern::example::Example;
pub use pattern::examples::Examples;
pub use pattern::fit::Fit;
pub use pattern::glob::Glob;
pub use pattern::pattern_end::PatternEnd;
pub use pattern::pattern_record::PatternRecord;
pub use pattern::patterns::Patterns;
pub use pattern::scope::Scope;
pub use pattern::Pattern;
pub use records::anchor::Anchor;
pub use records::id::pattern_id::PatternId;
pub use records::id::step_id::StepId;
pub use records::link::Link;
pub use records::phrase::Phrase;
pub use records::place::Place;
pub use records::project::Project;
pub use records::store::Store;
pub use refusal::Refusal;
pub use step::by::By;
pub use step::check::Check;
pub use step::done::Done;
pub use step::mark::Mark;
pub use step::notes::Notes;
pub use step::proof_file::ProofFile;
pub use step::state::State;
pub use step::step_record::StepRecord;
pub use step::steps::Steps;
pub use step::used::Used;
pub use step::Step;
pub use trail::Trail;
pub use trails::draft::Draft;
pub use trails::notice::Notice;
pub use trails::written::Written;
pub use trails::Trails;
