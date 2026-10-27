//! celis-patterns — the patterns agents leave for each other on one machine.
//!
//! A [`Pattern`] is what later work must keep. [`PatternStore`] reads them from a store and writes
//! them, signed by the account.

pub mod example;
pub mod examples;
pub mod glob;
pub mod pattern;
pub mod pattern_end;
pub mod pattern_record;
pub mod pattern_store;
pub mod patterns;
pub mod scope;

pub use example::Example;
pub use examples::Examples;
pub use glob::Glob;
pub use pattern::Pattern;
pub use pattern_end::PatternEnd;
pub use pattern_record::PatternRecord;
pub use pattern_store::PatternStore;
pub use patterns::Patterns;
pub use scope::Scope;
