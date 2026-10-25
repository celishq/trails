use super::*;
use celis_trails::{Anchor, Draft, Rule, Scope, Store, Trails};
use tempfile::TempDir;

/// The second the first record of a bench is written.
const AT: u64 = 1_800_000_000;

/// A store the tests write steps in, a second apart, read again after each: its folder, its trails, and
/// the second of the last record.
pub(super) struct Bench {
    pub home: TempDir,
    pub trails: Trails,
    pub at: u64,
}

impl Bench {
    pub fn new() -> Bench {
        let home = tempfile::tempdir().unwrap();
        let trails = Trails::open(Store::open(home.path()).unwrap(), AT, 600);
        Bench {
            home,
            trails,
            at: AT,
        }
    }

    /// Moves the store a second on and reads it again.
    fn tick(&mut self) {
        self.at += 1;
        self.trails.at(self.at);
    }

    /// Writes the step `title` of `project` under `under`, or a trail with none; its id.
    pub fn step(&mut self, project: &str, under: Option<&str>, title: &str) -> String {
        self.tick();
        let draft = Draft {
            content: title.to_string(),
            anchor: Anchor {
                project: project.to_string(),
                name: project.to_string(),
            },
            places: vec![],
            check: None,
            under: under.map(str::to_string),
            replaces: None,
            uses: vec![],
            session: None,
            new: false,
        };
        let id = self.trails.add(draft).unwrap().records[0].id.clone();
        self.trails.at(self.at);
        id
    }

    pub fn start(&mut self, id: &str) {
        self.tick();
        self.trails.start(id).unwrap();
        self.trails.at(self.at);
    }

    /// Closes the step `id` done, otherwise than planned when `changed`.
    pub fn done(&mut self, id: &str, changed: bool) {
        self.tick();
        let changed = changed.then(|| "another way".to_string());
        let home = self.home.path().to_path_buf();
        self.trails
            .done(
                id,
                celis_trails::Done::new("it works".into(), changed),
                &home,
            )
            .unwrap();
        self.trails.at(self.at);
    }

    pub fn drop(&mut self, id: &str) {
        self.tick();
        self.trails.drop(id, "not needed").unwrap();
        self.trails.at(self.at);
    }

    /// Writes the rule `text` of the trail `trail` of `project`; its id.
    pub fn rule(&mut self, project: &str, trail: &str, text: &str) -> String {
        self.tick();
        let rule = Rule {
            anchor: Anchor {
                project: project.to_string(),
                name: project.to_string(),
            },
            scope: Scope::Trail(trail.to_string()),
            replaces: None,
            content: text.to_string(),
            paths: vec![],
            situation: false,
            promoted: false,
            examples: Default::default(),
        };
        let id = self.trails.decide(rule, None).unwrap().records[0]
            .id
            .clone();
        self.trails.at(self.at);
        id
    }
}

/// The rows of `buf` as text, each with its spaces at the end cut.
pub(super) fn text(buf: &Buffer) -> Vec<String> {
    lines(buf)
        .iter()
        .map(|l| l.to_string().trim_end().to_string())
        .collect()
}

/// A bench with three trails: in `shop` an older one and a newer one, in `crm` one between them; each
/// with a step at work.
fn shelves() -> Bench {
    let mut bench = Bench::new();
    let old = bench.step("shop", None, "the old shop trail");
    let a = bench.step("shop", Some(&old), "a step");
    bench.start(&a);
    let crm = bench.step("crm", None, "the crm trail");
    let b = bench.step("crm", Some(&crm), "b step");
    bench.start(&b);
    let new = bench.step("shop", None, "the new shop trail");
    let c = bench.step("shop", Some(&new), "c step");
    bench.start(&c);
    bench
}

/// The lines of the screen of `bench` drawn by `view`, 100 wide and `rows` tall, with `seen` noting the
/// store first.
fn screen(bench: &Bench, view: &mut View, seen: &mut Seen, rows: u16) -> Vec<String> {
    Model::of(&bench.trails, |model| {
        let trails = folders(model.steps);
        seen.note(&trails, |t| rule_ids(model, t), Instant::now());
        let lines = frame(model, &trails, view, seen, (100, rows), Instant::now());
        lines
            .iter()
            .map(|l| l.to_string().trim_end().to_string())
            .collect()
    })
}

