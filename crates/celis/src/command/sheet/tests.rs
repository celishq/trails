use super::*;
use celis_trails::{Anchor, Draft, Store};
use tempfile::TempDir;

/// The second a store's one step closed.
const AT: u64 = 1_800_000_000;

/// A store with a trail and one step under it done as planned at `AT`, seen at `AT + later`.
fn worked(later: u64) -> (TempDir, Trails) {
    let home = tempfile::tempdir().unwrap();
    let mut trails = Trails::open(Store::open(home.path()).unwrap(), AT - 60, 600);
    let draft = |text: &str, under: Option<String>| Draft {
        content: text.to_string(),
        anchor: Anchor {
            project: "repo".into(),
            name: "repo".into(),
        },
        places: vec![],
        check: None,
        under,
        replaces: None,
        uses: vec![],
        session: None,
        new: false,
    };
    let trail = trails.add(draft("the trail", None)).unwrap().records[0]
        .id
        .clone();
    trails.at(AT - 60);
    let part = trails.add(draft("the part", Some(trail))).unwrap().records[0]
        .id
        .clone();
    trails.at(AT - 60);
    trails.start(&part).unwrap();
    trails.at(AT);
    trails
        .done(&part, "it works".into(), None, None, home.path())
        .unwrap();
    trails.at(AT + later);
    (home, trails)
}

/// The face of the card of `trails` whose title starts with `title`, its rows joined by new lines.
fn face(trails: &Trails, title: &str) -> String {
    let sheets = sheets(trails, 92, 43, false, false);
    let sheet = sheets.iter().find(|s| s.title.starts_with(title));
    sheet.expect(title).face.join("\n")
}

/// With no record in the 14 days before, the effect says `no data before`, and the work card says
/// `first 14 days` and draws a dot on each day with no step done (`spec/02-cli.md`, "Cards").
#[test]
fn a_card_with_no_data_before_says_so() {
    let (_home, trails) = worked(3_600);
    let effect = face(&trails, "00 EFFECT");
    assert!(effect.contains("· no data before"), "{effect}");
    assert!(!effect.contains("few records"), "{effect}");
    let work = face(&trails, "03 WORK");
    assert!(work.contains("first 14 days"), "{work}");
    assert!(!work.contains("vs 14 days before"), "{work}");
    assert!(work.contains("· · ·"), "{work}");
}

/// With no step done or closed in the last 14 days, the work, time and outcome cards each say so in one
/// line in place of their chart (`spec/02-cli.md`, "Cards").
#[test]
fn a_card_with_nothing_in_14_days_says_so() {
    let (_home, trails) = worked(20 * DAY);
    let work = face(&trails, "03 WORK");
    assert!(work.contains("no step done in 14 days"), "{work}");
    assert!(!work.contains("today"), "{work}");
    let time = face(&trails, "04 TIME");
    assert!(time.contains("no step done in 14 days"), "{time}");
    assert!(!time.contains("< 5m"), "{time}");
    let outcome = face(&trails, "05 OUTCOME");
    assert!(outcome.contains("no step closed in 14 days"), "{outcome}");
    assert!(!outcome.contains("dropped"), "{outcome}");
}

/// The effect counts its base as `1 step`, and a big share puts its `%` right after its digits
/// (`spec/02-cli.md`, "Cards").
#[test]
fn the_effect_counts_one_step_and_its_share_whole() {
    let (_home, trails) = worked(3_600);
    let effect = face(&trails, "00 EFFECT");
    assert!(effect.contains("1 step "), "{effect}");
    assert!(!effect.contains("1 steps"), "{effect}");
    assert!(effect.contains("▀▀▀%"), "{effect}");
}
