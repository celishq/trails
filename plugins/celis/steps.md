# Steps

The cases that the card does not cover, and that no command prints. Do not run `--help`.

## Decisions

A place is a file, a `file:line`, a folder, or a URL. A rule that holds at one place takes the scope
`at:<place>`.

When you move a file: write each decision at its old place again at the new place, with
`decide --replaces <id> --scope at:<new place>`. Run `celis grep "at:<old place>"` to see them.

## Past work

When the `celis:past` agent returns past work that went as planned, take the steps right under its ask
as your shapes, with the `changed` lines in mind. Copy the steps under a shape only when you take that
shape. Else outline the shapes yourself. A part that several shapes use is its own shape, first. Add
each step you take from past work with `--uses <its id>`, so later work can see which paths were
followed.

## Subagents and other sessions

When a subagent takes a step, give it the step's id. It runs `start` and `done` on that step, and adds
its own steps `--under` it. To go on with another session's trail, add your first step `--under <its
id>`.

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