/// All shows every trail under its project's folder, the folders in the order their oldest trails
/// started and the newest trail first in each, with how many trails each folder holds.
#[test]
fn all_shows_each_trail_under_its_folder() {
    let bench = shelves();
    let mut seen = Seen::settled();
    let mut view = Model::of(&bench.trails, |m| View::new(&folders(m.steps), None));
    let lines = screen(&bench, &mut view, &mut seen, 40);
    let at = |text: &str| lines.iter().position(|l| l.contains(text)).expect(text);
    assert!(lines[0].starts_with("  All trails"), "{lines:#?}");
    assert!(lines[2].contains("3 trails today"), "{lines:#?}");
    assert!(at("📁 shop  ·  2 trails") < at("the new shop trail"));
    assert!(at("the new shop trail") < at("the old shop trail"));
    assert!(at("the old shop trail") < at("📁 crm  ·  1 trail"));
}

/// A trail that starts while All shows its folder at the top goes first in the folder and pushes what
/// is below it down: the folder stays on its row.
#[test]
fn a_new_trail_pushes_all_down_under_its_folder() {
    let mut bench = shelves();
    let mut seen = Seen::settled();
    let mut view = Model::of(&bench.trails, |m| View::new(&folders(m.steps), None));
    let before = screen(&bench, &mut view, &mut seen, 40);
    let folder = before.iter().position(|l| l.contains("📁 shop")).unwrap();
    let newest = bench.step("shop", None, "the newest shop trail");
    let d = bench.step("shop", Some(&newest), "d step");
    bench.start(&d);
    let after = screen(&bench, &mut view, &mut seen, 40);
    assert_eq!(
        after.iter().position(|l| l.contains("📁 shop")),
        Some(folder),
        "{after:#?}"
    );
    let at = |text: &str| after.iter().position(|l| l.contains(text)).expect(text);
    assert!(
        at("the newest shop trail") < at("the new shop trail"),
        "{after:#?}"
    );
}

/// A trail that starts above the screen leaves the screen still and says on the line above that a new
/// trail is there.
#[test]
fn a_new_trail_above_the_screen_leaves_it_still() {
    let mut bench = shelves();
    let mut seen = Seen::settled();
    let mut view = Model::of(&bench.trails, |m| View::new(&folders(m.steps), None));
    screen(&bench, &mut view, &mut seen, 20);
    view.offset = 8.0;
    let before = screen(&bench, &mut view, &mut seen, 20);
    let newest = bench.step("shop", None, "the newest shop trail");
    let d = bench.step("shop", Some(&newest), "d step");
    bench.start(&d);
    let after = screen(&bench, &mut view, &mut seen, 20);
    let row = |lines: &[String]| lines.iter().position(|l| l.contains("the old shop trail"));
    assert_eq!(row(&before), row(&after), "{after:#?}");
    assert!(row(&after).is_some(), "{after:#?}");
    assert!(
        after[usize::from(HEAD)].contains("● a new trail above"),
        "{after:#?}"
    );
}

/// A trail's graph heads with its path, and counts the other trails that moved since they were last
/// shown.
#[test]
fn a_trail_heads_with_its_path_and_the_trails_that_moved() {
    let mut bench = shelves();
    let mut seen = Seen::new();
    let mut view = Model::of(&bench.trails, |m| View::new(&folders(m.steps), None));
    screen(&bench, &mut view, &mut seen, 40);
    let old = view.matches.last().cloned();
    view.show(old);
    let lines = screen(&bench, &mut view, &mut seen, 40);
    assert!(lines[0].contains("All trails  ›  📁 crm  ›"), "{lines:#?}");
    let other = view
        .trails
        .iter()
        .find(|t| Some(*t) != view.trail.as_ref())
        .cloned()
        .unwrap();
    bench.step("shop", Some(&other), "e step");
    let lines = screen(&bench, &mut view, &mut seen, 40);
    assert!(lines[0].ends_with("● 1 other trail moved"), "{lines:#?}");
}

/// Off a terminal, the graph prints All with every trail and the rules, as plain text.
#[test]
fn printed_shows_all_with_the_rules() {
    let mut bench = shelves();
    let shop = Model::of(&bench.trails, |m| folders(m.steps)[0].id().to_string());
    bench.rule(
        "shop",
        &shop,
        "When you price: round down, because the shelf does.",
    );
    let text = Model::of(&bench.trails, |model| printed(model, None)).unwrap();
    assert!(text.contains("📁 shop"), "{text}");
    assert!(text.contains("§ Rules 1"), "{text}");
    assert!(!text.contains('\x1b'), "{text}");
}
