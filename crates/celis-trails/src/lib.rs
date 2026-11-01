//! celis-trails — the trails of steps that agents take and close on one machine.
//!
//! A [`step`] is one part of the work, and a [`trail`] is one task as its steps. [`TrailStore`] reads
//! them from a store and writes them, signed by the account.

pub mod step;
pub mod trail;
pub mod trail_store;

pub use step::by::By;
pub use step::check::Check;
pub use step::done::Done;
pub use step::mark::Mark;
pub use step::notes::Notes;
pub use step::state::State;
pub use step::step_record::StepRecord;
pub use step::steps::Steps;
pub use step::Step;
pub use trail::Trail;
pub use trail_store::draft::Draft;
pub use trail_store::notice::Notice;
pub use trail_store::written::Written;
pub use trail_store::TrailStore;
