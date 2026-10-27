//! A card of a section of the stats board, drawn alone across the width or beside another.

use super::moment::Moment;
use super::sheet::Sheet;

/// A card of a section: its name and what it shows in its top border, the second it fades in at, and
/// what draws inside it from a column and a row, `w` wide, giving the rows the card takes.
pub(super) struct Panel {
    pub name: &'static str,
    pub says: String,
    pub start: f64,
    pub draw: fn(&mut Sheet, &Moment, u16, u16, u16) -> u16,
}

impl Panel {
    /// Draws it at `x`, `y`, `w` wide and at least `tall` rows: its inside, then its border; the rows it
    /// takes.
    pub fn place(&self, s: &mut Sheet, m: &Moment, (x, y, w): (u16, u16, u16), tall: u16) -> u16 {
        s.alpha = m.fade(self.start);
        s.clip = Some((x + 1, x + w - 1));
        let h = (self.draw)(s, m, x, y, w).max(tall);
        s.alpha = m.fade(self.start);
        s.card((x, y, w, h), self.name, &self.says);
        s.clip = None;
        h
    }

    /// The rows it takes `w` wide.
    pub fn height(&self, m: &Moment, w: u16) -> u16 {
        let mut scratch = Sheet::new(m.width, 8);
        (self.draw)(&mut scratch, m, 0, 0, w)
    }
}

/// `panels` side by side from row `y`, six tracks of the grid each, when six tracks hold 44 columns or
/// more, each as tall as the tallest, else one under another across the grid; the row after them.
pub(super) fn pair(s: &mut Sheet, m: &Moment, y: u16, panels: &[Panel]) -> u16 {
    let grid = m.grid();
    let half = grid.span(0, 6).1;
    if half >= 44 {
        let tall = panels.iter().map(|p| p.height(m, half)).max().unwrap_or(0);
        for (n, panel) in panels.iter().enumerate() {
            let (x, w) = grid.span(6 * n as u16, 6 * n as u16 + 6);
            panel.place(s, m, (x, y, w), tall);
        }
        return y + tall + 1;
    }
    let (x, w) = grid.span(0, 12);
    let mut y = y;
    for panel in panels {
        y += panel.place(s, m, (x, y, w), 0) + 1;
    }
    y
}
