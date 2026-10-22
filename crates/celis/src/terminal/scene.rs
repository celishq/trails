//! The trail scene above a live tree: peaks under faint stars, the dead-end and next-step signs, graves,
//! a green hill with its path, and a walker who climbs to the summit and plants a flag by its own.

use super::canvas::{Styled, BOLD, CYAN, RED};
use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// The scene, a row a line: its range of peaks, the dead-end and next-step signs, graves, and the green
/// hill with two tiny peaks beside it.
const ART: [&str; 15] = [
    r"   ·               /\         ✦       ·                                     ·",
    r"          /\      /::\         /\        /\",
    r"         /::\    /::::\       /::\      /::\",
    r"        /::::\  /::::::\     /::::\    /::::\     /\          |>",
    r"  _____/::::::\/::::::::\___/::::::\__/::::::\___/::\/\__     |",
    r"                                                            .-+--------+-.",
    r"         ______   ______                                   /++++++++++++++\",
    r"        /  x   | |  |>  \                                 /++++++++++++++++\",
    r"        \______| |______/                                /++++++++++++++++++\",
    r"      dead end | | next step                           .-'++++++++++++++++++'-.",
    r"               |/|/                                   /++++++++++++++++++++++++\",
    r"   x_x                                                         ////",
    r"  |RIP|  x_x                                                  ////",
    r" .......|RIP|................................................////..............",
    r" ,  '  ,  '  ,  '  ,  '  ,  '  ,  '  ,  '  ,  '  ,  '  ,  '  ,  '  ,  '  ,  '",
];

/// The rows of the art's sky, its peaks in them.
const PEAKS: usize = 5;

/// The columns of the art's own range of peaks, which the scene repeats left of it, mirrored right of it.
const RANGE: usize = 46;

/// The columns of the art's sky the scene keeps: up to the green hill's right edge.
const SKY: usize = 73;

/// The columns between the green hill's right edge and the first peak of the mirrored range.
const GAP: usize = 14;

/// The tiny peaks right of the green hill: their row, their column out from its right edge, and glyph.
const SMALL: [(usize, usize, char); 8] = [
    (3, 9, '/'),
    (3, 10, '\\'),
    (4, 6, '/'),
    (4, 7, '\\'),
    (4, 8, '/'),
    (4, 9, ':'),
    (4, 10, ':'),
    (4, 11, '\\'),
];

/// The graves left of the art in a wide scene: their column back from the art and the row of their face.
const GRAVES: [(usize, usize); 2] = [(14, 12), (31, 11)];

/// The rhythm of the grass.
const GRASS: &str = "  ,  '";

/// The rows the scene takes above a tree: a blank line, its own, and a blank line.
pub(crate) const ROWS: usize = ART.len() + 2;

/// The cells across the scene's art, with the room a walker's flag takes past it: the least width it
/// is drawn in.
pub(crate) const COLUMNS: usize = 84;

/// The column of the ground where the walker starts, and the last of the ground before the hill.
const GROUND: (usize, usize) = (48, 60);

/// The points of the hill's path after the ground, column and row, up to the summit.
const CLIMB: [(usize, usize); 10] = [
    (61, 13),
    (62, 12),
    (63, 11),
    (64, 10),
    (65, 9),
    (65, 8),
    (66, 7),
    (66, 6),
    (67, 5),
    (67, 4),
];

/// Where the walker plants the flag: its pole's column and the row of its cloth.
const PLANTED: (usize, usize) = (71, 3);

/// The ticks the walker climbs, and the ticks it cheers at the summit before it stands still.
const OPENING: (usize, usize) = (16, 12);

/// The ticks the scene's opening plays: the wait before the walker sets off, the climb and the cheer.
pub(crate) const PLAYS: usize = SETTING + OPENING.0 + OPENING.1;

/// The ticks the walker waits while the scene fades in, before it sets off.
const SETTING: usize = 3;

/// The seconds the scene takes to fade in.
pub(crate) const FADE: f64 = 0.5;

/// Where the walker's body stands at the `n`th point of its path: along the ground, then up the hill.
fn point(n: usize) -> (usize, usize) {
    let ground = GROUND.1 - GROUND.0 + 1;
    match n.checked_sub(ground) {
        None => (GROUND.0 + n, 13),
        Some(up) => CLIMB[up.min(CLIMB.len() - 1)],
    }
}

