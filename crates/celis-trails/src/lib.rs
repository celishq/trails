//! celis-trails — the trails of steps and the decisions agents leave for each other on one machine.
//!
//! A [`Store`] holds the records of every [`Project`], signed by the account, and the [`Command`] line
//! reads and writes them.

pub mod anchor;
pub mod command;
pub mod decision;
pub mod decisions;
pub mod failure;
pub mod grade;
pub mod help;
pub mod hit;
pub mod id;
pub mod mark;
pub mod past;
pub mod phrase;
pub mod project;
pub mod query;
pub mod retrieval_log;
pub mod scope;
pub mod state;
pub mod status;
pub mod step;
pub mod steps;
pub mod store;

pub use anchor::Anchor;
pub use command::Command;
pub use decision::Decision;
pub use decisions::Decisions;
pub use failure::Failure;
pub use grade::Grade;
pub use hit::Hit;
pub use mark::Mark;
pub use past::Past;
pub use phrase::Phrase;
pub use project::Project;
pub use query::Query;
pub use retrieval_log::RetrievalLog;
pub use scope::Scope;
pub use state::State;
pub use status::Status;
pub use step::Step;
pub use steps::Steps;
pub use store::Store;
