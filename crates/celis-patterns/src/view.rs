//! CIP-09, "The view": the patterns as one project sees them.

use crate::patterns::find;
use crate::{Cell, Layer, Pattern, Patterns, Scope};
use celis_store::Refusal;

/// The patterns of a store as the project `project` sees them: without the patterns of its cells that
/// are off, and with those of its cells that are on (`spec/09-cells.md`, "The view").
pub struct View<'p, 'a> {
    patterns: &'p Patterns<'a>,
    project: String,
}

impl<'p, 'a> View<'p, 'a> {
    pub fn new(patterns: &'p Patterns<'a>, project: &str) -> View<'p, 'a> {
        View {
            patterns,
            project: project.to_string(),
        }
    }

    /// The live patterns in scope.
    pub fn live(&self) -> Vec<Pattern<'a>> {
        let scoped = self.scoped().into_iter();
        scoped.filter(|p| p.is_live()).collect()
    }

    /// The patterns in scope, live or ended.
    pub fn scoped(&self) -> Vec<Pattern<'a>> {
        let every = self.patterns.every().iter();
        every.filter(|p| self.holds(p)).cloned().collect()
    }

    /// The live pattern in scope whose id is `id`, or the only one whose id starts with `id` when `id` is
    /// 4 or more characters.
    pub fn find(&self, id: &str) -> Result<Pattern<'a>, Refusal> {
        let every = self.patterns.every().iter();
        find(every.filter(|p| p.is_live() && self.holds(p)), id)
    }

    /// The live patterns in scope whose `within` names `id`, newest first.
    pub fn smaller(&self, id: &str) -> Vec<Pattern<'a>> {
        let live = self.live().into_iter();
        live.filter(|p| p.record.within.iter().any(|w| &**w == id))
            .collect()
    }

    /// Whether `pattern` is in the view: one of its layers is on, and it replaces no pattern out of it.
    pub fn shows(&self, pattern: &Pattern) -> bool {
        let on = self
            .layers(pattern)
            .into_iter()
            .any(|l| self.is_layer_on(l));
        let replaced = self.patterns.replaced(pattern);
        on && replaced.is_none_or(|r| self.shows(r))
    }

    /// The layers of `pattern`: the cells that hold it, else the base layer.
    pub fn layers(&self, pattern: &Pattern) -> Vec<Layer<'p>> {
        let cells = self.patterns.holding(pattern);
        match cells.is_empty() {
            true => vec![Layer::Base],
            false => cells.into_iter().map(Layer::Cell).collect(),
        }
    }

    /// The cells that are on and hold `pattern`.
    pub fn cells_on(&self, pattern: &Pattern) -> Vec<&'p Cell> {
        let cells = self.patterns.holding(pattern).into_iter();
        cells.filter(|c| self.is_on(c)).collect()
    }

    /// Whether `cell` is on in the project.
    pub fn is_on(&self, cell: &Cell) -> bool {
        self.patterns.cells().is_on(cell, &self.project)
    }

    fn is_layer_on(&self, layer: Layer) -> bool {
        match layer {
            Layer::Base => true,
            Layer::Cell(cell) => self.is_on(cell),
        }
    }

    /// Whether `pattern` holds in the project: in the view, and in a cell that is on or in its scope.
    fn holds(&self, pattern: &Pattern) -> bool {
        let record = &pattern.record;
        let scoped = record.scope == Scope::Me || record.anchor.project == self.project;
        self.shows(pattern) && (scoped || !self.cells_on(pattern).is_empty())
    }
}
