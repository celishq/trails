# Steps

The cases that the card does not cover, and that no command prints. Do not run `--help`.

## Rules

A place is a file, a `file:line`, a folder, or a URL. A rule that holds at one place takes the scope
`at:<place>`.

When you move a file: write each rule at its old place again at the new place, with
`decide --replaces <id> --scope at:<new place>`. Run `celis grep "at:<old place>"` to see them.

## Past work

When the `celis:past` agent returns past work that did the same thing as your ask and went as planned,
take the steps right under its ask as yours, with the `changed` lines in mind. Copy the steps under a
step only when you take that step. A past step that does the same thing in other words is the same
step. When its title names its own case, write it in words that hold in yours. Add each step you take
with `--uses <its id>`, so that later work can see which steps were reused.

## Subagents and other sessions

When a subagent or another agent takes work, add a step for it under your working step, and put in its
task: "celis step <id>: run celis step start <id> first and celis step done <id> last." `start` prints
the rules it must keep, and the agent adds its own steps `--under <id>`. To go on with another session's
trail, add your first step `--under <its id>`.

## Links

A project can keep its trails in `.celis/trails/`, so that git carries them to the team, or in any
folder, such as a synced drive. When celis says the project has trails this machine does not link, ask
the person whether to link this project to celis. When the person wants to keep the trails in the repo
or in a folder, tell them to run `celis link [<folder>]` in the project themselves. Never run
`celis link` or `celis unlink` yourself, and never write in a linked folder.

## Stop

Never write a proof for work you did not do. Stop and tell the person what is open when two reviews in
a row send you back to the same step, or at a limit of time or money the person set. When you lose track,
or after a compaction, run `celis tree`, and go on.
