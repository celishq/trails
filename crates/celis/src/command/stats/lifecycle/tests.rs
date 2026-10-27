use super::super::tests::{noon, Desk};
use super::*;
use crate::progress::Model;
use celis_trails::TrailStore;

/// A store of `shop` with a pattern written 2 days ago and used today, recalled, and misled by a fix,
/// and a pattern written today, replaced, and the new one promoted.
fn shop() -> Desk {
    let mut desk = Desk::new();
    let old = desk.pattern(2 * 86_400, "shop");
    let first = desk.pattern(3_000, "shop");
    let next = desk.revise(2_000, "shop", &first);
    desk.trails.at(noon() - 1_000);
    desk.trails.drop(&next, "promoted to AGENTS.md").unwrap();
    let ask = desk.add(900, "shop", None, "the price", &[&old], None);
    desk.recall(800, &ask, &old);
    desk.done(700, 60, &ask);
    desk.add(600, "shop", None, "the price again", &[], Some(&ask));
    desk
}

/// Each span counts the patterns written, replaced, dropped and promoted in it, and the recalls, uses
/// and misled steps of them (`spec/02-cli.md`, "Stats").
#[test]
fn each_span_counts_what_happened_to_the_patterns_in_it() {
    let mut desk = shop();
    desk.trails.at(noon());
    let spans = Model::of(&desk.trails, |m| Lifecycle::of(m, Some("shop")));
    let kinds: Vec<&str> = spans.iter().map(|s| s.span).collect();
    assert_eq!(kinds.len(), 17);
    assert_eq!((kinds[0], kinds[7], kinds[11]), ("day", "week", "month"));
    let today = &spans[6];
    assert_eq!(today.from, crate::text::clock::midnight(noon()));
    let counts = |s: &Lifecycle| {
        let Lifecycle {
            written,
            replaced,
            dropped,
            promoted,
            recalled,
            used,
            misled,
            ..
        } = *s;
        [written, replaced, dropped, promoted, recalled, used, misled]
    };
    assert_eq!(counts(today), [1, 1, 1, 1, 1, 1, 1]);
    assert_eq!(counts(&spans[4]), [1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(counts(&spans[10]), [2, 1, 1, 1, 1, 1, 1]);
    assert_eq!(counts(&spans[16]), [2, 1, 1, 1, 1, 1, 1]);
    assert_eq!(
        today.line(),
        format!(
            "lifecycle day {} written:1 replaced:1 dropped:1 promoted:1 recalled:1 used:1 misled:1",
            crate::text::clock::date(noon())
        )
    );
}
