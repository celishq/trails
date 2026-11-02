use super::super::tests::{noon, Desk};
use super::*;
use crate::progress::Model;

/// A store of `shop` today: an ask whose step names a pattern, an ask that names it itself, an ask with
/// a dropped step that names it twice, an ask that names none, an open ask that names it, and an ask
/// done 20 days ago that names it.
fn shop() -> Desk {
    let mut desk = Desk::new();
    let day = 86_400;
    let pattern = desk.pattern(30 * day, "shop");
    let old = desk.add(20 * day, "shop", None, "the old ask", &[&pattern], None);
    desk.done(20 * day - 60, 60, &old);
    let led = desk.add(5_000, "shop", None, "the price", &[], None);
    let step = desk.add(5_000, "shop", Some(&led), "round it", &[&pattern], None);
    desk.done(4_900, 60, &step);
    desk.done(4_800, 0, &led);
    let named = desk.add(4_000, "shop", None, "the tax", &[&pattern], None);
    let part = desk.add(4_000, "shop", Some(&named), "add it", &[], None);
    desk.done(3_900, 60, &part);
    desk.done(3_800, 0, &named);
    let tried = desk.add(3_000, "shop", None, "the label", &[], None);
    let lost = desk.add(
        3_000,
        "shop",
        Some(&tried),
        "print it",
        &[&pattern, &pattern],
        None,
    );
    desk.drop(2_900, 60, &lost);
    let kept = desk.add(2_900, "shop", Some(&tried), "write it", &[], None);
    desk.done(2_800, 60, &kept);
    desk.done(2_700, 0, &tried);
    let plain = desk.add(2_000, "shop", None, "the till", &[], None);
    desk.done(1_900, 60, &plain);
    desk.add(1_000, "shop", None, "the shelf", &[&pattern], None);
    desk
}

/// The reuse of a span counts its done asks, those whose tree names a pattern, the ask included, and
/// each pattern each step of those trees names, whatever its end.
#[test]
fn reuse_counts_done_asks_whose_tree_names_a_pattern() {
    let mut desk = shop();
    desk.trails.at(noon());
    let reuse = Model::of(&desk.trails, |m| {
        Reuse::between(m.steps, noon() - 86_400, noon() + 1)
    });
    assert_eq!((reuse.asks, reuse.reused, reuse.count()), (4, 3, 3));
    assert_eq!(reuse.percent(), Some(75));
    assert_eq!(reuse.top().len(), 1);
}
