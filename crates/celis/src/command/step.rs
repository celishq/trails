//! `celis step`: leave, close and drop steps.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Review;
use celis_trails::records::id::short;
use celis_trails::{Draft, Notice, Place, Project, Written};
use std::io::Write;

/// What an agent does at a new ask, before its first step.
const ASK: &str = "celis: find the rules of this ask first: celis rules prints the words the rules use. \
Search the rules with the words that fit the ask: celis grep -i -E \" rule [0-9a-f]{8} live .*(<word>|<word>)\". \
Keep the {me} rules and those of this project.";

/// What an agent does while a step works at `places`: get the rules before each change, and check the
/// change against them before done.
fn work(places: &[String]) -> String {
    let files: Vec<&str> = places
        .iter()
        .filter(|p| !p.contains("://"))
        .map(|p| p.split(':').next().unwrap_or(p))
        .collect();
    let named = match files.is_empty() {
        true => "<files>".to_string(),
        false => format!("{} <other files>", files.join(" ")),
    };
    format!(
        "celis: before each change, run celis rules {named} -- \"<command>\", and keep each rule. \
         Before done, run celis rules on each file you changed, and fix your change where it breaks a rule."
    )
}

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
        let first = written.records.first().map(|n| n.id.clone());
        self.print(written, args, out, err)?;
        let trails = self.trails()?;
        let steps = trails.steps();
        match first.and_then(|id| steps.find(&id).ok()) {
            Some(step) if step.is_trail() || step.is_ask() => super::put(err, ASK),
            _ => Ok(()),
        }
    }

    /// Starts the open step `args` names, and prints the id of its start, the rules its plan links,
    /// then how to find and check the rules of its work.
    fn step_start(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let id = args.id("step start")?;
        let trails = self.trails()?;
        let written = trails.start(id)?;
        self.print(written, args, out, err)?;
        let steps = trails.steps();
        let places = &steps.find(id)?.record().places;
        super::put(err, &work(places))
    }

    /// Closes the open step `args` names, done with its proof and what changed from its plan, and prints
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
        let pitfall = args.pitfall.clone();
        let written = trails.done(id, proof, changed.clone(), pitfall, &project.folder)?;
        self.print(written, args, out, err)?;
        let steps = trails.steps();
        let review = Review::of(trails.records(), steps.find(id)?, changed.as_deref());
        self.review(&review, err)
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
        for (said, rule) in &written.rules {
            let line = crate::text::line::rule(said, rule);
            super::show(
                out,
                args.json,
                "decision",
                crate::text::json::rule(said, rule),
                &line,
            )?;
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
