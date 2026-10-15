//! celis-trails — the trails of steps and the decisions agents leave for each other on one machine.
//!
//! A [`step`] is one part of the work, a [`trail`] is one task as its steps, and a [`decision`] is a rule
//! for later work. [`reuse`] says what later work took from them. [`Trails`] writes them, signed by the
//! account, into the [`store`].

pub mod decision;
pub mod failure;
pub mod id;
pub mod period;
pub mod reuse;
pub mod step;
pub mod store;
pub mod trail;
pub mod trails;

pub use decision::decisions::Decisions;
pub use decision::scope::Scope;
pub use decision::Decision;
pub use failure::Failure;
pub use id::decision_id::DecisionId;
pub use id::step_id::StepId;
pub use period::Period;
pub use reuse::Reuse;
pub use step::check::Check;
pub use step::flags::Flags;
pub use step::grade::Grade;
pub use step::mark::Mark;
pub use step::state::State;
pub use step::step_record::StepRecord;
pub use step::steps::Steps;
pub use step::tally::Tally;
pub use step::Step;
pub use store::anchor::Anchor;
pub use store::link::Link;
pub use store::phrase::Phrase;
pub use store::project::Project;
pub use store::Store;
pub use trail::Trail;
pub use trails::draft::Draft;
pub use trails::notice::Notice;
pub use trails::review::Review;
pub use trails::written::Written;
pub use trails::Trails;
