//! CIP-09, "The view": a layer of patterns.

use crate::Cell;

/// A layer of patterns: a cell, or the base layer of the patterns in no cell, which is always on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer<'c> {
    Base,
    Cell(&'c Cell),
}