/// The scene `ticks` after the tree opened, `width` wide: the walker waits `SETTING` ticks, climbs its
/// path, cheers at the summit with its flag planted, then stands there cheering; its art in the middle,
/// more peaks, graves, ground and grass on both sides when the width leaves room; a blank line before
/// and after.
pub(crate) fn scene(ticks: usize, width: usize) -> Vec<Styled> {
    let ticks = ticks.saturating_sub(SETTING);
    let last = GROUND.1 - GROUND.0 + 1 + CLIMB.len() - 1;
    let n = (ticks * last / OPENING.0).min(last);
    let cheer = ticks >= OPENING.0 && (ticks >= OPENING.0 + OPENING.1 || ticks % 4 < 2);
    let mut cells: Vec<Vec<char>> = ART
        .iter()
        .map(|row| {
            let mut row: Vec<char> = row.chars().collect();
            row.resize(COLUMNS, ' ');
            row
        })
        .collect();
    let mut painted = vec![];
    let mut put =
        |cells: &mut Vec<Vec<char>>, (column, row): (usize, usize), glyph: char, look: Style| {
            cells[row][column] = glyph;
            painted.push(((column, row), look));
        };
    let (x, y) = point(n);
    let walker = Style::new().patch(BOLD);
    let flag = CYAN.patch(BOLD);
    if n == last {
        put(&mut cells, PLANTED, '|', flag);
        put(&mut cells, (PLANTED.0 + 1, PLANTED.1), '>', flag);
        put(&mut cells, (PLANTED.0, PLANTED.1 + 1), '|', flag);
    }
    if cheer {
        for (i, glyph) in "\\^o^/".chars().enumerate() {
            put(&mut cells, (x + i - 2, y - 1), glyph, walker);
        }
    } else {
        for (i, glyph) in "•_•".chars().enumerate() {
            put(&mut cells, (x + i - 1, y - 1), glyph, walker);
        }
        if n < last {
            put(&mut cells, (x + 2, y - 1), '|', flag);
            put(&mut cells, (x + 3, y - 1), '>', flag);
            put(&mut cells, (x + 2, y), '|', flag);
        }
    }
    put(&mut cells, (x, y), 'O', walker);
    let across = width.saturating_sub(2).max(COLUMNS);
    let left = (across - COLUMNS) / 2;
    let mut rows = vec![Styled::default()];
    for (row, line) in cells.iter().enumerate() {
        let mut drawn: Vec<(char, Style)> = (0..across).map(|x| fill(row, x, left)).collect();
        let end = if row < PEAKS { SKY } else { COLUMNS };
        let lead = line.iter().take_while(|&&c| c == ' ').count();
        let trail = line.iter().rposition(|&c| c != ' ').map_or(0, |p| p + 1);
        let ground = row + 2 >= ART.len();
        for (column, &glyph) in line.iter().enumerate().take(end) {
            let at = left + column;
            if ground && (column < lead || column >= trail) {
                continue;
            }
            if row == PEAKS - 1 && column < lead && left > 0 {
                drawn[at] = ('_', look(row, 0, '_'));
                continue;
            }
            let mine = painted.iter().rev().find(|(at, _)| *at == (column, row));
            drawn[at] = (
                glyph,
                mine.map_or_else(|| look(row, column, glyph), |(_, look)| *look),
            );
        }
        let end = drawn
            .iter()
            .rposition(|(c, _)| *c != ' ')
            .map_or(0, |p| p + 1);
        let mut line = Styled::from(Span::raw("  "));
        line.spans.extend(
            drawn[..end]
                .iter()
                .map(|(glyph, look)| Span::styled(glyph.to_string(), *look)),
        );
        rows.push(line);
    }
    rows.push(Styled::default());
    rows
}

