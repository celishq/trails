//! `--demo` of `tree` and `stats`: made-up trails in every state, with four weeks of history,
//! in a folder of their own; and of `tree --graph`, eleven trails of today that move live.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::{PatternRecord, PatternStore, Scope};
use celis_store::{Anchor, Home, Store};
use celis_trails::{Draft, TrailStore};
use std::cell::{RefCell, RefMut};
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

impl Command {
    /// Runs the command `args` names, `tree` or `stats`, on made-up trails written to a folder of their
    /// own, and removes the folder at the end; `tree --graph` shows the trails of the graph's demo, written live on a terminal.
    pub(crate) fn demo(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let home = std::env::temp_dir().join(format!("celis-demo-{}", std::process::id()));
        let demo = Demo {
            home: home.clone(),
            now: crate::command::now(),
            trails: RefCell::new(None),
        };
        if args.graph {
            let play = graph::Play::start(&home, demo.now, std::io::stdout().is_terminal());
            let shown = play.and_then(|play| {
                let mut env = self.env.clone();
                env.insert("CELIS_HOME".to_string(), home.display().to_string());
                if let Some(opening) = self.opening() {
                    env.insert("CELIS_OPENING".to_string(), opening.display().to_string());
                }
                let command = Command {
                    args: self.args.clone(),
                    env,
                    folder: self.folder.clone(),
                };
                let args = Args {
                    demo: false,
                    ..args.clone()
                };
                let shown = command.tree(&args, out);
                play.stop();
                shown
            });
            let _ = std::fs::remove_dir_all(&home);
            return shown;
        }
        let written = match args.words[0].as_str() {
            "stats" if args.next => demo.month().map(|_| ()),
            _ => demo.write(),
        };
        let shown = written.and_then(|()| {
            let mut env = self.env.clone();
            env.insert("CELIS_HOME".to_string(), home.display().to_string());
            if let Some(opening) = self.opening() {
                env.insert("CELIS_OPENING".to_string(), opening.display().to_string());
            }
            let command = Command {
                args: self.args.clone(),
                env,
                folder: self.folder.clone(),
            };
            let args = Args {
                demo: false,
                ..args.clone()
            };
            match args.words[0].as_str() {
                "stats" => command.stats(&args, out),
                _ => command.tree(&args, out),
            }
        });
        let _ = std::fs::remove_dir_all(&home);
        shown
    }
}

/// The folder the made-up trails are written to, the time they lead up to, and the trails written so
/// far.
struct Demo {
    home: PathBuf,
    now: u64,
    trails: RefCell<Option<Store>>,
}

