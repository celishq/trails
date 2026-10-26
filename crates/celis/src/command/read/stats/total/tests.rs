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