/// What the scene's `row` shows at column `x` off its art, which starts at `left`: the art's range of
/// peaks repeated to the left and mirrored to the right past a gap with two tiny peaks, graves to the
/// left, dots on the ground and tufts in the grass; else a blank.
fn fill(row: usize, x: usize, left: usize) -> (char, Style) {
    let back = |by: usize| left.checked_sub(by);
    for (by, face) in GRAVES {
        let Some(at) = back(by) else { continue };
        if (face..=face + 1).contains(&row) && (at..at + 5).contains(&x) {
            let shape = if row == face { " x_x " } else { "|RIP|" };
            let glyph = shape.chars().nth(x - at).unwrap_or(' ');
            if glyph != ' ' || row + 2 < ART.len() {
                return (glyph, look(row, 0, glyph));
            }
        }
    }
    let glyph = match row {
        r if r < PEAKS && x < left => peak(r, (x + RANGE - left % RANGE) % RANGE, false),
        r if r < PEAKS && x >= left + SKY => {
            let out = x - left - SKY;
            match SMALL.iter().find(|(r, o, _)| *r == row && *o == out) {
                Some((.., glyph)) => *glyph,
                None if row == PEAKS - 1 && (5..GAP).contains(&out) => '_',
                None if out < GAP => ' ',
                None => peak(row, (out - GAP) % RANGE, true),
            }
        }
        r if r + 2 == ART.len() => '.',
        r if r + 1 == ART.len() => {
            let phase = (x + GRASS.len() + 1 - left % GRASS.len()) % GRASS.len();
            GRASS.chars().nth(phase).unwrap_or(' ')
        }
        _ => ' ',
    };
    let column = if row + 2 >= ART.len() { 20 } else { 0 };
    (glyph, look(row, column, glyph))
}

/// The glyph at `column` of the art's range in `row`, its base joined at the edges; mirrored when
/// `mirrored`.
fn peak(row: usize, column: usize, mirrored: bool) -> char {
    let column = if mirrored { RANGE - 1 - column } else { column };
    let glyph = ART[row].chars().nth(column).unwrap_or(' ');
    let glyph = match (row, glyph) {
        (r, ' ') if r == PEAKS - 1 && column < 2 => '_',
        (_, glyph) => glyph,
    };
    match (mirrored, glyph) {
        (true, '/') => '\\',
        (true, '\\') => '/',
        (_, glyph) => glyph,
    }
}

/// The look of `glyph` at `row` and `column`, by what it belongs to: the sky, a peak and its slopes
/// receding in indigo, the summit's flag, the neon hill and its path, the signs, the graves in a quiet
/// slate, or the ground.
fn look(row: usize, column: usize, glyph: char) -> Style {
    let rgb = |(r, g, b): (u8, u8, u8)| Style::new().fg(Color::Rgb(r, g, b));
    let bold = |style: Style| style.patch(BOLD);
    match (row, column, glyph) {
        (_, _, ' ') => Style::new(),
        (_, _, '·' | '✦') => rgb((60, 72, 120)),
        (0..=1, 0..60, '/' | '\\') => rgb((129, 140, 200)),
        (0..=4, 0..60, ':') => rgb(slope(row)),
        (0..=4, 0..60, '_') => rgb((35, 38, 90)),
        (0..=4, 0..60, _) => rgb((70, 76, 150)),
        (3..=4, 62..=63, _) => bold(CYAN),
        (6..=10, 54.., '+') if (row + column) % 2 == 1 => rgb((52, 211, 153)),
        (6..=10, 54.., '+') => rgb((5, 150, 105)),
        (5..=10, 54.., _) => rgb((45, 212, 191)),
        (11.., 58.., '/') => rgb((110, 231, 183)),
        (7, 0..30, 'x') => bold(RED),
        (7, 0..30, '>') | (7, 20, '|') => bold(CYAN),
        (9, 0..14, _) => bold(RED),
        (9, 18..30, _) => bold(CYAN),
        (6..=10, 0..30, _) => rgb((217, 160, 90)),
        (11..=13, 0..14, _) => rgb((100, 116, 139)),
        (14, _, _) => rgb((6, 95, 70)),
        (13, _, _) => rgb((51, 65, 85)),
        _ => rgb((203, 213, 225)),
    }
}

/// The colour of a slope at `row`: lighter near the peak, darker toward the ground.
fn slope(row: usize) -> (u8, u8, u8) {
    let (top, low) = ((79.0, 86.0, 160.0), (35.0, 38.0, 90.0));
    let k = (row.saturating_sub(1) as f64 / 3.0).min(1.0);
    let mix = |a: f64, b: f64| (a + (b - a) * k).round() as u8;
    (mix(top.0, low.0), mix(top.1, low.1), mix(top.2, low.2))
}

#[cfg(test)]
mod tests;
