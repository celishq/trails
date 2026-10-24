# Steps

The cases that the card does not cover, and that no command prints. Do not run `--help`.

## Rules

A place is a file, a `file:line`, a folder, or a URL. A rule that holds at one place takes the scope
`at:<place>`.

To find more rules than the past agent brought, run `celis rules` for the words the rules use, and
search them with `celis grep -i -E " rule [0-9a-f]{8} live .*(<word>|<word>)"`.

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

- When the person names a named trail, add the ask with `--uses <name>`.
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

## Other sessions

To go on with another session's trail, add your first step `--under <its id>`.

## Taste

A taste block is a rule with a good and a bad step, which `celis rules <file>` gives with their proofs
and blobs: an example shows what a rule means better than more words. In Build it binds as any rule;
in creative work it guides, and a variant that breaks it on purpose keeps it. Keep a block from:

- a pick: `decide --good <pick> --bad <the other> --path <the file>`, in the person's words, or
  "because the person picked it" when they gave no reason; never your own choice;
- a correction: the hunk it replaced and the fix in the fix's proof, a few lines each, and
  `--good <the fix>`;
- the person's own rewrite, as `git log` shows at the next ask: a step that `--replaces` yours, closed
  `--by person` with both hunks.

When 3 blocks make the same point, or the picks of a creative ask share one, propose one rule that says
it, and replace them when the person agrees.

## Links

When the person wants to keep the trails in the repo or in a folder, tell them to run
`celis link [<folder>]` in the project themselves.

## Stop

Never write a proof for work you did not do. Stop and tell the person what is open when two reviews in
a row send you back to the same step, or at a limit of time or money the person set. When you lose track,
or after a compaction, run `celis tree`, and go on.