impl Demo {
    /// Writes the made-up trails: the history of four weeks; a done one of a past day; today, an active
    /// one with a step in each state, patterns and a change, a revised one, and in a second project an
    /// active, an expired and a dropped one.
    fn write(&self) -> Result<(), Failure> {
        let (shop, crm) = (anchor("shop"), anchor("crm"));
        self.history(&shop, &crm)?;
        let day = 2 * 24 * 3600;
        let set = self.add(
            day,
            &shop,
            None,
            "Set up the shop\nexpect: the shop takes its first sale",
            &[],
        )?;
        let till = self.add(
            day - 60,
            &shop,
            Some(&set),
            "Open the till\nexpect: the till opens",
            &[],
        )?;
        self.close(day - 600, &till, "the till opens", None)?;

        let cart = self.add(
            3 * 3600,
            &shop,
            None,
            "Fix the cart total\nexpect: the cart test passes",
            &[],
        )?;
        let floats = self.add(
            3 * 3600 - 60,
            &shop,
            Some(&cart),
            "Add up the cart total\nexpect: the total of three items is their sum",
            &["shelf/cart.py"],
        )?;
        self.close(3 * 3600 - 400, &floats, "the total adds up", None)?;
        let cents = self.replace(
            3 * 3600 - 500,
            &shop,
            &cart,
            "Add up the cart total in cents\nexpect: the total is correct to the cent",
            &floats,
        )?;
        self.close(
            3 * 3600 - 900,
            &cents,
            "the total adds up to the cent",
            None,
        )?;

        let import = self.add(
            2 * 3600,
            &crm,
            None,
            "Import the contacts\nexpect: each contact of the export is in the list once",
            &[],
        )?;
        let csv = self.add(
            2 * 3600 - 60,
            &crm,
            Some(&import),
            "Read the CSV export\nexpect: the count of contacts read is the count of rows",
            &[],
        )?;
        self.close(2 * 3600 - 300, &csv, "1,204 contacts read", None)?;
        self.add(
            2 * 3600 - 320,
            &crm,
            Some(&import),
            "Remove the duplicate contacts\nexpect: no email is in the list twice",
            &[],
        )?;
        let sync = self.add(
            5000,
            &crm,
            None,
            "Sync with the old CRM\nexpect: a change in one shows in the other",
            &[],
        )?;
        self.trails(4900)?
            .drop(&sync, "the old CRM shuts down next week")?;
        let leads = self.add(
            900,
            &crm,
            None,
            "Merge the old leads into the contacts\nexpect: each old lead is a contact",
            &[],
        )?;
        let email = self.add(
            890,
            &crm,
            Some(&leads),
            "Link each lead to its contact\nexpect: no lead is without a contact",
            &[],
        )?;
        self.trails(120)?.start(&email)?;
        self.add(
            880,
            &crm,
            Some(&leads),
            "Move the notes of each lead to its contact\nexpect: each note shows on the contact",
            &[],
        )?;

        let tier = self.add(
            1500,
            &shop,
            None,
            "Add a senior tier to the price list\nexpect: a senior pays 80% on the shelf, the receipt and the frame",
            &[],
        )?;
        self.note(1490, &tier, "When you price a senior: pay 80% of the price, rounded down to the cent, because the receipt must match the shelf.")?;
        self.note(
            1480,
            &tier,
            "When you show a price: use the shop's currency, because the receipt showed dollars.",
        )?;
        let pays = self.add(
            1470,
            &shop,
            Some(&tier),
            "Charge a senior 80% of the price\nexpect: the price test passes",
            &["shelf/prices.py:40"],
        )?;
        self.close(
            1300,
            &pays,
            "the price test passes",
            Some("rounded down, not to the nearest cent"),
        )?;
        let receipt = self.add(
            1290,
            &shop,
            Some(&tier),
            "Show the tier on the receipt\nexpect: the receipt has a Senior line",
            &[],
        )?;
        let line = self.add(
            1280,
            &shop,
            Some(&receipt),
            "Add the tier's line to the receipt\nexpect: the receipt shows Senior -20%",
            &["till/receipt.py"],
        )?;
        self.close(1100, &line, "the receipt shows Senior -20%", None)?;
        let badge = self.add(
            1090,
            &shop,
            Some(&receipt),
            "Print the badge\nexpect: the badge prints",
            &[],
        )?;
        self.close(1000, &badge, "the badge prints", None)?;
        self.close(990, &receipt, "the receipt shows the tier", None)?;
        let banner = self.add(
            980,
            &shop,
            Some(&tier),
            "Keep the old discount banner\nexpect: the banner shows next to the tier",
            &[],
        )?;
        self.trails(970)?
            .drop(&banner, "the tier replaces the banner")?;
        let frame = self.add(
            960,
            &shop,
            Some(&tier),
            "Show the tier in the frame\nexpect: the frame shows the senior price",
            &["web/frame.tsx"],
        )?;
        let label = self.add(
            950,
            &shop,
            Some(&frame),
            "Add the badge to the frame\nexpect: the badge shows by the price",
            &[],
        )?;
        self.trails(300)?.start(&label)?;
        self.add(
            940,
            &shop,
            Some(&tier),
            "Round the price down\nexpect: no price has a part of a cent",
            &[],
        )?;
        Ok(())
    }

