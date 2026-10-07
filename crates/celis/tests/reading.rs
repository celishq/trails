//! CIP-10, "Reading".
mod support;

use support::*;

/// A cairn written 14 days ago with no rating has the score 0.5.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_cairn_written_14_days_ago_scores_one_half() {
    let repo = Repo::new();
    let writer = agent(&repo, "w");
    let cairn = finding(&repo, &writer, now() - 14 * DAY);

    assert_near(score(&repo, &cairn), 0.5);
}

/// A cairn written 28 days ago and rated `+` today scores 1.25, and comes before a new cairn with no rating.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn an_old_cairn_rated_today_scores_1_25_and_precedes_a_new_one() {
    let repo = Repo::new();
    let (writer, b) = (agent(&repo, "w"), agent(&repo, "b"));
    let old = finding(&repo, &writer, now() - 28 * DAY);
    let new = finding(&repo, &writer, now() - 1000);
    rating(&repo, &b, &old, true, now());

    let ids: Vec<String> = listed(&repo, "reader", &[])
        .iter()
        .map(|o| o["id"].as_str().unwrap().to_string())
        .collect();

    assert_near(score(&repo, &old), 1.25);
    assert_eq!(ids, [old.id, new.id]);
}

/// A new cairn rated `-` by two agents scores -1, and the list still prints it, last.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_cairn_rated_wrong_by_two_agents_scores_minus_one_and_prints_last() {
    let repo = Repo::new();
    let (writer, b, c) = (agent(&repo, "w"), agent(&repo, "b"), agent(&repo, "c"));
    let bad = finding(&repo, &writer, now() - 1000);
    let fine = finding(&repo, &writer, now() - 2000);
    rating(&repo, &b, &bad, false, now() - 500);
    rating(&repo, &c, &bad, false, now() - 400);

    let ids: Vec<String> = listed(&repo, "reader", &[])
        .iter()
        .map(|o| o["id"].as_str().unwrap().to_string())
        .collect();

    assert_near(score(&repo, &bad), -1.0);
    assert_eq!(ids, [fine.id, bad.id]);
}

/// A cairn stays in the list however old it is.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn no_cairn_leaves_the_list_for_its_age() {
    let repo = Repo::new();
    let writer = agent(&repo, "w");
    let old = finding(&repo, &writer, now() - 200 * DAY);

    assert!(score(&repo, &old).is_some());
}

/// A cairn of another project with a higher score comes before this project's cairn.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_reader_orders_by_score_across_projects() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let write = |folder: &std::path::Path, text: &str| {
        let o = repo.run(folder, &["add", "--task", "t", "--", text], &[], "");
        assert!(o.status.success(), "{}", err(&o));
        out(&o).trim().to_string()
    };
    write(&repo.path(), "Before you run make, here.");
    let theirs = write(&shop, "Before you run make, in the shop.");
    let rated = repo.run(&shop, &["rate", &theirs, "+"], &[], "");
    assert!(rated.status.success(), "{}", err(&rated));

    let here: Vec<String> = listed(&repo, "reader", &[])
        .iter()
        .map(|o| o["content"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(
        here,
        [
            "Before you run make, in the shop.",
            "Before you run make, here."
        ]
    );
}

/// A reader orders cairns by score, then newest first.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_reader_orders_by_score_then_newest_first() {
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

/// Among rated cairns the higher score comes first, even when it is older.
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

/// A cairn of weight 10 written 14 days ago scores 5, and a weight tag that is "11" or "x" scores as weight 1
/// (`spec/cairns/10-cairns.md`, "Reading").
#[test]
fn a_cairn_of_weight_10_scores_5_and_a_bad_weight_scores_as_1() {
    let repo = Repo::new();
    let writer = agent(&repo, "w");
    let at = now() - 14 * DAY;
    let heavy = cairn(&repo, &writer, "finding", &[("weight", "10")], at);
    let high = cairn(&repo, &writer, "finding", &[("weight", "11")], at);
    let text = cairn(&repo, &writer, "finding", &[("weight", "x")], at);

    assert_near(score(&repo, &heavy), 5.0);
    assert_near(score(&repo, &high), 0.5);
    assert_near(score(&repo, &text), 0.5);
}

/// A weight tag of "0", "11", "+5" or "x" in a record reads as weight 1.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_reader_must_read_a_weight_that_is_not_a_whole_number_from_1_to_10_as_1() {
    let repo = Repo::new();
    let writer = agent(&repo, "w");
    let at = now() - 14 * DAY;
    for bad in ["0", "11", "+5", "x"] {
        let note = cairn(&repo, &writer, "finding", &[("weight", bad)], at);
        assert_near(score(&repo, &note), 0.5);
    }
}

/// A rating written after a first read changes the score of the next read.
///
/// (`spec/cairns/10-cairns.md`, "Reading")
#[test]
fn a_reader_must_work_out_the_score_from_the_records_alone_at_the_time_it_reads() {
    let repo = Repo::new();
    let (writer, b) = (agent(&repo, "w"), agent(&repo, "b"));
    let note = finding(&repo, &writer, now() - 14 * DAY);
    assert_near(score(&repo, &note), 0.5);

    rating(&repo, &b, &note, true, now());

    assert_near(score(&repo, &note), 1.5);
}
