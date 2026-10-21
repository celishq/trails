use super::*;
use crate::command::graph::tests::Bench;
use std::time::Duration;

/// The steps by state count each state with its mark, and its word when asked, leaving out a state
/// with none.
#[test]
fn a_tally_counts_each_state_by_its_mark() {
    let mut bench = Bench::new();
    let trail = bench.step("shop", None, "the trail");
    let a = bench.step("shop", Some(&trail), "a");
    let b = bench.step("shop", Some(&trail), "b");
    let c = bench.step("shop", Some(&trail), "c");
    bench.start(&a);
    bench.done(&a, false);
    bench.start(&b);
    bench.done(&b, false);
    bench.drop(&c);
    let steps = bench.trails.steps();
    let under: Vec<Step> = steps.find(&trail).unwrap().children();
    let text = |words| {
        tally(&under, "⠋", words)
            .iter()
            .map(|s| s.content.to_string())
            .collect::<String>()
    };
    assert_eq!(text(false), "✓ 2 · ✗ 1");
    assert_eq!(text(true), "✓ 2 done · ✗ 1 dropped");
}

/// A step done just now draws its check stroke by stroke before it settles on `✓`, while one done
/// before the view opened shows `✓` at once.
#[test]
fn a_check_is_drawn_as_a_step_is_done() {
    let mut bench = Bench::new();
    let trail = bench.step("shop", None, "the trail");
    let a = bench.step("shop", Some(&trail), "a");
    bench.start(&a);
    let mut seen = Seen::new();
    let now = Instant::now();
    let note = |seen: &mut Seen, bench: &Bench, now| {
        let steps = bench.trails.steps();
        let trails = crate::command::graph::folders::folders(&steps);
        seen.note(&trails, |_| vec![], now);
    };
    note(&mut seen, &bench, now);
    bench.done(&a, false);
    note(&mut seen, &bench, now);
    let steps = bench.trails.steps();
    let step = steps.find(&a).unwrap();
    let unfolded = HashSet::new();
    let at = |later: u64| Moment {
        seen: &seen,
        now: now + Duration::from_millis(later),
        unfolded: &unfolded,
        at: 0,
    };
    assert!(STROKES.contains(&at(200).mark(step).0));
    assert_eq!(at(2000).mark(step).0, "✓");
}
