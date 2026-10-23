//! Colours on a light terminal background: each colour of the dark palette given the contrast against
//! the light screen that it has against the dark one, its hue kept.

use ratatui::style::{Color, Modifier, Style};

/// A colour's red, green and blue.
pub(crate) type Rgb = (u8, u8, u8);

/// The dark background the palette is made for.
pub(crate) const DARK: Rgb = (35, 38, 49);

/// The contrast dim text keeps on a light screen.
const AA: f64 = 4.5;

/// The contrast a dim line or bar track keeps on a light screen.
const FAINT: f64 = 1.6;

/// The most contrast a vivid colour takes on a light screen, so its hue still shows.
const VIVID: f64 = 5.0;

/// The clean colours a light screen draws in place of the dark palette's vivid ones, which darkened
/// would turn muddy: each dark colour, its text and its fill in bars and blocks. Done, changed and
/// dropped are Apple's system green, orange and red, their text a deeper shade of the same hue; working
/// sky blue, rules violet, and a second share, such as a day of new steps, a light grey.
const CLEAN: [(Rgb, Rgb, Rgb); 6] = [
    ((52, 230, 140), (36, 138, 61), (52, 199, 89)),
    ((34, 211, 238), (10, 149, 216), (10, 149, 216)),
    ((250, 204, 21), (179, 92, 0), (255, 149, 0)),
    ((251, 113, 133), (215, 0, 21), (255, 59, 48)),
    ((192, 96, 255), (124, 58, 237), (124, 58, 237)),
    ((86, 120, 168), (208, 215, 222), (208, 215, 222)),
];

/// The grey of dim text with no colour of its own.
const GREY: Rgb = (127, 130, 139);

/// Whether `screen`, a background colour, is light.
pub(crate) fn light(screen: Rgb) -> bool {
    luminance(screen) > 0.4
}

/// The background the palette's colours are mixed against on `screen`: the screen when dark, else
/// `DARK`.
pub(crate) fn ground(screen: Rgb) -> Rgb {
    match light(screen) {
        true => DARK,
        false => screen,
    }
}

/// `style` as it reads on `screen`: on a light one, each colour given its contrast against `DARK` or its
/// `CLEAN` text, or its fill in a `graphic`, dim colours half way to it first, dim text with no colour a
/// grey at `AA` and a dim `graphic` with none faint; on a dark one, as it is.
pub(crate) fn legible(style: Style, screen: Rgb, graphic: bool) -> Style {
    if !light(screen) {
        return style;
    }
    let mut look = style;
    if let Some(bg) = rgb(style.bg) {
        look = look.bg(colour(twin(bg, screen, true)));
    }
    let dim = style.add_modifier.contains(Modifier::DIM);
    let fg = match (rgb(style.fg), dim, graphic) {
        (Some(fg), true, _) => Some(twin(half(fg, DARK), screen, graphic)),
        (Some(fg), false, _) => Some(twin(fg, screen, graphic)),
        (None, true, true) => Some(toward(GREY, screen, FAINT)),
        (None, true, false) => Some(toward(GREY, screen, AA)),
        (None, false, _) => None,
    };
    if let Some(fg) = fg {
        look = look.fg(colour(fg));
        look.add_modifier -= Modifier::DIM;
    }
    look
}

/// The colour of `colour`'s hue with the contrast against the light `screen` that `colour` has
/// against `DARK`, at most `VIVID` when it is vivid; its text in `CLEAN` when it has one, its fill when
/// a `graphic`.
pub(crate) fn twin(colour: Rgb, screen: Rgb, graphic: bool) -> Rgb {
    let clean = |(_, text, fill): &(Rgb, Rgb, Rgb)| if graphic { *fill } else { *text };
    if let Some(found) = CLEAN.iter().find(|(dark, _, _)| *dark == colour) {
        return clean(found);
    }
    if let Some(mixed) = between(colour, clean) {
        return mixed;
    }
    let (r, g, b) = colour;
    let vivid = r.max(g).max(b) - r.min(g).min(b) > 60;
    let target = contrast(colour, DARK);
    toward(
        colour,
        screen,
        if vivid { target.min(VIVID) } else { target },
    )
}

