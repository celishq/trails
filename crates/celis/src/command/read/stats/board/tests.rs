use super::super::tests::shop;
use super::*;
use crate::terminal::canvas::width;

/// The board of the month `month`, `w` wide and settled, its sections opened as `open`, as text a row
/// a line.
fn board(month: &Month, w: u16, open: [bool; 4]) -> Vec<String> {
    drawn(month, &Total::default(), w, open)
}

/// The board of `month` and `total`, `w` wide and settled, its sections opened as `open`, as text a
/// row a line.
fn drawn(month: &Month, total: &Total, w: u16, open: [bool; 4]) -> Vec<String> {
    let m = Moment {
        month,
        total,
        t: 9.0,
        width: w,
        day: DAYS - 1,
        focus: 0,
        open,
    };
    let (sheet, length, _) = frame(&m);
    let lines = sheet.lines(0, length);
    assert!(lines.iter().all(|l| width(l) <= usize::from(w)));
    lines
        .iter()
        .map(|l| l.to_string().trim_end().to_string())
        .collect()
}

/// At any width from the narrowest to the widest the board draws the four headline tiles and every
/// section, opened or closed, and never asks for a wider screen.
#[test]
fn the_board_draws_every_section_at_any_width() {
    let month = shop().month();
    for w in [32, 36, 44, 60, 80, 108, 120] {
        for open in [[false; 4], [true; 4]] {
            let text = board(&month, w, open).join("\n");
            for name in ["REUSE RATE", "FIRST-PASS", "RESOLVED STEPS", "MEDIAN STEP"] {
                assert!(text.contains(name), "{name} at {w}:\n{text}");
            }
            let sections = match open[0] {
                true => [
                    "▾ Learning",
                    "▾ Quality",
                    "▾ Output and speed",
                    "▾ Resources",
                ],
                false => [
                    "▸ LEARNING",
                    "▸ QUALITY",
                    "▸ OUTPUT AND SPEED",
                    "▸ RESOURCES",
                ],
            };
            for name in sections {
                assert!(text.contains(name), "{name} at {w}:\n{text}");
            }
            assert!(!text.contains("wider"), "{text}");
        }
    }
}

/// The headline tiles stand four a row on a wide board, two on a middling one, and one on a narrow one.
#[test]
fn the_tiles_stand_four_two_or_one_a_row() {
    let month = shop().month();
    let row = |lines: &[String], text: &str| lines.iter().position(|l| l.contains(text));
    for (w, same) in [(108, true), (80, true), (44, false)] {
        let lines = board(&month, w, [false; 4]);
        let tiles = (row(&lines, "REUSE RATE"), row(&lines, "FIRST-PASS"));
        assert_eq!(tiles.0 == tiles.1, same, "{lines:#?}");
    }
    let lines = board(&month, 80, [false; 4]);
    assert_ne!(row(&lines, "REUSE RATE"), row(&lines, "RESOLVED STEPS"));
}

/// Opened, the sections show only what the tiles do not: the reused steps and the patterns, the top patterns
/// by hits, how the resolved steps ended, the weeks and the step times, the daily activity, and each
/// project's time with its rework and drops; no tile number shows again, and prior is said once, in the
/// Overview heading.
#[test]
fn opened_sections_show_their_details() {
    let month = shop().month();
    let text = board(&month, 108, [true; 4]).join("\n");
    for part in [
        "REUSE · past steps reapplied",
        "1 reused step  ▲ 1",
        "PATTERNS · learned patterns applied",
        "1 of 3 steps applied a pattern  ▲ 1",
        "1 pattern kept: 1 applied, 1 new",
        "TOP PATTERNS · most hits",
        "RESOLVED STEPS · how they ended",
        "4 resolved  ▲ 3",
        "2 first pass  ▲ 1",
        "1 reworked  ▲ 1",
        "1 dropped  ▲ 1",
        "0 expired",
        "RESOLVED STEPS · by week",
        "STEP TIME · successful steps, start to done",
        "DAILY ACTIVITY",
        "ALLOCATION · active time",
        "📁 shop",
        "30m active  ▲ 20m",
        "■ productive  6m",
        "■ rework  20m",
        "■ dropped  4m",
    ] {
        assert!(text.contains(part), "{part:?} in\n{text}");
    }
    assert_eq!(text.matches("prior").count(), 1, "{text}");
    for gone in ["follows", "median step time", "of active"] {
        assert!(!text.contains(gone), "{gone:?} in\n{text}");
    }
}

