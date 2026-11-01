use super::super::tests::shop;
use super::*;

/// The total counts every trail and ended step, the done steps that reused past work and that applied
/// a pattern, the patterns kept and how many a step ever used, and the time of every step with none under it,
/// from the first step on.
#[test]
fn the_total_counts_all_time() {
    let mut desk = shop();
    desk.trails.at(super::super::tests::noon());
    let total = Model::of(&desk.trails, Total::of);
    let first = super::super::tests::noon() - 20 * 86_400;
    assert_eq!(total.since, first);
    assert_eq!((total.trails, total.resolved), (3, 5));
    assert_eq!((total.reused, total.hits), (1, 1));
    assert_eq!((total.patterns, total.used), (1, 1));
    assert_eq!(total.time, 600 + 300 + 1_200 + 240 + 60);
}

/// An ask with no steps under it counts as an ended step, and its uses as hits.
#[test]
fn an_ask_with_no_steps_under_it_counts_as_a_step() {
    let mut desk = shop();
    let pattern = desk.pattern(500, "shop");
    let alone = desk.add(500, "shop", None, "alone", &[&pattern], None);
    desk.done(400, 60, &alone);
    desk.trails.at(super::super::tests::noon());
    let total = Model::of(&desk.trails, Total::of);
    assert_eq!((total.trails, total.resolved), (4, 6));
    assert_eq!((total.reused, total.hits), (2, 2));
}
