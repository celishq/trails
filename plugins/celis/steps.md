# Steps

The cases that the card does not cover, and that no command prints. Do not run `--help`.

## Rules

A place is a file, a `file:line`, a folder, or a URL. A rule that holds at one place takes the scope
`at:<place>`.

When you move a file: write each rule of its old place again for the new place, with
`decide --replaces <id>` and the new place as its scope or `--path`. Run `celis rules <old place>` to
see them.

## Past work

When the `celis:past` agent returns past work that did the same thing as your ask and went as planned,
take the steps right under its ask as yours, with the `changed` lines in mind. Copy the steps under a
step only when you take that step. A past step that does the same thing in other words is the same
step. When its title names its own case, write it in words that hold in yours. Add each step you take
with `--uses <its id>`, so that later work can see which steps were reused.

## Named trails

A step that uses a named trail keeps its rules, and the rules of each named trail that it uses.

- When the person names a named trail, or runs `/celis:use <name>`, add the ask with `--uses <name>`.
- When the person wants a named trail's rule to work another way in this project, write the new rule
  with `decide --replaces <its id>`. It holds in this project only, and the trail keeps its rule
  elsewhere.
- When celis says that a rule ends an override, tell the person. They choose whether to write the
  override again with `decide --replaces <the new id>` in its project.
- When a done trail did work that holds in other projects, and it has no name, give it one with
  `celis name <trail id> <name>`: a short noun in lowercase words joined by `-`, such as `code-review`.

To write a named trail `<name>` that work in every project can use:

1. `celis step add --project skill:<name> --new [--uses <id>]... -- "<what the work does>"`. Its ask and
   its trail print.
2. For each step, in order: `celis step add --project skill:<name> --under <ask id> -- "<title>"`, then
   `celis step done <its id> --as-planned -- "<where it comes from>"`.
3. For each rule: `celis decide --project skill:<name> --scope trail:<trail id> -- "<rule>"`.
4. `celis step done <ask id> --as-planned -- "<what it was made from>"`, then
   `celis name <trail id> <name>`.

## Tidy

When a search brings rules that say the same thing or say to do different things, or many stale trails,
tell the person that `/celis:tidy` tidies them.

## Subagents and other sessions

When a subagent or another agent takes work, add a step for it under your working step. Put in its task
each rule of that work, as celis printed it, and: "celis step <id>: run celis step start <id> first and
celis step done <id> last." `start` prints the rules at the step's places, and the agent adds its own
steps `--under <id>`. To go on with another session's trail, add your first step `--under <its id>`.

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
