use super::*;
use crate::command::graph::seen::Seen;
use crate::command::graph::tests::{text, Bench};
use std::collections::HashSet;
use std::time::Instant;

/// A bench with a trail and one step under it, `fast`, with five steps under it: one done, one with two
/// steps under it done, one dropped, one at work, one open; `fast` at work.
fn fast() -> (Bench, String, String) {
    let mut bench = Bench::new();
    let trail = bench.step("shop", None, "the checkout");
    let fast = bench.step("shop", Some(&trail), "Make the checkout fast");
    bench.start(&fast);
    let measure = bench.step("shop", Some(&fast), "Measure each step");
    bench.start(&measure);
    bench.done(&measure, false);
    let index = bench.step("shop", Some(&fast), "Index the order tables");
    bench.start(&index);
    for title in ["Index orders by buyer", "Index lines by order"] {
        let id = bench.step("shop", Some(&index), title);
        bench.start(&id);
        bench.done(&id, false);
    }
    bench.done(&index, true);
    let banner = bench.step("shop", Some(&fast), "Keep the banner");
    bench.drop(&banner);
    let cache = bench.step("shop", Some(&fast), "Cache the options");
    bench.start(&cache);
    bench.step("shop", Some(&fast), "Test it under load");
    (bench, fast, index)
}

/// The rows of the card of `id` in `bench`, `width` wide, opened or not, with `unfolded` open.
fn card(bench: &Bench, id: &str, width: u16, opened: bool, unfolded: &[&str]) -> Vec<String> {
    let steps = bench.trails.steps();
    let step = steps.find(id).unwrap();
    let seen = Seen::settled();
    let unfolded: HashSet<String> = unfolded.iter().map(|s| s.to_string()).collect();
    let m = Moment {
        seen: &seen,
        now: Instant::now(),
        unfolded: &unfolded,
        at: bench.at,
    };
    let height = Card::height(step, opened, width, &m);
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    let card = Card {
        step,
        area: buf.area,
        opened,
        selected: false,
        row: None,
        alpha: 1.0,
    };
    card.draw(&mut buf, &m);
    text(&buf)
}

/// A working card lists the step at work first, then the open one, then the newest closed, four at
/// most, and counts the rest by state, over a bar of its steps done.
#[test]
fn a_working_card_lists_the_work_first() {
    let (bench, fast, _) = fast();
    let rows = card(&bench, &fast, WIDTH, false, &[]);
    assert!(rows[0].starts_with("╭─ ▸ "), "{rows:#?}");
    assert!(rows[1].contains("Make the checkout"), "{rows:#?}");
    assert!(rows[5].contains("Cache the options"), "{rows:#?}");
    assert!(rows[6].contains("○ Test it under"), "{rows:#?}");
    assert!(rows[7].contains("✗ Keep the banner"), "{rows:#?}");
    assert!(rows[8].contains("+2 more"), "{rows:#?}");
    assert!(rows[9].contains("4/7"), "{rows:#?}");
}

/// An opened card lists the steps right under it, each nesting folded under ▸ until the person
/// unfolds it; unfolded, its steps hang from its mark.
#[test]
fn an_opened_card_unfolds_each_nesting_by_itself() {
    let (bench, fast, index) = fast();
    let folded = card(&bench, &fast, 60, true, &[]);
    let row = folded
        .iter()
        .find(|r| r.contains("Index the order tables"))
        .unwrap();
    assert!(
        row.contains("├─ ▸ ✓˜ Index the order tables"),
        "{folded:#?}"
    );
    assert!(
        !folded.iter().any(|r| r.contains("Index orders by buyer")),
        "{folded:#?}"
    );
    let open = card(&bench, &fast, 60, true, &[&index]);
    let nested = open
        .iter()
        .find(|r| r.contains("Index orders by buyer"))
        .unwrap();
    assert!(
        nested.contains("│    ├─── ✓ Index orders by buyer"),
        "{open:#?}"
    );
    assert!(
        open.iter()
            .any(|r| r.contains("2/2 · ") && r.contains("Index the order tables")),
        "{open:#?}"
    );
}

/// A closed card shows its title, and under it its steps by state and its time.
#[test]
fn a_closed_card_shows_its_counts_and_time() {
    let (bench, _, index) = fast();
    let rows = card(&bench, &index, WIDTH, false, &[]);
    assert_eq!(rows.len(), 4, "{rows:#?}");
    assert!(rows[1].contains("Index the order ta…"), "{rows:#?}");
    assert!(
        rows[2].contains("│ ✓ 2") && rows[2].contains("changed · "),
        "{rows:#?}"
    );
}
