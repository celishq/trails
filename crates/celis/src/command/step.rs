//! `celis step`: leave, close and drop steps.

use crate::command::close::Review;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::records::id::short;
use celis_trails::{By, Done, Draft, Link, Notice, Place, Project, Written};
use std::io::Write;

impl Command {
    /// Runs `step <verb>` with `args`, writing to `out` and notes to `err`.
    pub(crate) fn step(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        match args.words.get(1).map(String::as_str).unwrap_or("") {
            "add" => self.step_add(args, out, err, project),
            "start" => self.step_start(args, out, err),
            "done" => self.step_close(args, out, err, project),
            "drop" => self.step_drop(args, out),
            "edit" => self.step_edit(args, out, project),
            "" => Err(Failure::usage("expected: celis step <command>")),
            verb => Err(Failure::usage(format!("no command \"step {verb}\""))),
        }
    }

    /// Writes the step `args` states on `project`, and prints the ids `Trails::add` gives.
    fn step_add(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let content = args.required_text("step add needs its text after --")?;
        let anchor = match args.places.first() {
            Some(at) if args.project.is_none() => project.of_place(at).anchor,
            _ => project.anchor.clone(),
        };
        let draft = Draft {
            content,
            anchor,
            places: args
                .places
                .iter()
                .map(|p| Place::resolve(p, &project.folder, &self.folder))
                .collect(),
            check: args.check.clone(),
            under: args.under.clone(),
            replaces: args.replaces.clone(),
            uses: args.uses.clone(),
            session: args.session.clone().or_else(|| self.session()),
            new: args.new,
        };
        let written = self.trails()?.add(draft)?;
        self.print(written, args, out, err)
    }

    /// Starts the open step `args` names, and prints the id of its start, then the rules its plan
    /// links.
    fn step_start(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let id = args.id("step start")?;
        let written = self.trails()?.start(id)?;
        self.print(written, args, out, err)
    }

    /// Closes the open step `args` names, done with its proof, what changed from its plan, who gave the
    /// proof and the files it keeps, and prints
    /// the id of the done record, then of its pitfall's rule, and its review.
    fn step_close(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let id = args.id("step done")?;
        let trails = self.trails()?;
        let proof = args.required_text("step done needs its proof after --")?;
        let changed =
            match (&args.changed, args.planned) {
                (Some(changed), false) if !changed.trim().is_empty() => Some(changed.clone()),
                (None, true) => None,
                _ => return Err(Failure::usage(
                    "step done needs one of --changed <what differs from the plan> or --as-planned",
                )),
            };
        let by = match args.by.as_deref() {
            None => None,
            Some(text) => Some(By::parse(text).ok_or_else(|| {
                Failure::usage(format!("step done --by takes judge or person, not {text}"))
            })?),
        };
        let mut files = Vec::new();
        for file in &args.files {
            let place = Place::resolve(file, &project.folder, &self.folder);
            files.push(trails.keep(place, &self.folder.join(file))?);
        }
        let done = Done {
            proof,
            changed,
            by,
            files,
        };
        let pitfall = args.pitfall.clone();
        let written = trails.done(id, done.clone(), pitfall, &project.folder)?;
        self.print(written, args, out, err)?;
        let steps = trails.steps();
        let review = Review::of(trails.records(), steps.find(id)?, &done, super::now());
        let linked = !Link::all(&self.home()?, &project.folder).is_empty();
        self.review(&review, &trails, linked, err)
    }

    /// Drops the open step `args` names and each open step under it, and prints each drop's id.
    fn step_drop(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.id("step drop")?;
        let trails = self.trails()?;
        let reason = args.required_text("step drop needs its reason after --")?;
        self.print(trails.drop(id, &reason)?, args, out, &mut std::io::sink())
    }

    /// Gives the step `args` names the text after `--`, its `--at` places or its `--check`, and prints the
    /// id of the edit.
    fn step_edit(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let id = args.id("step edit")?;
        let trails = self.trails()?;
        let text = args.text.clone().unwrap_or_default();
        if text.trim().is_empty() && args.places.is_empty() && args.check.is_none() {
            return Err(Failure::usage(
                "step edit needs --check, --at or its text after --",
            ));
        }
        let places = args
            .places
            .iter()
            .map(|p| Place::resolve(p, &project.folder, &self.folder))
            .collect();
        let written = trails.edit(id, text, places, args.check.clone())?;
        self.print(written, args, out, &mut std::io::sink())
    }

    /// Prints the id of each record of `written` and each of its rules on `out`, and its notices on
    /// `err`.
    pub(crate) fn print(
        &self,
        written: Written,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        for record in &written.records {
            self.id(out, args, record)?;
        }
        if !written.rules.is_empty() {
            let trails = self.trails()?;
            for (said, rule) in &written.rules {
                let line = crate::text::line::rule(said, rule, &trails);
                let json = crate::text::json::rule(said, rule);
                super::show(out, args.json, "decision", json, &line)?;
            }
        }
        for notice in &written.notices {
            self.notice(notice, err)?;
        }
        Ok(())
    }

    fn notice(&self, notice: &Notice, err: &mut dyn Write) -> Result<(), Failure> {
        let line = match notice {
            Notice::Started { ask, trail } => format!(
                "celis: {} is the ask, under its new trail {}.",
                short(ask),
                short(trail)
            ),
            Notice::Ask(ask) => format!(
                "celis: when {0} holds more than one job, a part a person can check alone such as an item of its list, \
                 add only its first job with step add --under {0}, and each next job after you close one.",
                short(ask)
            ),
            Notice::Joined(trail) => format!(
                "celis: the step joined the trail {}; edit its title when it no longer covers the work.",
                short(trail)
            ),
            Notice::NotStarted(step) => format!(
                "celis: {} has not started; keep one step ahead: take it first, or drop it.",
                short(step)
            ),
            Notice::Ended { rule, project } => format!(
                "celis: this ends the override {} of [{project}]; tell the person, and write it again \
                 there with decide --replaces when it still holds.",
                short(rule)
            ),
        };
        super::put(err, &line)
    }
}
