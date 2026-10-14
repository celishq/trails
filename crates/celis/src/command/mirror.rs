//! `celis mirror`: keep the project's trails in its own folder.

use super::{Args, Command};
use celis_trails::{Failure, Mirror, Project, Store};
use serde_json::json;
use std::io::Write;

/// Where the mirror is, from the project's top folder.
const FOLDER: &str = ".celis/trails";

impl Command {
    /// Turns on the mirror of `project`, syncs it with the store, and prints the counts copied out and in;
    /// with `--off`, turns it off and keeps the folder.
    pub(super) fn mirror(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let home = self.home()?;
        let mirror = Mirror::of(project);
        if args.off {
            mirror.turn_off(&home)?;
            let value = json!({"folder": FOLDER, "off": true});
            return match args.json {
                true => super::put(out, &value.to_string()),
                false => super::put(out, &format!("{FOLDER} · off")),
            };
        }
        mirror.turn_on(&home)?;
        let (went, came) = Store::open(&home)?.with(mirror).sync()?;
        let value = json!({"folder": FOLDER, "out": went, "in": came});
        let line = format!("{FOLDER} · {went} out · {came} in");
        match args.json {
            true => super::put(out, &value.to_string()),
            false => super::put(out, &line),
        }
    }
}
