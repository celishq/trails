//! CIP-10, "Reading".

mod support;

use support::*;

/// A reader works out strength as 1, plus the `+` ratings, less the `-` ratings.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn strength_is_one_plus_helped_less_wrong() {
    let repo = Repo::new();
    let writer = agent(&repo, "w");
    let raters: Vec<_> = ["b", "c", "d"].map(|s| agent(&repo, s)).into();
    let cairn = finding(&repo, &writer, now() - 1000);
    assert_eq!(strength(&repo, &cairn), Some(1));

    rating(&repo, &raters[0], &cairn, true, now() - 500);
    rating(&repo, &raters[1], &cairn, true, now() - 400);
    rating(&repo, &raters[2], &cairn, false, now() - 300);

    assert_eq!(strength(&repo, &cairn), Some(2));
}

/// A confirmed cairn lives 14 days from its newest support: the cairn, or its newest `+` rating.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_confirmed_cairn_lives_14_days_from_its_newest_support() {
    let repo = Repo::new();
    let (writer, b) = (agent(&repo, "w"), agent(&repo, "b"));
    let supported = finding(&repo, &writer, now() - 20 * DAY);
    rating(&repo, &b, &supported, true, now() - 10 * DAY);
    let lone = finding(&repo, &writer, now() - 21 * DAY);
    let fresh = finding(&repo, &writer, now() - 13 * DAY);
    rating(&repo, &b, &fresh, true, now() - 12 * DAY);

    let ids: Vec<String> = listed(&repo, "reader", &[])
        .iter()
        .map(|o| o["id"].as_str().unwrap().to_string())
        .collect();

    assert!(ids.contains(&supported.id) && ids.contains(&fresh.id));
    assert!(!ids.contains(&lone.id));
}

/// A confirmed cairn is live up to 14 days from its support and gone a minute after.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_confirmed_cairn_expires_at_14_days_to_the_minute() {
    let repo = Repo::new();
    let (writer, b) = (agent(&repo, "w"), agent(&repo, "b"));
    let inside = finding(&repo, &writer, now() - 30 * DAY);
    rating(&repo, &b, &inside, true, now() - 14 * DAY + 60);
    let outside = finding(&repo, &writer, now() - 31 * DAY);
    rating(&repo, &b, &outside, true, now() - 14 * DAY - 60);

    assert_eq!(strength(&repo, &inside), Some(2));
    assert_eq!(strength(&repo, &outside), None);
}

/// A cairn no other agent rated `+` lives 3 days from its writing and is gone a minute after.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn an_unconfirmed_cairn_expires_at_3_days_to_the_minute() {
    let repo = Repo::new();
    let writer = agent(&repo, "w");
    let inside = finding(&repo, &writer, now() - 3 * DAY + 60);
    let outside = finding(&repo, &writer, now() - 3 * DAY - 60);

    assert_eq!(strength(&repo, &inside), Some(1));
    assert_eq!(strength(&repo, &outside), None);
}

/// A spot keeps 14 days from its writing though no agent rated it.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_spot_lives_14_days() {
    let first_spot_at = |age: u64| {
        let repo = Repo::new();
        let by = agent(&repo, "a");
        let spot = cairn(&repo, &by, "spot", &[("else", "E-NET")], now() - age);
        let out = run(&repo, "a", &["list", "--json", "--error", "E-NET"]);
        let shown = out
            .lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
            .find_map(|o| o.get("spot").cloned());
        shown == Some(serde_json::Value::from(spot.id.as_str()))
    };

    assert!(first_spot_at(14 * DAY - 60));
    assert!(!first_spot_at(14 * DAY + 60));
}

/// A reader works out expiry from the records alone: a `-` rating is not support.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_wrong_rating_is_not_support() {
    let repo = Repo::new();
    let (writer, b, c) = (agent(&repo, "w"), agent(&repo, "b"), agent(&repo, "c"));
    let old = finding(&repo, &writer, now() - 20 * DAY);
    rating(&repo, &b, &old, true, now() - 18 * DAY);
    rating(&repo, &c, &old, false, now() - 1000);

    assert_eq!(strength(&repo, &old), None);
}

/// A cairn rated `-` by two or more agents is pulled, however strong it is.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_cairn_rated_wrong_by_two_agents_is_pulled() {
    let repo = Repo::new();
    let t = now();
    let writer = agent(&repo, "w");
    let raters: Vec<_> = ["b", "c", "d", "e", "f"].map(|s| agent(&repo, s)).into();
    let one = finding(&repo, &writer, t - 1000);
    let two = finding(&repo, &writer, t - 2000);
    for (i, rater) in raters.iter().take(3).enumerate() {
        rating(&repo, rater, &one, true, t - 500 + i as u64);
        rating(&repo, rater, &two, true, t - 500 + i as u64);
    }
    rating(&repo, &raters[3], &one, false, t - 100);
    rating(&repo, &raters[3], &two, false, t - 100);
    rating(&repo, &raters[4], &two, false, t - 100);

    assert_eq!(strength(&repo, &one), Some(3));
    assert_eq!(strength(&repo, &two), None);
}

/// A reader orders cairns confirmed first, then by strength, then newest first.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_reader_orders_by_strength_then_newest_first() {
    let repo = Repo::new();
    let (writer, b) = (agent(&repo, "w"), agent(&repo, "b"));
    let old = finding(&repo, &writer, now() - 3000);
    let strong = finding(&repo, &writer, now() - 2000);
    let new = finding(&repo, &writer, now() - 1000);
    rating(&repo, &b, &strong, true, now() - 500);

    let ids: Vec<String> = listed(&repo, "reader", &[])
        .iter()
        .map(|o| o["id"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(ids, [strong.id, new.id, old.id]);
}

/// A reader shows spots as they are, now as a count: three spots of one error stay three.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_reader_shows_spots_as_they_are() {
    let repo = Repo::new();
    let spots: Vec<_> = ["a", "b", "c"]
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let by = agent(&repo, s);
            cairn(
                &repo,
                &by,
                "spot",
                &[("else", "E-NET")],
                now() - 1000 + i as u64,
            )
        })
        .collect();

    let out = run(&repo, "a", &["list", "--json", "--error", "E-NET"]);

    let line: serde_json::Value = out
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|o| o.get("spots").is_some())
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(line["spots"], 3);
    assert_eq!(line["spot"], spots[0].id.as_str());
}

/// Among confirmed cairns the stronger one comes first, even when it is older.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn among_confirmed_cairns_the_stronger_comes_first_even_when_older() {
    let repo = Repo::new();
    let (writer, b, c) = (agent(&repo, "w"), agent(&repo, "b"), agent(&repo, "c"));
    let strong = finding(&repo, &writer, now() - 3000);
    let weak = finding(&repo, &writer, now() - 1000);
    rating(&repo, &b, &strong, true, now() - 900);
    rating(&repo, &c, &strong, true, now() - 800);
    rating(&repo, &b, &weak, true, now() - 700);

    let ids: Vec<String> = listed(&repo, "reader", &[])
        .iter()
        .map(|o| o["id"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(ids, [strong.id, weak.id]);
}
