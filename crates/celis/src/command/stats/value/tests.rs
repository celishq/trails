use super::super::tests::Desk;
use super::*;
use crate::progress::Model;

/// A store of `shop`: an ask that used and recalled a pattern, then a fix of it; an ask that used none,
/// then a fix of it whose ask replaced that pattern.
fn shop() -> Desk {
    let mut desk = Desk::new();
    let pattern = desk.pattern(4 * 86_400, "shop");
    let used = desk.add(5_000, "shop", None, "the price", &[&pattern], None);
    desk.recall(4_900, &used, &pattern);
    desk.done(4_800, 60, &used);
    let again = desk.add(4_000, "shop", None, "the price again", &[], None);
    let fix = desk.add(
        4_000,
        "shop",
        Some(&again),
        "fix the price",
        &[],
        Some(&used),
    );
    desk.done(3_900, 60, &fix);
    desk.done(3_800, 0, &again);
    let plain = desk.add(3_000, "shop", None, "the tax", &[], None);
    desk.done(2_900, 60, &plain);
    let later = desk.add(2_000, "shop", None, "the tax again", &[], None);
    let mend = desk.add(
        2_000,
        "shop",
        Some(&later),
        "fix the tax",
        &[],
        Some(&plain),
    );
    desk.revise(1_900, "shop", &pattern);
    desk.done(1_800, 60, &mend);
    desk.done(1_700, 0, &later);
    desk
}

/// The value counts the done asks, those that used a pattern, the corrections and the repeat ones,
/// first-pass success with and without a pattern, and the recall funnel.
#[test]
fn the_value_counts_asks_corrections_and_the_funnel() {
    let mut desk = shop();
    desk.trails.at(super::super::tests::noon());
    let value = Model::of(&desk.trails, |m| Value::of(m, Some("shop")));
    assert_eq!((value.asks, value.covered), (4, 1));
    assert_eq!((value.corrections, value.failed, value.missed), (2, 1, 1));
    assert_eq!((value.with, value.without), (Some(0), Some(66)));
    assert_eq!((value.recalled, value.used, value.held), (1, 1, 0));
    assert_eq!((value.reach, value.promoted), (0, 0));
}
