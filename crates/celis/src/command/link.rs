//! `celis link` and `celis unlink`: keep the project's trails in a folder.

use super::{Args, Command};
use celis_trails::{Failure, Link, Project, Store};
use serde_json::json;
use std::io::Write;

impl Command {
    /// Links `project` to the folder `args` names, else to `.celis/trails`, syncs it with the store, and
    /// prints the counts copied out and in.
    pub(super) fn link(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let home = self.home()?;
        if let Some(given) = args.words.get(1) {
            std::fs::create_dir_all(self.folder.join(given)).map_err(anyhow::Error::from)?;
        }
        let link = Link::new(project, &self.place(args, project));
        link.turn_on(&home)?;
        let place = link.place.clone();
        let (went, came) = Store::open(&home)?.with(link).sync()?;
        let value = json!({"folder": place, "out": went, "in": came});
        let line = format!("{place} · {went} out · {came} in");
        super::put(out, &if args.json { value.to_string() } else { line })
    }

    /// Takes the link of `project` to the folder `args` names, else to `.celis/trails`, out of
    /// `$CELIS_HOME/links`; the folder stays.
    pub(super) fn unlink(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let link = Link::new(project, &self.place(args, project));
        link.turn_off(&self.home()?)?;
        let value = json!({"folder": link.place, "unlinked": true});
        let line = format!("{} · unlinked", link.place);
        super::put(out, &if args.json { value.to_string() } else { line })
    }

    /// The folder `args` names, from the project's top folder when it lies in it, else as a full path.
    fn place(&self, args: &Args, project: &Project) -> String {
        match args.words.get(1) {
            Some(given) => {
                project.place(&self.folder.join(given).display().to_string(), &self.folder)
            }
            None => Link::FOLDER.to_string(),
        }
    }
}
