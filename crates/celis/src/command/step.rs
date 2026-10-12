//! `celis step`: leave, steer, close and drop steps.

use super::{Args, Command};
use celis_trails::id::short;
use celis_trails::{Direction, Draft, Failure, Notice, Project, Written};
use std::io::Write;

impl Command {
    /// Runs `step <verb>` with `args`, writing to `out` and notes to `err`.
    pub(super) fn step(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        match args.words.get(1).map(String::as_str).unwrap_or("") {
            "add" => self.step_add(args, out, err, project),
            "start" => self.step_start(args, out, err, project),
            "done" => self.step_close(args, out, err, project),
            "steer" => self.step_steer(args, out),
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
            Some(at) => project.of_place(at).anchor(),
            None => project.anchor(),
        };
        let draft = Draft {
            content,
            anchor,
            places: args
                .places
                .iter()
                .map(|p| project.place(p, &self.folder))
                .collect(),
            check: args.check.clone(),
            under: args.under.clone(),
            replaces: args.replaces.clone(),
            fixes: args.fixes.clone(),
            session: self.session(),
            trail: self.env.get("CELIS_TRAIL").cloned(),
            new: args.new,
        };
        let written = self.trails()?.add(draft)?;
        self.print(written, args, out, err)
    }

    /// Starts the open step `args` names, and prints the id of its start, then the decisions at its
    /// places.
    fn step_start(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let id = args.id("step start")?;
        let written = self.trails()?.start(id, &project.folder)?;
        self.print(written, args, out, err)
    }

    /// Closes the open step `args` names, done with its proof and what changed from its plan, and prints
    /// the id of the done record, then of its pitfall's decision.
    fn step_close(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let id = args.id("step done")?;
        let trails = self.trails()?;
        trails.steps().find_open(id)?;
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
        let written = trails.done(id, proof, changed, pitfall, &project.folder)?;
        self.print(written, args, out, err)
    }

    /// Writes what the person asked for while the open step `args` names was open, and prints its id.
    fn step_steer(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.id("step steer")?;
        let trails = self.trails()?;
        trails.steps().find_open(id)?;
        let text = args.required_text("step steer needs what the person asked after --")?;
        self.print(trails.steer(id, text)?, args, out, &mut std::io::sink())
    }

    /// Drops the open step `args` names and each open step under it, and prints each drop's id.
    fn step_drop(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.id("step drop")?;
        let trails = self.trails()?;
        trails.steps().find_open(id)?;
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
        trails.steps().find(id)?;
        let text = args.text.clone().unwrap_or_default();
        if text.trim().is_empty() && args.places.is_empty() && args.check.is_none() {
            return Err(Failure::usage(
                "step edit needs --check, --at or its text after --",
            ));
        }
        let places = args
            .places
            .iter()
            .map(|p| project.place(p, &self.folder))
            .collect();
        let written = trails.edit(id, text, places, args.check.clone())?;
        self.print(written, args, out, &mut std::io::sink())
    }

    /// Prints the id of each record of `written` and each of its decisions on `out`, and its notices on
    /// `err`.
    pub(super) fn print(
        &self,
        written: Written,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        for record in &written.records {
            self.id(out, args, record)?;
        }
        for (said, decision) in &written.decisions {
            let line = decision.line(said);
            super::show(out, args.json, "decision", decision.json(said), &line)?;
        }
        for notice in &written.notices {
            self.notice(notice, err)?;
        }
        Ok(())
    }

    fn notice(&self, notice: &Notice, err: &mut dyn Write) -> Result<(), Failure> {
        let line = match notice {
            Notice::LongTitle(words) => format!(
                "celis: the title has {words} words; keep it to 8 or fewer with step edit, and put the detail on the lines after it."
            ),
            Notice::Joined(trail) => format!(
                "celis: the step joined the trail {}; edit its title when it no longer covers the work.",
                short(trail)
            ),
            Notice::CheckPassesBefore(step) => format!(
                "celis: the check of {} passes before the work, so it cannot prove it; give a check that fails now with step edit --check.",
                short(step)
            ),
            Notice::Retro(retro) => return self.retro(retro, err),
            Notice::Direction(direction) => return direction_lines(direction, err),
        };
        super::put(err, &line)
    }
}

/// Prints on `err` that a direction check is due, then the ask, whether its check passes, its open
/// steps, and a line that asks to drop or replace each that does not serve it.
fn direction_lines(direction: &Direction, err: &mut dyn Write) -> Result<(), Failure> {
    let (n, trail) = (direction.done, short(&direction.trail));
    super::put(
        err,
        &format!("celis: {n} steps of the trail {trail} are done; check its direction."),
    )?;
    let Some((ask, title)) = &direction.ask else {
        return Ok(());
    };
    super::put(err, &format!("  ask  {} {title}", short(ask)))?;
    match &direction.check {
        Some(Ok(())) => super::put(err, "  check passes")?,
        Some(Err(last)) => super::put(err, &format!("  check fails: {last}"))?,
        None => {}
    }
    for (id, title) in &direction.open {
        super::put(err, &format!("  open {} {title}", short(id)))?;
    }
    super::put(
        err,
        "celis: drop or replace each open step that does not serve the ask.",
    )
}
