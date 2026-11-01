# Steps

The cases that the card leaves out. Do not run `--help`.

## Paths

A pattern that holds in one module or for one kind of file takes it as `--path`, a glob such as
`shelf/**` or `**/*.svg`, never one file: the When says the moment, and the path says the place. The
index shows each path, so read it when you choose. To find a pattern by its words, run
`celis grep -i -E " pattern [0-9a-f]{8} live .*(<word>|<word>)"`. When you move a module, replace each
pattern of its old place with one of the new `--path`.

## Patterns

Keep a pattern from:

- a pick: `--good <the pick> --bad <the other>`, in the person's words; never your own choice;
- a reaction: `--good` or `--bad` what it was about;
- a fix of done work with no "always" or "never": the old hunk and the fix in the fix step's proof,
  and `--good <the fix step>`;
- the person's own rewrite, which `git log` shows: a step that `--replaces` yours, `--by person`.

To add an example, replace the pattern: the new one keeps the old examples.

## Work

- When the intent changes, run `celis step edit <the ask's id> -- "<its title and new lines>"`.
- When a step grows past one action, or you try the same thing twice, split it.
- To go on with an open ask of another session, add your steps `--under <its id>`.
- To hand a step to an agent, put in its task: `celis step <id>: run celis step start <id> first and
  celis step done <id> last.`

## Stop

Never write a proof for work you did not do. Stop and tell the person what is open when two reviews
send you back to the same step, or at a limit the person set. When you lose track, run `celis tree`.
