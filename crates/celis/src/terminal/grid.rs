//! The grid every board lays out on: twelve tracks across the width with two-cell gutters, so cards,
//! tiles, bars and legends on stats, tree and graph start and end in shared columns.

/// The tracks across a board.
pub(crate) const TRACKS: u16 = 12;

/// The blank cells between two tracks, and between the width and the outer tracks.
pub(crate) const GUTTER: u16 = 2;

/// The grid of a board: the column of its first track and the cells its tracks and inner gutters cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Grid {
    left: u16,
    inner: u16,
}

impl Grid {
    /// The grid of a board `width` wide, a gutter at each side.
    pub fn of(width: u16) -> Grid {
        Grid {
            left: GUTTER,
            inner: width.saturating_sub(2 * GUTTER),
        }
    }

    /// The column where track `k` starts; past the last track, the column after its gutter.
    pub fn edge(&self, k: u16) -> u16 {
        let k = u32::from(k.min(TRACKS));
        let step = u32::from(self.inner + GUTTER) * k / u32::from(TRACKS);
        self.left + step as u16
    }

    /// The column and width of the tracks from `from` up to `to`, with the gutters between them.
    pub fn span(&self, from: u16, to: u16) -> (u16, u16) {
        let x = self.edge(from);
        (x, self.edge(to).saturating_sub(x + GUTTER))
    }

    /// The column after the last track.
    pub fn right(&self) -> u16 {
        self.left + self.inner
    }

    /// The column inside a card where the text of track `k` starts, a border and a space past its edge.
    pub fn column(&self, k: u16) -> u16 {
        self.edge(k) + GUTTER
    }

    /// The first track whose text column is at or after `column`; past the last track when none is.
    pub fn track(&self, column: u16) -> u16 {
        (0..TRACKS)
            .find(|k| self.column(*k) >= column)
            .unwrap_or(TRACKS)
    }

    /// The first track text column at or after `column`; the right edge when none is.
    pub fn after(&self, column: u16) -> u16 {
        match self.track(column) {
            TRACKS => self.right(),
            k => self.column(k),
        }
    }

    /// The text column `tracks` tracks into the card whose border stands at `card`, or the first track
    /// text column at or after `from` when that is later.
    pub fn inside(&self, card: u16, tracks: u16, from: u16) -> u16 {
        let k = (0..TRACKS)
            .find(|k| self.edge(*k) >= card)
            .unwrap_or(TRACKS);
        let least = self.edge(k + tracks) + GUTTER;
        self.after(from.max(least))
    }
}

#[cfg(test)]
mod tests;
