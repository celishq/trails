//! celis-trails — the trails of steps and the decisions agents leave for each other on one machine.
//!
//! A [`step`] is one part of the work, a [`trail`] is one task as its steps, and a [`decision`] is a rule
//! for later work. [`Trails`] writes them, signed by the account, into the [`records`] store.

pub mod decision;
pub mod records;
pub mod refusal;
pub mod step;
pub mod trail;
pub mod trails;

pub use decision::decisions::Decisions;
pub use decision::scope::Scope;
pub use decision::Decision;
pub use records::anchor::Anchor;
pub use records::id::decision_id::DecisionId;
pub use records::id::step_id::StepId;
pub use records::link::Link;
pub use records::phrase::Phrase;
pub use records::place::Place;
pub use records::project::Project;
pub use records::store::Store;
pub use refusal::Refusal;
pub use step::check::Check;
pub use step::mark::Mark;
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
