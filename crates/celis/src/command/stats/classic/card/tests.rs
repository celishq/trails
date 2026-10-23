use super::super::window::DAY;
use super::*;
use celis_trails::{Anchor, Draft, Rule, Scope, Store, Trails};
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
        .done(
            &part,
            celis_trails::Done::new("it works".into(), None),
            None,
            home.path(),
        )
        .unwrap();
    trails.at(AT + later);
    (home, trails)
}

/// The cards of `trails`, 92 and 43 characters wide inside.
fn drawn(trails: &Trails) -> [Card; CARDS] {
    Model::of(trails, |model| cards(model, 92, 43, false))
}

/// The face of the card of `trails` whose title starts with `title`, its rows joined by new lines.
fn face(trails: &Trails, title: &str) -> String {
    let cards = drawn(trails);
    let card = cards.iter().find(|s| s.title.starts_with(title));
    let face = card.expect(title).face.iter().map(|r| r.to_string());
    face.collect::<Vec<_>>().join("\n")
}

/// With no record in the 14 days before, the effect prints no change row and no `vs 14 BEFORE`, and the
/// work card no change, and a dot on each day with no step done.
#[test]
fn a_card_with_no_data_before_shows_no_change() {
    let (_home, trails) = worked(3_600);
    let cards = drawn(&trails);
    assert_eq!(cards[0].title, "00 EFFECT · LAST 14 DAYS");
    let effect: Vec<String> = cards[0].face.iter().map(|r| r.to_string()).collect();
    let effect = effect.join("\n");
    assert_eq!(cards[0].face.len(), 5, "{effect}");
    assert!(!effect.contains("no data before"), "{effect}");
    assert!(!effect.contains("few records"), "{effect}");
    let work = face(&trails, "03 WORK");
    assert!(!work.contains("first 14 days"), "{work}");
    assert!(!work.contains("vs 14 days before"), "{work}");
    assert!(work.contains("· · ·"), "{work}");
}

/// With no step done or closed in the last 14 days, the work, time and outcome cards each say so in one
/// line in place of their chart.
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

/// The effect says under each share what it counts among the steps done, and a big share puts its `%` a
/// space after its digits.
#[test]
fn the_effect_says_what_each_share_counts() {
    let (_home, trails) = worked(3_600);
    let effect = face(&trails, "00 EFFECT");
    for counts in [
        "of 1 done, no change",
        "of 1 done, revised",
        "of 1 done, used a rule",
    ] {
        assert!(effect.contains(counts), "{counts:?} in\n{effect}");
    }
    assert!(effect.contains("▀▀▀ %"), "{effect}");
}

/// `worked`, with a new trail added at `AT` and a step under it that follows the done step and uses a
/// rule.
fn reused(later: u64) -> (TempDir, Trails) {
    let (home, mut trails) = worked(later);
    let part = Model::of(&trails, |m| m.steps.all()[1].id().to_string());
    let anchor = Anchor {
        project: "repo".into(),
        name: "repo".into(),
    };
    let rule = Rule {
        anchor: anchor.clone(),
        scope: Scope::Project,
        replaces: None,
        content: "When a rule holds: keep it, because it works.".into(),
        paths: vec![],
        situation: false,
        promoted: false,
        good: None,
        bad: None,
    };
    trails.at(AT);
    let rule = trails.decide(rule, None).unwrap().records[0].id.clone();
    trails.at(AT);
    let draft = |content: &str, under: Option<String>, uses: Vec<String>| Draft {
        content: content.into(),
        anchor: anchor.clone(),
        places: vec![],
        check: None,
        new: under.is_none(),
        under,
        replaces: None,
        uses,
        session: None,
    };
    let trail = trails
        .add(draft("the next trail", None, vec![]))
        .unwrap()
        .records[0]
        .id
        .clone();
    trails.at(AT);
    trails
        .add(draft("the next", Some(trail), vec![part, rule]))
        .unwrap();
    trails.at(AT + later);
    (home, trails)
}

/// The reuse and rules cards draw the follows and rule uses of each of the last 14 days as columns, a
/// dot on each day with none, then the steps and rules used most.
#[test]
fn the_reuse_cards_draw_each_day_as_a_column() {
    let (_home, trails) = reused(3_600);
    for title in ["01 REUSE", "02 RULES USED"] {
        let face = face(&trails, title);
        assert!(face.contains("· · ·"), "{face}");
        assert!(face.contains("today"), "{face}");
    }
    let reuse = face(&trails, "01 REUSE");
    assert!(reuse.contains("follows"), "{reuse}");
    assert!(reuse.ends_with(" 1"), "{reuse}");
    let rules = face(&trails, "02 RULES USED");
    assert!(rules.contains("rule uses"), "{rules}");
    assert!(rules.contains("a rule holds"), "{rules}");
    assert!(rules.ends_with(" 1"), "{rules}");
}

/// With no follow and no rule use in the last 14 days, the reuse and rules cards each say so in one line
/// in place of their columns.
#[test]
fn the_reuse_cards_with_nothing_in_14_days_say_so() {
    let (_home, trails) = reused(20 * DAY);
    let reuse = face(&trails, "01 REUSE");
    assert!(reuse.contains("no follow in 14 days"), "{reuse}");
    assert!(!reuse.contains("today"), "{reuse}");
    let rules = face(&trails, "02 RULES USED");
    assert!(rules.contains("no rule used in 14 days"), "{rules}");
    assert!(!rules.contains("today"), "{rules}");
}

/// The projects card ends each time at one column, the widest time setting it.
#[test]
fn the_projects_card_aligns_its_times() {
    let (home, mut trails) = worked(3_600);
    let anchor = Anchor {
        project: "other".into(),
        name: "other".into(),
    };
    let draft = |content: &str, under: Option<String>| Draft {
        content: content.into(),
        anchor: anchor.clone(),
        places: vec![],
        check: None,
        new: under.is_none(),
        under,
        replaces: None,
        uses: vec![],
        session: None,
    };
    let long = AT + 19 * 3_600 + 14 * 60;
    trails.at(AT);
    trails.add(draft("the long trail", None)).unwrap();
    trails.at(AT);
    let trail = Model::of(&trails, |m| {
        let all = m.steps.all();
        let trail = all.iter().find(|s| s.title() == "the long trail");
        trail.unwrap().id().to_string()
    });
    trails.at(AT);
    let part = trails
        .add(draft("the long part", Some(trail)))
        .unwrap()
        .records[0]
        .id
        .clone();
    trails.at(AT);
    trails.start(&part).unwrap();
    trails.at(long);
    trails
        .done(
            &part,
            celis_trails::Done::new("it took long".into(), None),
            None,
            home.path(),
        )
        .unwrap();
    trails.at(long + 60);
    let face = face(&trails, "06 PROJECTS");
    let ends: Vec<usize> = face.lines().map(|l| l.chars().count()).collect();
    assert!(face.contains("19h 14m"), "{face}");
    assert_eq!(ends[0], ends[1], "{face}");
}
