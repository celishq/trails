use super::month::Month;
use crate::progress::Model;
use crate::text::clock::midnight;
use celis_patterns::{PatternRecord, PatternStore, Scope};
use celis_store::{Anchor, Home, Store};
use celis_trails::{Draft, TrailStore};
use tempfile::TempDir;

/// Noon of a day, the time a desk reads its month at.
pub(super) fn noon() -> u64 {
    midnight(1_800_000_000) + 12 * 3_600
}

/// A store the tests write in at the times they pick: its folder and its trails.
pub(super) struct Desk {
    pub home: TempDir,
    pub trails: Store,
}

impl Desk {
    pub fn new() -> Desk {
        let home = tempfile::tempdir().unwrap();
        let trails = Store::open(Home::open(home.path()).unwrap(), noon(), 600);
        Desk { home, trails }
    }

    /// Writes, `ago` seconds before noon, the step `title` of `project` under `under`, or a trail, that
    /// uses `uses` and replaces `replaces`; its id.
    pub fn add(
        &mut self,
        ago: u64,
        project: &str,
        under: Option<&str>,
        title: &str,
        uses: &[&str],
        replaces: Option<&str>,
    ) -> String {
        self.trails.at(noon() - ago);
        let draft = Draft {
            content: title.to_string(),
            anchor: Anchor {
                project: project.to_string(),
                name: project.to_string(),
            },
            places: vec![],
            check: None,
            under: under.map(str::to_string),
            replaces: replaces.map(str::to_string),
            uses: uses.iter().map(|u| u.to_string()).collect(),
            session: None,
            new: under.is_none(),
        };
        let id = self.trails.add(draft).unwrap().records[0].id.clone();
        self.trails.at(noon() - ago);
        id
    }

    /// Starts the step `id` `took` seconds before `ago`, and closes it done `ago` seconds before noon.
    pub fn done(&mut self, ago: u64, took: u64, id: &str) {
        self.trails.at(noon() - ago - took);
        self.trails.start(id).unwrap();
        self.trails.at(noon() - ago);
        let home = self.home.path().to_path_buf();
        self.trails
            .done(id, celis_trails::Done::new("it works".into(), None), &home)
            .unwrap();
        self.trails.at(noon() - ago);
    }

    /// Starts the step `id` `took` seconds before `ago`, and drops it `ago` seconds before noon.
    pub fn drop(&mut self, ago: u64, took: u64, id: &str) {
        self.trails.at(noon() - ago - took);
        self.trails.start(id).unwrap();
        self.trails.at(noon() - ago);
        self.trails.drop(id, "not needed").unwrap();
        self.trails.at(noon() - ago);
    }

    /// Writes a pattern of `project` `ago` seconds before noon; its id.
    pub fn pattern(&mut self, ago: u64, project: &str) -> String {
        self.trails.at(noon() - ago);
        let pattern = PatternRecord {
            anchor: Anchor {
                project: project.to_string(),
                name: project.to_string(),
            },
            scope: Scope::Project,
            replaces: None,
            content: "When you price: round down, because the shelf does.".into(),
            paths: vec![],
            old_rule: false,
            within: vec![],
            examples: Default::default(),
        };
        let id = self.trails.decide(pattern, None).unwrap().id;
        self.trails.at(noon() - ago);
        id
    }

    /// The month of the store at noon.
    pub fn month(&mut self) -> Month {
        self.trails.at(noon());
        Model::of(&self.trails, Month::of)
    }
}

/// A store of `shop` whose steps 20 days ago and today end each way: a step done new 20 days ago, then
/// today one done that follows it and applies a pattern, one done that a later step revises, the revision,
/// and one dropped.
pub(super) fn shop() -> Desk {
    let mut desk = Desk::new();
    let day = 86_400;
    let old = desk.add(20 * day, "shop", None, "the old trail", &[], None);
    let source = desk.add(20 * day, "shop", Some(&old), "the source", &[], None);
    desk.done(20 * day - 600, 600, &source);
    let pattern = desk.pattern(3_000, "shop");
    let trail = desk.add(3_000, "shop", None, "the trail", &[], None);
    let reused = desk.add(
        3_000,
        "shop",
        Some(&trail),
        "reused",
        &[&source, &pattern],
        None,
    );
    let revised = desk.add(3_000, "shop", Some(&trail), "revised", &[], None);
    let dropped = desk.add(3_000, "shop", Some(&trail), "dropped", &[], None);
    desk.done(2_700, 300, &reused);
    desk.done(1_500, 1_200, &revised);
    desk.drop(1_200, 240, &dropped);
    let next = desk.add(900, "shop", None, "the next trail", &[], None);
    let fix = desk.add(900, "shop", Some(&next), "the fix", &[], Some(&revised));
    desk.done(600, 60, &fix);
    desk
}