/// The Output sparkline marks a day with no resolved step with a dot on the bottom line, level with the
/// base of the bars.
#[test]
fn the_sparkline_marks_an_empty_day_on_its_baseline() {
    let month = shop().month();
    let lines = board(&month, 108, [false; 4]);
    let output = lines.iter().find(|l| l.contains("under 5m")).unwrap();
    assert!(output.contains("⡀⡀⡀⡀⡀⡀█ a day"), "{output}");
}

/// A Total block of all time stands under the head, each total's name over its number, the patterns
/// kept with how many a step ever used, and the board under it keeps to 14 days against the prior 14
/// with its 28-day chart.
#[test]
fn the_total_block_stands_over_the_14_day_board() {
    let mut desk = shop();
    let month = desk.month();
    let total = Model::of(&desk.trails, Total::of);
    let lines = drawn(&month, &total, 108, [true; 4]);
    let text = lines.join("\n");
    assert!(lines[2].contains("Total  since"), "{text}");
    for name in [
        "resolved steps",
        "reused steps",
        "pattern hits",
        "patterns used",
        "trails",
        "active time",
    ] {
        assert!(lines[4].contains(name), "{name} in\n{text}");
    }
    assert!(lines[5].starts_with("  5 "), "{text}");
    assert!(lines[5].contains("1 of 1"), "{text}");
    for part in [
        "Overview  last 14 days vs prior 14",
        "DAILY ACTIVITY · last 28 days",
        "hits by project",
    ] {
        assert!(text.contains(part), "{part:?} in\n{text}");
    }
}

/// Closed, Resources splits its bar as Allocation splits the time: productive green, rework amber and
/// dropped rose, and no colour a project.
#[test]
fn the_closed_resources_bar_splits_the_time_as_allocation() {
    use crate::terminal::canvas::{GREEN, RED, SLATE, YELLOW};
    let month = shop().month();
    let m = Moment {
        month: &month,
        total: &Total::default(),
        t: 9.0,
        width: 108,
        day: DAYS - 1,
        focus: 0,
        open: [false; 4],
    };
    let (sheet, length, _) = frame(&m);
    let lines = sheet.lines(0, length);
    let at = lines
        .iter()
        .position(|l| l.to_string().contains("▸ RESOURCES"))
        .unwrap();
    let block = |c: char| ('\u{2580}'..='\u{259F}').contains(&c);
    let looks: Vec<_> = lines[at + 1..at + 5]
        .iter()
        .flat_map(|l| &l.spans)
        .filter(|s| s.content.chars().any(block))
        .map(|s| s.style.fg)
        .collect();
    for look in [GREEN, YELLOW, RED] {
        assert!(looks.contains(&look.fg), "{looks:?}");
    }
    assert!(!looks.contains(&SLATE.fg), "{looks:?}");
}

/// Closed, Output draws its days in first pass green, as Quality draws first pass.
#[test]
fn the_closed_output_sparkline_is_first_pass_green() {
    use crate::terminal::canvas::GREEN;
    let month = shop().month();
    let m = Moment {
        month: &month,
        total: &Total::default(),
        t: 9.0,
        width: 108,
        day: DAYS - 1,
        focus: 0,
        open: [false; 4],
    };
    let (sheet, length, _) = frame(&m);
    let lines = sheet.lines(0, length);
    let output = lines
        .iter()
        .find(|l| l.to_string().contains("under 5m"))
        .unwrap();
    let bars: Vec<_> = output
        .spans
        .iter()
        .filter(|s| s.content.contains('█'))
        .map(|s| s.style.fg)
        .collect();
    assert!(
        !bars.is_empty() && bars.iter().all(|fg| *fg == GREEN.fg),
        "{bars:?}"
    );
}

