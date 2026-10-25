# Steps

The cases that the card does not cover, and that no command prints. Do not run `--help`.

## Rules

A rule that holds at one file or folder takes it as `--path`, such as `shelf/prices.py` or
`shelf/**`.

To find more rules than `celis search` brought, search them with
`celis grep -i -E " rule [0-9a-f]{8} live .*(<word>|<word>)"`.

When you move a file: write each rule of its old place again for the new place, with
`decide --replaces <id>` and the new place as its `--path`. Run
`celis search <old place> --kind rule -- "move"` to see them.

## Past work

When `celis search` brings a past ask that did the same thing as your ask and went as planned,
take the steps right under its ask as yours, with the `changed` lines in mind. Copy the steps under a
step only when you take that step. A past step that does the same thing in other words is the same
step. When its title names its own case, write it in words that hold in yours. Add each step you take
with `--uses <its id>`, so that later work can see which steps were reused.

## Other sessions

To go on with another session's trail, add your first step `--under <its id>`. To hand a step to
another agent, put in its task: `celis step <id>: run celis step start <id> first and celis step done
<id> last.`

## Patterns

A pattern is a rule with good and bad examples, which `celis search` gives with
the steps' proofs and the blobs of the files: an example shows what it means better than more words. A
pattern guides and never binds, in every kind of work; a rule wins over it. Keep a pattern from:

- a pick: `decide --good <pick> --bad <the other> --path <the file>`, in the person's words, or
  "because the person picked it" when they gave no reason; never your own choice;
- a reaction to a result: `--good` or `--bad` the step, the file or the URL the person reacted to;
- a correction of done work: the hunk it replaced and the fix in the fix's proof, a few lines each,
  and `--good <the fix>`;
- the person's own rewrite, as `git log` shows at the next ask: a step that `--replaces` yours, closed
  `--by person` with both hunks.

To add an example to a pattern, replace it with `decide --replaces <its id>` and the new example: the
new pattern keeps the old examples. When 3 patterns make the same point, or the picks of a creative ask
share one, propose one rule that says it, and replace them when the person agrees.

## Links

When the person wants to keep the trails in the repo or in a folder, tell them to run
`celis link [<folder>]` in the project themselves.

## Stop

Never write a proof for work you did not do. Stop and tell the person what is open when two reviews in
a row send you back to the same step, or at a limit of time or money the person set. When you lose track,
or after a compaction, run `celis tree`, and go on.
