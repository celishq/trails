use super::*;

/// A white screen.
const WHITE: (u8, u8, u8) = (255, 255, 255);

fn fg(style: Style) -> (u8, u8, u8) {
    match style.fg {
        Some(Color::Rgb(r, g, b)) => (r, g, b),
        other => panic!("{other:?}"),
    }
}

fn on_white(colour: (u8, u8, u8)) -> (u8, u8, u8) {
    let (r, g, b) = colour;
    fg(legible(Style::new().fg(Color::Rgb(r, g, b)), WHITE, false))
}

/// On a dark screen every look stays as it is.
#[test]
fn a_dark_screen_keeps_every_look() {
    let looks = [
        Style::new().fg(Color::Rgb(34, 211, 238)),
        Style::new().add_modifier(Modifier::DIM),
        Style::new().bg(Color::White),
    ];
    for look in looks {
        assert_eq!(legible(look, DARK, false), look);
    }
}

/// On a light screen a colour keeps the contrast it has on the dark one, a vivid one at most 5 to 1 so
/// its hue shows: a slate border stays as faint as it was, and an orange reads.
#[test]
fn a_light_screen_keeps_each_colours_contrast() {
    let slate = (60, 90, 130);
    let (dark, light) = (contrast(slate, DARK), contrast(on_white(slate), WHITE));
    assert!((dark - light).abs() < 0.3, "{dark} {light}");
    let orange = on_white((249, 115, 22));
    let read = contrast(orange, WHITE);
    assert!((AA..=VIVID + 0.1).contains(&read), "{orange:?} {read}");
}

/// On a light screen the palette's amber, rose and violet stay vivid, not muddy, and a second share's
/// slate turns a neutral grey that stands apart from working's sky blue.
#[test]
fn a_light_screen_draws_the_palette_clean() {
    for vivid in [(250, 204, 21), (251, 113, 133), (192, 96, 255)] {
        let (r, g, b) = on_white(vivid);
        let spread = r.max(g).max(b) - r.min(g).min(b);
        assert!(spread > 170, "{vivid:?} turned {:?}", (r, g, b));
    }
    let (slate, sky) = (on_white((86, 120, 168)), on_white((34, 211, 238)));
    let grey = slate.2 - slate.0;
    assert!(grey < 50 && sky.2 - sky.0 > 150, "{slate:?} {sky:?}");
}

/// On a light screen done's green and working's cyan turn a green and a sky blue: each above 3 to 1,
/// and the one is green where the other is blue.
#[test]
fn a_light_screen_draws_done_and_working_apart() {
    let (green, blue) = (on_white((52, 230, 140)), on_white((34, 211, 238)));
    for colour in [green, blue] {
        assert!(contrast(colour, WHITE) > 3.0, "{colour:?}");
    }
    assert!(
        green.1 > green.2 + 60 && blue.2 > blue.1 + 50,
        "{green:?} {blue:?}"
    );
}

/// On a light screen what stood out more on the dark one stands out more still: white turns near black,
/// a picked card's light border darker than a card's border, a border faint.
#[test]
fn a_light_screen_keeps_what_stands_out() {
    let white = fg(legible(Style::new().fg(Color::White), WHITE, true));
    assert!(contrast(white, WHITE) > 13.0, "{white:?}");
    let (picked, edge) = (on_white((241, 245, 249)), on_white((44, 66, 104)));
    assert!(contrast(picked, WHITE) > contrast(edge, WHITE) + 5.0);
    assert!(contrast(edge, WHITE) < 2.0, "{edge:?}");
}

/// On a light screen dim text with no colour reads at 4.5 to 1, and a dim bar track stays faint under
/// the cyan beside it.
#[test]
fn a_light_screen_draws_dim_text_readable_and_a_dim_track_faint() {
    let dim = Style::new().add_modifier(Modifier::DIM);
    let text = legible(dim, WHITE, false);
    assert!(!text.add_modifier.contains(Modifier::DIM));
    assert!(contrast(fg(text), WHITE) >= AA - 0.05, "{text:?}");
    let track = fg(legible(dim, WHITE, true));
    let cyan = on_white((34, 211, 238));
    assert!(contrast(track, WHITE) < 2.0, "{track:?}");
    assert!(contrast(cyan, track) > 1.8, "{cyan:?} {track:?}");
}

/// The palette mixes against the screen when dark, and against the dark it is made for when light.
#[test]
fn the_palette_mixes_against_the_dark_on_a_light_screen() {
    assert_eq!(ground(WHITE), DARK);
    assert_eq!(ground((10, 10, 10)), (10, 10, 10));
}

/// On a light screen done, changed and dropped fill a bar with Apple's system green, orange and red,
/// and read in text in a deeper shade of the same hue.
#[test]
fn a_light_screen_fills_bars_bright_and_writes_text_deeper() {
    let fills = [(52, 199, 89), (255, 149, 0), (255, 59, 48)];
    let darks = [(52, 230, 140), (250, 204, 21), (251, 113, 133)];
    for (dark, fill) in darks.into_iter().zip(fills) {
        let (r, g, b) = dark;
        let look = Style::new().fg(Color::Rgb(r, g, b));
        let (bar, text) = (
            fg(legible(look, WHITE, true)),
            fg(legible(look, WHITE, false)),
        );
        assert_eq!(bar, fill);
        assert!(
            contrast(text, WHITE) > contrast(bar, WHITE) + 1.0,
            "{text:?} {bar:?}"
        );
    }
}

/// On a light screen a colour mixed between two of the palette's lies as far between their clean twins:
/// half way from slate to cyan is half way from the clean slate to the sky blue.
#[test]
fn a_light_screen_mixes_between_clean_twins() {
    let (slate, cyan) = ((86, 120, 168), (34, 211, 238));
    let half = |a: u8, b: u8| ((u16::from(a) + u16::from(b)) / 2) as u8;
    let mixed = (
        half(slate.0, cyan.0),
        half(slate.1, cyan.1),
        half(slate.2, cyan.2),
    );
    let (clean_slate, sky) = (on_white(slate), on_white(cyan));
    let expected = (
        half(clean_slate.0, sky.0),
        half(clean_slate.1, sky.1),
        half(clean_slate.2, sky.2),
    );
    let got = on_white(mixed);
    let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
    assert!(
        near(got.0, expected.0) && near(got.1, expected.1) && near(got.2, expected.2),
        "{got:?} {expected:?}"
    );
}