/// Opened, Resolved steps by week draws all four weeks' bars in one colour, none grey, and every week's
/// label in the text's own look, none bold or dim.
#[test]
fn every_weeks_bar_is_one_colour() {
    use crate::terminal::canvas::CYAN;
    let month = shop().month();
    let m = Moment {
        month: &month,
        total: &Total::default(),
        t: 9.0,
        width: 108,
        day: DAYS - 1,
        focus: 0,
        open: [false, false, true, false],
    };
    let (sheet, length, _) = frame(&m);
    let lines = sheet.lines(0, length);
    let at = lines
        .iter()
        .position(|l| {
            let text = l.to_string().to_lowercase();
            text.contains("resolved steps") && text.contains("by week")
        })
        .unwrap();
    let block = |c: char| ('\u{2580}'..='\u{259F}').contains(&c);
    let bars: Vec<_> = lines[at + 2..at + 6]
        .iter()
        .flat_map(|l| &l.spans)
        .filter(|s| s.content.chars().any(block))
        .map(|s| s.style)
        .collect();
    assert!(bars.len() >= 4, "{bars:?}");
    assert!(bars.iter().all(|look| look.fg == CYAN.fg), "{bars:?}");
    let labels: Vec<_> = lines[at + 2..at + 6]
        .iter()
        .flat_map(|l| &l.spans)
        .filter(|s| s.content.contains(" – "))
        .map(|s| (s.content.to_string(), s.style.add_modifier))
        .collect();
    assert_eq!(labels.len(), 4, "{labels:?}");
    assert!(labels.iter().all(|(_, m)| m.is_empty()), "{labels:?}");
}

/// The display column of each character of `line`.
fn columns(line: &str) -> Vec<(usize, char)> {
    let mut at = 0;
    line.chars()
        .map(|c| {
            let here = at;
            at += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            (here, c)
        })
        .collect()
}

/// The board lays out on one grid at any width: every card and tile ends in one column, the bars of the
/// cards with labels start in one column, a track's text column, and every legend square stands on a
/// track's text column.
#[test]
fn the_board_keeps_to_one_grid() {
    let month = shop().month();
    for w in [60, 80, 108, 119] {
        let grid = crate::terminal::grid::Grid::of(w);
        let tracks: Vec<usize> = (0..12).map(|k| usize::from(grid.column(k))).collect();
        let lines = board(&month, w, [true; 4]);
        let ends: std::collections::BTreeSet<usize> = lines
            .iter()
            .filter_map(|l| {
                columns(l)
                    .iter()
                    .rev()
                    .find(|(_, c)| *c == '╮')
                    .map(|(at, _)| *at)
            })
            .collect();
        assert_eq!(ends.len(), 1, "{w}: {ends:?}");
        let bar = |text: &str| {
            let line = lines.iter().rev().find(|l| l.contains(text));
            let line = line.unwrap_or_else(|| panic!("{w}: no {text:?} in {lines:#?}"));
            columns(line)
                .into_iter()
                .find(|(_, c)| "█▏▎▍▌▋▊▉".contains(*c))
                .map(|(at, _)| at)
                .unwrap()
        };
        let starts = [bar("this week"), bar(" – "), bar("📁 shop")];
        assert!(starts.iter().all(|s| *s == starts[0]), "{w}: {starts:?}");
        assert!(tracks.contains(&starts[0]), "{w}: {starts:?} {tracks:?}");
        for line in lines
            .iter()
            .filter(|l| l.contains('■') && !l.contains("■ ■") && !l.contains(" · "))
        {
            for (at, c) in columns(line) {
                if c == '■' {
                    assert!(tracks.contains(&at), "{w}: {line:?} {tracks:?}");
                }
            }
        }
    }
}
