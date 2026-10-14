//! `celis tree --demo`: made-up trails in every state, in a folder of their own.

use super::{Args, Command};
use celis_trails::{Anchor, Decision, Draft, Failure, Scope, Store, Trails};
use std::io::Write;
use std::path::PathBuf;

impl Command {
    /// Prints or watches, as `tree` does, made-up trails written to a folder of their own, and removes
    /// the folder at the end.
    pub(super) fn demo(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let home = std::env::temp_dir().join(format!("celis-demo-{}", std::process::id()));
        let demo = Demo {
            home: home.clone(),
            now: super::now(),
        };
        let shown = demo.write().and_then(|()| {
            let mut env = self.env.clone();
            env.insert("CELIS_HOME".to_string(), home.display().to_string());
            let command = Command {
                args: self.args.clone(),
                env,
                folder: self.folder.clone(),
            };
            let args = Args {
                demo: false,
                ..args.clone()
            };
            command.tree(&args, out)
        });
        let _ = std::fs::remove_dir_all(&home);
        shown
    }
}

/// The folder the made-up trails are written to, and the time they lead up to.
struct Demo {
    home: PathBuf,
    now: u64,
}

impl Demo {
    /// Writes the made-up trails: a done one of a past day; today, an active one with a step in each
    /// state, rules and a change, a revised one, and in a second project an active, an expired and a
    /// dropped one.
    fn write(&self) -> Result<(), Failure> {
        let (shop, crm) = (anchor("shop"), anchor("crm"));
        let day = 2 * 24 * 3600;
        let set = self.add(day, &shop, None, "Set up the shop", &[])?;
        let till = self.add(day - 60, &shop, Some(&set), "Open the till", &[])?;
        self.close(day - 600, &till, "the till opens", None)?;

        let cart = self.add(3 * 3600, &shop, None, "Fix the cart total", &[])?;
        let floats = self.add(
            3 * 3600 - 60,
            &shop,
            Some(&cart),
            "Sum the cart in floats",
            &["shelf/cart.py"],
        )?;
        self.close(3 * 3600 - 400, &floats, "the total adds up", None)?;
        let cents = self.replace(
            3 * 3600 - 500,
            &shop,
            &cart,
            "Sum the cart in cents",
            &floats,
        )?;
        self.close(
            3 * 3600 - 900,
            &cents,
            "the total adds up to the cent",
            None,
        )?;

        let import = self.add(2 * 3600, &crm, None, "Import the contacts", &[])?;
        let csv = self.add(
            2 * 3600 - 60,
            &crm,
            Some(&import),
            "Read the CSV export",
            &[],
        )?;
        self.close(2 * 3600 - 300, &csv, "1,204 contacts read", None)?;
        self.add(
            2 * 3600 - 320,
            &crm,
            Some(&import),
            "Merge the duplicates",
            &[],
        )?;
        let sync = self.add(5000, &crm, None, "Sync with the old CRM", &[])?;
        self.trails(4900)?
            .drop(&sync, "the old CRM shuts down next week")?;
        let leads = self.add(
            900,
            &crm,
            None,
            "Merge the old leads into the contacts",
            &[],
        )?;
        let email = self.add(890, &crm, Some(&leads), "Match the leads by email", &[])?;
        self.trails(120)?.start(&email)?;
        self.add(880, &crm, Some(&leads), "Move the notes over", &[])?;

        let tier = self.add(
            1500,
            &shop,
            None,
            "Add a senior tier to the price list",
            &[],
        )?;
        self.rule(1490, &shop, &tier, "When you price a senior: pay 80% of the price, rounded down to the cent, because the receipt must match the shelf.")?;
        self.rule(
            1480,
            &shop,
            &tier,
            "When you show a price: use the shop's currency, because the receipt showed dollars.",
        )?;
        let pays = self.add(
            1470,
            &shop,
            Some(&tier),
            "A senior pays 80%",
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
            "Show the tier on the receipt",
            &[],
        )?;
        let line = self.add(
            1280,
            &shop,
            Some(&receipt),
            "Add the tier's line",
            &["till/receipt.py"],
        )?;
        self.close(1100, &line, "the receipt shows Senior -20%", None)?;
        let badge = self.add(1090, &shop, Some(&receipt), "Print the badge", &[])?;
        self.close(1000, &badge, "the badge prints", None)?;
        self.close(990, &receipt, "the receipt shows the tier", None)?;
        let banner = self.add(980, &shop, Some(&tier), "Keep the old discount banner", &[])?;
        self.trails(970)?
            .drop(&banner, "the tier replaces the banner")?;
        let frame = self.add(
            960,
            &shop,
            Some(&tier),
            "The frame shows the tier",
            &["web/frame.tsx"],
        )?;
        let label = self.add(950, &shop, Some(&frame), "Add the badge to the frame", &[])?;
        self.trails(300)?.start(&label)?;
        self.add(940, &shop, Some(&tier), "Round the price down", &[])?;
        Ok(())
    }

    /// The trails as they stand `ago` seconds before now, writing at that time.
    fn trails(&self, ago: u64) -> Result<Trails, Failure> {
        Ok(Trails::open(Store::open(&self.home)?, self.now - ago, 600))
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
        self.write_step(ago, anchor, under, text, places, None)
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
        self.write_step(ago, anchor, Some(under), text, &[], Some(old))
    }

    /// Writes a step as `add` and `replace` say; its id.
    fn write_step(
        &self,
        ago: u64,
        anchor: &Anchor,
        under: Option<&str>,
        text: &str,
        places: &[&str],
        replaces: Option<&str>,
    ) -> Result<String, Failure> {
        let draft = Draft {
            content: text.to_string(),
            anchor: anchor.clone(),
            places: places.iter().map(|p| p.to_string()).collect(),
            check: None,
            under: under.map(str::to_string),
            replaces: replaces.map(str::to_string),
            uses: vec![],
            session: None,
            new: false,
        };
        let written = self.trails(ago)?.add(draft)?;
        Ok(written.records[0].id.clone())
    }

    /// Starts the step `id` 60 seconds before `ago`, then closes it done with `proof` and what changed,
    /// `ago` seconds before now.
    fn close(&self, ago: u64, id: &str, proof: &str, changed: Option<&str>) -> Result<(), Failure> {
        self.trails(ago + 60)?.start(id)?;
        let changed = changed.map(str::to_string);
        self.trails(ago)?
            .done(id, proof.to_string(), changed, None, &self.home)?;
        Ok(())
    }

    /// Writes the rule `text` of the trail `trail`, `ago` seconds before now.
    fn rule(&self, ago: u64, anchor: &Anchor, trail: &str, text: &str) -> Result<(), Failure> {
        let decision = Decision {
            anchor: anchor.clone(),
            scope: Scope::Trail(trail.to_string()),
            replaces: None,
            content: text.to_string(),
        };
        self.trails(ago)?.decide(decision, None)?;
        Ok(())
    }
}

/// The anchor of the made-up project `name`.
fn anchor(name: &str) -> Anchor {
    Anchor {
        project: format!("demo-{name}"),
        name: name.to_string(),
    }
}
