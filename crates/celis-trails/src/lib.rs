//! celis-trails — the trails of steps and the decisions agents leave for each other on one machine.
//!
//! [`trail`] says what a trail is and [`decision`] what a decision is. [`Trails`] writes them, signed by
//! the account, into the [`store`].

pub mod decision;
pub mod failure;
pub mod id;
pub mod store;
pub mod trail;
pub mod trails;

pub use decision::decisions::Decisions;
pub use decision::scope::Scope;
pub use decision::Decision;
pub use failure::Failure;
pub use store::anchor::Anchor;
pub use store::phrase::Phrase;
pub use store::project::Project;
pub use store::retrieval_log::RetrievalLog;
pub use store::Store;
pub use trail::check::Check;
pub use trail::grade::Grade;
pub use trail::mark::Mark;
pub use trail::past::Past;
pub use trail::state::State;
pub use trail::status::Status;
pub use trail::step::Step;
pub use trail::steps::Steps;
pub use trails::direction::Direction;
pub use trails::draft::Draft;
pub use trails::notice::Notice;
pub use trails::retro::Retro;
pub use trails::written::Written;
pub use trails::Trails;
