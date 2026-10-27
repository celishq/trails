use super::super::tests::shop;
use super::*;

/// A done step that a later step revises is reworked, else one that uses a pattern is reused, else
/// new; a dropped step is dropped, and a done step that uses a pattern applied it.
#[test]
fn each_closed_step_counts_once_by_how_it_ended() {
    let month = shop().month();
    let now = Outcomes {
        reused: 1,
        new: 1,
        reworked: 1,
        dropped: 1,
        expired: 0,
        applied: 1,
    };
    assert_eq!(month.now(), now);
    assert_eq!(month.now().resolved(), 4);
    assert_eq!(month.now().reuse_rate(), 33);
    assert_eq!(month.now().first_pass_rate(), 50);
    assert_eq!(month.days[DAYS - 1], now);
}

/// A step that closed 20 days ago counts in the prior 14 days on its day, and the median of each half
/// reads the successful steps of that half.
#[test]
fn a_step_of_20_days_ago_counts_in_the_prior_half() {
    let month = shop().month();
    assert_eq!(month.prior().new, 1);
    assert_eq!(month.prior().resolved(), 1);
    assert_eq!(month.days[DAYS - 1 - 20].new, 1);
    assert_eq!(month.prior_median, Some(600));
    assert_eq!(month.median, Some(300));
    assert_eq!(month.spans, [1, 1, 1, 0, 0]);
}

/// Each project's time sums its steps with none under them, with the time of a revised step as rework
/// and of a dropped one as dropped; the pattern the reused step used counts as
/// applied in its project.
#[test]
fn time_rework_and_patterns_come_from_the_steps() {
    let month = shop().month();
    let shop = Project {
        name: "shop".into(),
        active: 300 + 1_200 + 240 + 60,
        rework: 1_200,
        dropped: 240,
    };
    assert_eq!(month.projects, vec![shop]);
    assert_eq!(month.prior_active, 600);
    assert_eq!((month.patterns, month.learned, month.kept), (1, 1, 1));
    assert_eq!(month.applied, vec![("shop".to_string(), 1)]);
}

/// An ask with no steps under it counts as a step, and its uses as applied; the uses of an ask with
/// steps under it count as hits of its pattern.
#[test]
fn an_ask_counts_as_a_step_when_no_step_is_under_it() {
    let mut desk = shop();
    let pattern = desk.pattern(500, "shop");
    let alone = desk.add(500, "shop", None, "alone", &[&pattern], None);
    desk.done(400, 60, &alone);
    let parent = desk.add(500, "shop", None, "parent", &[&pattern], None);
    let part = desk.add(500, "shop", Some(&parent), "part", &[], None);
    desk.done(300, 60, &part);
    desk.done(200, 0, &parent);
    let month = desk.month();
    assert_eq!((month.now().reused, month.now().new), (2, 2));
    assert_eq!(month.now().applied, 2);
    assert_eq!(month.top[0], (pattern, month.top[0].1.clone(), 2));
}
