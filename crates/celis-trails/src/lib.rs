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
pub mod id;
pub mod mark;
pub mod phrase;
pub mod project;
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
pub use mark::Mark;
pub use phrase::Phrase;
pub use project::Project;
pub use scope::Scope;
pub use state::State;
pub use status::Status;
pub use step::Step;
pub use steps::Steps;
pub use store::Store;
