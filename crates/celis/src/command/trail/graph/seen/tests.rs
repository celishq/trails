use super::*;
use crate::command::trail::graph::tests::Bench;
use celis_trails::TrailStore;
use std::time::Duration;

/// What stood on the first look plays nothing; a step added later, and a step whose state changed,
/// change from the moment the view saw them, with the state before kept.
#[test]
fn only_what_changes_after_the_first_look_plays() {
    let mut bench = Bench::new();
    let trail = bench.step("shop", None, "the trail");
    let a = bench.step("shop", Some(&trail), "a");
    let mut seen = Seen::new();
    let now = Instant::now();
    let note = |seen: &mut Seen, bench: &Bench, now| {
        let steps = bench.trails.steps();
        let trails = crate::command::trail::graph::folders::folders(&steps);
        seen.note(&trails, |_| vec![], now);
    };
    note(&mut seen, &bench, now);
    assert_eq!(seen.changed(&a, now), None);
    bench.start(&a);
    let b = bench.step("shop", Some(&trail), "b");
    let later = now + Duration::from_secs(1);
    note(&mut seen, &bench, later);
    let soon = later + Duration::from_millis(500);
    assert_eq!(seen.changed(&a, soon), Some(0.5));
    assert_eq!(seen.step(&a).and_then(|c| c.before), Some(Phase::Open));
    assert_eq!(seen.added(&a, soon), None);
    assert_eq!(seen.added(&b, soon), Some(0.5));
}

/// A trail that moved since it was last shown has news, and none once it is shown.
#[test]
fn a_trail_has_news_until_it_is_shown() {
    let mut bench = Bench::new();
    let trail = bench.step("shop", None, "the trail");
    let mut seen = Seen::new();
    let now = Instant::now();
    let note = |seen: &mut Seen, bench: &Bench, now| {
        let steps = bench.trails.steps();
        let trails = crate::command::trail::graph::folders::folders(&steps);
        seen.note(&trails, |_| vec![], now);
    };
    note(&mut seen, &bench, now);
    assert!(!seen.news(&trail));
    let a = bench.step("shop", Some(&trail), "a");
    bench.start(&a);
    note(&mut seen, &bench, now + Duration::from_secs(1));
    assert!(seen.news(&trail));
    seen.show(&trail, now + Duration::from_secs(2));
    assert!(!seen.news(&trail));
}
