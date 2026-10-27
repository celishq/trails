//! celis-patterns — the patterns agents leave for each other on one machine.
//!
//! A [`Pattern`] is what later work must keep. [`PatternStore`] reads them from a store and writes
//! them, signed by the account. A [`Cell`] groups patterns, and [`CellStore`] writes cells. A [`View`]
//! is the patterns as one project sees them.

pub mod cell;
pub mod example;
pub mod examples;
pub mod glob;
pub mod pattern;
pub mod pattern_end;
pub mod pattern_record;
pub mod pattern_store;
pub mod patterns;
pub mod scope;
pub mod view;

pub use cell::cell_store::CellStore;
pub use cell::cells::Cells;
pub use cell::export::Export;
pub use cell::layer::Layer;
pub use cell::mask::Mask;
pub use cell::membership::Membership;
pub use cell::Cell;
pub use example::Example;
pub use examples::Examples;
pub use glob::Glob;
pub use pattern::Pattern;
pub use pattern_end::PatternEnd;
pub use pattern_record::PatternRecord;
pub use pattern_store::PatternStore;
pub use patterns::Patterns;
pub use scope::Scope;
pub use view::View;