    /// Writes a trail of three steps each day from 27 days to 3 days ago, in turn in each project: the
    /// first uses a pattern, the others follow the steps of a week before and take less time. The later
    /// two weeks change, revise and take longer less often, and use patterns more.
    fn history(&self, shop: &Anchor, crm: &Anchor) -> Result<(), Failure> {
        let start = 28 * DAY;
        let mut patterns = vec![];
        for (anchor, text) in [
            (shop, "When you change a price: change the shelf label in the same step, because a shopper pays what the shelf says."),
            (shop, "When you test the till: run it with an empty cart first, because an empty cart broke the total twice."),
            (crm, "When you import contacts: match them by email before by name, because names repeat."),
            (crm, "When you change a contact field: keep the old value in its history, because sales asks for it."),
        ] {
            patterns.push((anchor, self.write_pattern(start, anchor, Scope::Project, text)?));
        }
        let mut days: Vec<[(String, u64); 3]> = vec![];
        for (i, day) in (3..28u64).rev().enumerate() {
            let recent = day < 14;
            let anchor = if i % 2 == 0 { shop } else { crm };
            let [first, second, third] = TASKS[i % TASKS.len()];
            let at = day * DAY + 3_600 * (i as u64 % 5);
            let trail = self.add(at, anchor, None, first, &[])?;
            let own: Vec<&String> = patterns
                .iter()
                .filter(|(a, _)| *a == anchor)
                .map(|(_, id)| id)
                .collect();
            let pattern = (recent || i % 3 == 0).then(|| own[i / 2 % own.len()].clone());
            let a = self.write_step(
                at - 60,
                anchor,
                Some(&trail),
                second,
                &[],
                None,
                pattern.as_slice(),
            )?;
            let took = 300 + (i as u64 * 137) % 1_500;
            let changed = match recent {
                true => i % 5 == 0,
                false => i % 2 == 0,
            };
            self.finish(
                at - 120 - took,
                took,
                &a,
                "it works",
                changed.then_some("it took a second try"),
            )?;
            let ago = days.len().checked_sub(7).map(|w| days[w].clone());
            let mut ids = vec![(a, took)];
            for (k, title) in [(1, third), (2, "Try it on the staging shop")] {
                let uses: Vec<String> = ago.iter().map(|w| w[k].0.clone()).collect();
                let id = self.write_step(
                    at - 180 - took,
                    anchor,
                    Some(&trail),
                    title,
                    &[],
                    None,
                    &uses,
                )?;
                let less = match (&ago, recent) {
                    (Some(w), true) => (w[k].1 / 2).max(60),
                    (Some(w), false) => (w[k].1 * 4 / 5).max(60),
                    (None, _) => took,
                };
                self.finish(at - 240 - took - less, less, &id, "it holds", None)?;
                ids.push((id, less));
            }
            let revise = match recent {
                true => i % 10 == 0,
                false => i % 4 == 0,
            };
            if let (true, Some(last)) = (revise, days.last()) {
                let fix = self.write_step(
                    at - 3_900,
                    anchor,
                    Some(&trail),
                    "Fix the last change",
                    &[],
                    Some(&last[2].0),
                    &[],
                )?;
                self.finish(at - 4_200, 240, &fix, "fixed", None)?;
            }
            let [a, b, c]: [(String, u64); 3] =
                ids.try_into().map_err(|_| anyhow::anyhow!("three steps"))?;
            days.push([a, b, c]);
        }
        Ok(())
    }

