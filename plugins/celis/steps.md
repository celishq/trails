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
