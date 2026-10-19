//! celis-trails — the trails of steps and the rules agents leave for each other on one machine.
//!
//! A [`step`] is one part of the work, a [`trail`] is one task as its steps, and a [`rule`] is what later
//! work must keep. [`Trails`] writes them, signed by the account, into the [`records`] store.

pub mod records;
pub mod refusal;
pub mod rule;
pub mod step;
pub mod trail;
pub mod trails;

pub use records::anchor::Anchor;
pub use records::id::rule_id::RuleId;
pub use records::id::step_id::StepId;
pub use records::link::Link;
pub use records::phrase::Phrase;
pub use records::place::Place;
pub use records::project::Project;
pub use records::store::Store;
pub use refusal::Refusal;
pub use rule::rules::Rules;
pub use rule::scope::Scope;
pub use rule::Rule;
pub use step::check::Check;
pub use step::mark::Mark;
pub use step::name::Name;
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