    /// The trails as they stand `ago` seconds before now, writing at that time.
    fn trails(&self, ago: u64) -> Result<RefMut<'_, Store>, Failure> {
        let mut trails = self.trails.borrow_mut();
        if trails.is_none() {
            *trails = Some(Store::open(Home::open(&self.home)?, self.now - ago, 600));
        }
        let mut trails = RefMut::map(trails, |t| t.as_mut().expect("opened"));
        trails.at(self.now - ago);
        Ok(trails)
    }

    /// Writes the step `text` of `anchor` at `places`, under `under` or as a trail, `ago` seconds before
    /// now; its id.
    fn add(
        &self,
        ago: u64,
        anchor: &Anchor,
        under: Option<&str>,
        text: &str,
        places: &[&str],
    ) -> Result<String, Failure> {
        self.write_step(ago, anchor, under, text, places, None, &[])
    }

    /// Writes the step `text` under `under` that replaces `old`, `ago` seconds before now; its id.
    fn replace(
        &self,
        ago: u64,
        anchor: &Anchor,
        under: &str,
        text: &str,
        old: &str,
    ) -> Result<String, Failure> {
        self.write_step(ago, anchor, Some(under), text, &[], Some(old), &[])
    }

    /// Writes a step as `add` and `replace` say, that uses `uses`; its id.
    #[allow(clippy::too_many_arguments)]
    fn write_step(
        &self,
        ago: u64,
        anchor: &Anchor,
        under: Option<&str>,
        text: &str,
        places: &[&str],
        replaces: Option<&str>,
        uses: &[String],
    ) -> Result<String, Failure> {
        let draft = Draft {
            content: text.to_string(),
            anchor: anchor.clone(),
            places: places.iter().map(|p| p.to_string()).collect(),
            check: None,
            under: under.map(str::to_string),
            replaces: replaces.map(str::to_string),
            uses: uses.to_vec(),
            session: None,
            new: false,
        };
        let trails = self.trails(ago)?;
        let written = trails.add(draft)?;
        Ok(written.records[0].id.clone())
    }

    /// Starts the step `id` 60 seconds before `ago`, then closes it done with `proof` and what changed,
    /// `ago` seconds before now.
    fn close(&self, ago: u64, id: &str, proof: &str, changed: Option<&str>) -> Result<(), Failure> {
        self.finish(ago, 60, id, proof, changed)
    }

    /// Starts the step `id` `took` seconds before `ago`, then closes it done with `proof` and what
    /// changed, `ago` seconds before now.
    fn finish(
        &self,
        ago: u64,
        took: u64,
        id: &str,
        proof: &str,
        changed: Option<&str>,
    ) -> Result<(), Failure> {
        self.trails(ago + took)?.start(id)?;
        let changed = changed.map(str::to_string);
        self.trails(ago)?.done(
            id,
            celis_trails::Done::new(proof.to_string(), changed),
            &self.home,
        )?;
        Ok(())
    }

    /// Writes the note `text` on the trail `trail`, `ago` seconds before now.
    fn note(&self, ago: u64, trail: &str, text: &str) -> Result<(), Failure> {
        self.trails(ago)?.note(trail, text.to_string())?;
        Ok(())
    }

    /// Writes the pattern `text` of `anchor` in `scope`, `ago` seconds before now; its id.
    fn write_pattern(
        &self,
        ago: u64,
        anchor: &Anchor,
        scope: Scope,
        text: &str,
    ) -> Result<String, Failure> {
        let pattern = PatternRecord {
            anchor: anchor.clone(),
            scope,
            replaces: None,
            content: text.to_string(),
            paths: vec![],
            old_rule: false,
            within: vec![],
            cells: vec![],
            examples: Default::default(),
        };
        let trails = self.trails(ago)?;
        Ok(trails.decide(pattern, None)?.id)
    }
}

/// The seconds of a day.
const DAY: u64 = 24 * 3600;

/// The trail, the first step and the second step of each day of the history.
const TASKS: [[&str; 3]; 6] = [
    [
        "Add gift cards",
        "Keep the card balance",
        "Take a card at the till",
    ],
    [
        "Clean up old contacts",
        "Find the contacts with no email",
        "Archive those contacts",
    ],
    [
        "Show stock on the shelf page",
        "Read the stock count",
        "Show it by the price",
    ],
    [
        "Add a birthday field",
        "Add the field to the form",
        "Show it on the contact card",
    ],
    [
        "Speed up the checkout",
        "Work out the cart total once",
        "Look up each price once",
    ],
    [
        "Tag the leads by source",
        "Give each lead its source",
        "Filter the list by tag",
    ],
];

/// The anchor of the made-up project `name`.
fn anchor(name: &str) -> Anchor {
    Anchor {
        project: format!("demo-{name}"),
        name: name.to_string(),
    }
}

mod graph;
mod month;