/// The colour as far between two colours' `clean` twins as `colour` is between those two colours of
/// `CLEAN`; none when it lies between no two of them.
fn between(colour: Rgb, clean: impl Fn(&(Rgb, Rgb, Rgb)) -> Rgb) -> Option<Rgb> {
    let lane = |a: Rgb, b: Rgb| {
        let (from, to) = (rgb_f(a), rgb_f(b));
        let at = rgb_f(colour);
        let way: Vec<f64> = (0..3).map(|i| to[i] - from[i]).collect();
        let length: f64 = way.iter().map(|w| w * w).sum();
        let k = (0..3).map(|i| (at[i] - from[i]) * way[i]).sum::<f64>() / length;
        let off = (0..3).map(|i| (from[i] + way[i] * k - at[i]).abs());
        (k > 0.0 && k < 1.0 && off.fold(0.0, f64::max) <= 2.0).then_some(k)
    };
    CLEAN.iter().enumerate().find_map(|(n, a)| {
        CLEAN[n + 1..].iter().find_map(|b| {
            let k = lane(a.0, b.0)?;
            let (clean_a, clean_b) = (clean(a), clean(b));
            let part =
                |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * k).round() as u8;
            Some((
                part(clean_a.0, clean_b.0),
                part(clean_a.1, clean_b.1),
                part(clean_a.2, clean_b.2),
            ))
        })
    })
}

/// `colour`'s parts as numbers.
fn rgb_f((r, g, b): Rgb) -> [f64; 3] {
    [f64::from(r), f64::from(g), f64::from(b)]
}

/// `colour` mixed toward black or white, its hue kept, until its contrast against the light `screen`
/// is `target`; black when no mix is dark enough.
fn toward(colour: Rgb, screen: Rgb, target: f64) -> Rgb {
    let wanted = (luminance(screen) + 0.05) / target - 0.05;
    let mixed = |k: f64| {
        let part = |c: u8| match k < 0.0 {
            true => f64::from(c) * (1.0 + k),
            false => f64::from(c) + (255.0 - f64::from(c)) * k,
        };
        let part = |c: u8| part(c).round() as u8;
        (part(colour.0), part(colour.1), part(colour.2))
    };
    let (mut low, mut high) = (-1.0, 1.0);
    for _ in 0..30 {
        let k = (low + high) / 2.0;
        match luminance(mixed(k)) < wanted {
            true => low = k,
            false => high = k,
        }
    }
    mixed(low)
}

/// The contrast of `a` against `b`, from 1 to 21.
pub(crate) fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// The relative luminance of `colour`, from 0 to 1.
fn luminance((r, g, b): Rgb) -> f64 {
    let linear = |c: u8| {
        let c = f64::from(c) / 255.0;
        match c <= 0.04045 {
            true => c / 12.92,
            false => ((c + 0.055) / 1.055).powf(2.4),
        }
    };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

/// The colour half way from `a` to `b`.
fn half(a: Rgb, b: Rgb) -> Rgb {
    let mid = |x: u8, y: u8| ((u16::from(x) + u16::from(y)) / 2) as u8;
    (mid(a.0, b.0), mid(a.1, b.1), mid(a.2, b.2))
}

/// `(r, g, b)` as a true colour.
fn colour((r, g, b): Rgb) -> Color {
    Color::Rgb(r, g, b)
}

/// The true colour of `colour`, the named ones as a terminal's default palette shows them; none for the
/// terminal's own.
fn rgb(colour: Option<Color>) -> Option<Rgb> {
    let named = match colour? {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Reset | Color::Indexed(_) => return None,
        Color::Black => (0, 0, 0),
        Color::Red => (205, 49, 49),
        Color::Green => (13, 188, 121),
        Color::Yellow => (229, 229, 16),
        Color::Blue => (36, 114, 200),
        Color::Magenta => (188, 63, 188),
        Color::Cyan => (17, 168, 205),
        Color::Gray => (229, 229, 229),
        Color::DarkGray => (102, 102, 102),
        Color::LightRed => (241, 76, 76),
        Color::LightGreen => (35, 209, 139),
        Color::LightYellow => (245, 245, 67),
        Color::LightBlue => (59, 142, 234),
        Color::LightMagenta => (214, 112, 214),
        Color::LightCyan => (41, 184, 219),
        Color::White => (244, 245, 247),
    };
    Some(named)
}

#[cfg(test)]
mod tests;
