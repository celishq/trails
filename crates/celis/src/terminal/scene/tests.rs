use super::*;
use crate::terminal::canvas::{line, visible};

/// The text of `rows`, with no codes.
fn text(rows: &[Styled]) -> Vec<String> {
    rows.iter().map(|r| line(r, false)).collect()
}

/// The scene is its art between two blank lines, no row wider than its columns when the width leaves no
/// room.
#[test]
fn the_scene_is_its_art_between_blank_lines() {
    let rows = text(&scene(0, 2 + COLUMNS));
    assert_eq!(rows.len(), ROWS);
    assert_eq!((rows[0].as_str(), rows[ROWS - 1].as_str()), ("", ""));
    assert!(rows.iter().all(|r| visible(r) <= 2 + COLUMNS), "{rows:?}");
    assert!(
        rows.iter().any(|r| r.contains("dead end | | next step")),
        "{rows:?}"
    );
}

/// The walker starts on the ground with its flag, then stands at the summit cheering, its flag planted.
#[test]
fn the_walker_climbs_to_the_summit_and_plants_its_flag() {
    let start = text(&scene(0, 2 + COLUMNS));
    assert!(
        start[13].contains("•_•|>") && start[14].contains("O.|"),
        "{start:?}"
    );
    let end = text(&scene(OPENING.0 + OPENING.1, 2 + COLUMNS));
    assert!(end[4].contains("\\^o^/ |>"), "{end:?}");
    assert!(end[5].contains("O"), "{end:?}");
    assert_eq!(end, text(&scene(OPENING.0 + OPENING.1 + 50, 2 + COLUMNS)));
}

/// A wider scene keeps its art in the middle and fills both sides: peaks, two more graves, unbroken ground
/// and grass, every row as wide as it is given.
#[test]
fn a_wider_scene_fills_both_sides_round_its_art() {
    let rows = text(&scene(OPENING.0 + OPENING.1, 150));
    let narrow = text(&scene(OPENING.0 + OPENING.1, 2 + COLUMNS));
    let left = (148 - COLUMNS) / 2;
    assert!(rows[10].contains("dead end | | next step"), "{rows:?}");
    assert_eq!(
        rows[10].find("dead end"),
        narrow[10].find("dead end").map(|at| at + left)
    );
    assert!(
        rows[5].contains("___/::\\/\\__") && rows[5].contains("_/\\/::\\__"),
        "{rows:?}"
    );
    assert_eq!(
        rows[14].matches("|RIP|").count() + rows[13].matches("|RIP|").count(),
        4,
        "{rows:?}"
    );
    assert!(!rows[14].trim().contains("  "), "{rows:?}");
    assert!(
        rows[1..ROWS - 1].iter().all(|r| visible(r) <= 150),
        "{rows:?}"
    );
}
