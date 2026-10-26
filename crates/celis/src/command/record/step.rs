//! `celis step`: leave, close and drop steps.

use crate::command::record::close::Review;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_store::records::id::short;
use celis_store::{Link, Place, Project};
use celis_trails::{By, Done, Draft, Notice, TrailStore, Written};
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
            "note" => self.step_note(args, out),
            "" => Err(Failure::usage("expected: celis step <command>")),
            verb => Err(Failure::usage(format!("no command \"step {verb}\""))),
        }
    }

    /// Writes the step `args` states on `project`, and prints the ids `TrailStore::add` gives.
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
            recalled: args.recalled.clone(),
            session: args.session.clone().or_else(|| self.session()),
        };
        let session = draft.session.clone();
        let trails = self.trails()?;
        let written = trails.add(draft)?;
        if let (Some(session), Some(step)) = (session, written.records.first()) {
            self.place_kept(&session, &step.id)?;
        }
        self.print(written, args, out, err)
    }

    /// Starts the open step `args` names, and prints the id of its start, then the patterns and notes
    /// its plan links.
    fn step_start(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let id = args.id("step start")?;
        let trails = self.trails()?;
        let written = trails.start(id)?;
        self.print(written, args, out, err)
    }

    /// Closes the open step `args` names, done with its proof, what changed from its plan, who gave the
    /// proof and the files it keeps, and prints the id of the done record and its review.
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
        let changed = self.changed(&args.files);
        let steps = trails.steps();
        let step = steps.find_open(id)?;
        if done.by.is_none() && step.is_judged() {
            let lines = self.unjudged(step, &trails, &changed);
            if !lines.is_empty() {
                lines
                    .iter()
                    .try_for_each(|line| crate::command::put(err, line))?;
                return Err(Failure::unjudged(step.id()));
            }
        }
        let written = trails.done(id, done.clone(), &project.folder)?;
        self.print(written, args, out, err)?;
        let steps = trails.steps();
        let review = Review::of(
            trails.records(),
            steps.find(id)?,
            &done,
            crate::command::now(),
        );
        let linked = !Link::all(&self.home()?, &project.folder).is_empty();
        self.review(&review, (&trails, &changed), linked, err)
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

    /// Writes the text after `--` as a note on the step `args` names, and prints the note's id.
    fn step_note(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.id("step note")?;
        let trails = self.trails()?;
        let text = args.required_text("step note needs its text after --")?;
        self.print(trails.note(id, text)?, args, out, &mut std::io::sink())
    }

    /// Prints the id of each record of `written`, each of its patterns and each of its notes on `out`, and
    /// its notices on `err`.
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
        if !written.patterns.is_empty() {
            let trails = self.trails()?;
            for pattern in &written.patterns {
                let line = crate::text::line::pattern(pattern, &trails);
                let json = crate::text::json::pattern(pattern);
                crate::command::show(out, args.json, "pattern", json, &line)?;
            }
        }
        for note in &written.notes {
            let (line, json) = (crate::text::line::note(note), crate::text::json::note(note));
            crate::command::show(out, args.json, "note", json, &line)?;
        }
        for notice in &written.notices {
            self.notice(notice, err)?;
        }
        Ok(())
    }

    fn notice(&self, notice: &Notice, err: &mut dyn Write) -> Result<(), Failure> {
        let line = match notice {
            Notice::Ask(ask) => format!(
                "celis: when {0} holds more than one job, a part a person can check alone such as an item of its list, \
                 add only its first job with step add --under {0}, and each next job after you close one.",
                short(ask)
            ),
            Notice::NotStarted(step) => format!(
                "celis: {} has not started; keep one step ahead: take it first, or drop it.",
                short(step)
            ),
        };
        crate::command::put(err, &line)
    }
}
