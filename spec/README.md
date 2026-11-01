# CIPs — Trails

These specs hold the contract of the patterns and of the trails that are their evidence, for the crates
`celis-store`, `celis-patterns`, `celis-trails` and `celis`. A pattern says what to choose in a context,
and why, and guides the agent's judgement. A trail is one task the person gives, broken into steps that
agents take and close with proofs. The patterns and trails of every project on the machine share one
store. Records are nostr notes your account signs.

## How a CIP works

A CIP is one concept in one numbered file, `NN-name.md`, opening with frontmatter: `cip`, `title`,
`status` (draft or final) and `depends`. Then come one paragraph on what it defines, and its sections. A
section opens with a JSON example of its record, then one line for each tag, then **Writers** and
**Readers**: the rules on each. CIP-02 gives **Rules** on commands, then their output.

Every change of `spec/` MUST keep these rules:

1. **A CIP holds a rule only when another end breaks if it changes:** another `celis` version that reads
   the same store or a linked folder, the plugin with its hooks, or a script that reads the
   `--json` of a record. What only `celis` reads or shows — text for a person or an agent, and values it
   works out, such as states by time, grades or reuse — is the app's, and its tests own it.
2. **Data first.** A section opens with the record, then one line for each tag.
3. **Writers and Readers.** A reader rule exists only when two readers must agree.
4. **MUST only where interop breaks.** Else plain words, or MAY.
5. **One concept a CIP.** A CIP cites what another owns, and never repeats it.
6. **The base CIP says once that a reader ignores what it does not know.**
7. **This README is the one registry** of kinds, tags, scopes, errors and retired names.
8. **Examples are real records or commands,** not stories.
9. **A rule states what, never how or why.** How belongs in the code, why in `DESIGN.md` or the plan.
10. **A section heading is stable.** Tests cite it, so a rename or removal changes each citation in the same
    commit.
11. **The spec changes first.** A test of a spec rule cites its section: ``(`spec/03-steps.md`,
    "Closing")``. A test of what the spec leaves out cites nothing.
12. **A CIP adds, and never changes another.** A final CIP is frozen. A draft CIP MAY give a name a new
    meaning only when older readers still read each record right; else the meaning is a new CIP.

## The CIPs

| CIP | Name | Holds | Depends |
|---|---|---|---|
| 01 | Records | the record, the store, the account, projects, places, links | — |
| 02 | The command line | usage, the records each command writes, errors, and the output the plugin reads | CIP-01 |
| 03 | Steps | the step, its start, done, drop, edit and note | CIP-01 |
| 04 | Trails | the trail, its sessions and prompts, its notes | CIP-01, 03, 05 |
| 05 | Patterns | the pattern, its scope and its examples | CIP-01 |
| 06 | Reuse | the patterns a step uses, revisions | CIP-03, 04, 05 |
| 08 | Files | the files of a done and of a pattern, and their blobs | CIP-01, 03, 05 |
| 09 | Cells | the cell, its masks, the view, and its export | CIP-01, 05, 08 |

## Names

| Name | What | CIP |
|---|---|---|
| `project:<id>` · `project` | anchor of a project, its `K`, and its name tag | 01 |
| `figma:<key>` · `web:<host>` | the project id of a record outside git | 01 |
| `$CELIS_HOME` · `$CELIS_HOME/trails/` · `$CELIS_HOME/phrase` · `$CELIS_HOME/links` | the home, the store, the phrase, the links | 01, 02 |
| `celis` and its commands and flags | the command line | 02 |
| `usage` · `no-pattern` · `no-step` · `closed` · `open-steps` · `check-failed` · `secret` · `no-cell` · `cell-taken` · `cell-yours` · `invalid` · `error` | errors | 02 |
| 1111 with `step` in `celis.step` | kind: a step (NIP-22) | 03 |
| `under` · `replaces` · `at` · `check` | step tags | 03 |
| 1985 with `start` · `done` · `drop` · `edit` · `note` in `celis.step` | kinds: a start, a done, a drop, an edit, a note (NIP-32) | 03 |
| `changed` · `by` | done tags | 03 |
| `session` · `agent` | step tags | 04 |
| 1985 with `prompt` in `celis.step` | kind: a person's prompt, on the trail (NIP-32) | 04 |
| 1111 with `pattern` in `celis.pattern` | kind: a pattern (NIP-22) | 05 |
| `scope` · `replaces` | pattern tags | 05 |
| `path` | pattern tag: its paths | 05 |
| `good` · `bad` | pattern tags: its examples | 05 |
| `within` | pattern tag: the larger pattern it completes | 05 |
| `me` · `project` | the scopes of a pattern | 05 |
| `uses` | step tag | 06 |
| 1985 with `recall` in `celis.step` · `pattern` | kind: a recall of a pattern for a step, and its tag (NIP-32) | 06 |
| `misled` | step tag and done tag | 06 |
| `imeta` with `url` · `x` · `m` · `size` · `path` | done tag: a file of the proof (NIP-92, BUD-10) | 08 |
| `$CELIS_HOME/trails/blobs/` · `<folder>/blobs/` | the blobs of the store and of a linked folder | 08 |
| 1985 with `<cell name>` in `celis.cell` · `e` | kind: a membership of patterns in a cell (NIP-32) | 09 |
| 1985 with `on` · `off` in `celis.mask` · `cell` · `p` · `scope` | kind: a mask of a cell, and its tags (NIP-32) | 09 |
| `cell.jsonl` · `cell.md` · `blobs/<sha256>.<ext>` | the files of an exported cell | 09 |

**Retired.** Older versions wrote these names, and no new name takes one. A reader ignores each of them,
and `celis` refuses each command and flag as a usage error. `migrate` reads the ones its row names
(CIP-02, "Migrate").

| Name | Was |
|---|---|
| `file` | a step tag, now `at` |
| 1985 with `steer` in `celis.step` | a steer, now a prompt |
| 1111 in `celis.cairn` · kind 7 · `celis.outcome` · `celis.source` · splits | cairns, ratings, outcomes, sources, splits |
| a `done` with a `step` tag in place of `e` | a done of a step in the code |
| 1985 with `name` in `celis.step` · `skill:<name>` | the name of a trail, and the project of a trail from a skill file |
| 1111 with `decision` in `celis.decision` | a pattern, now `pattern` in `celis.pattern`; `migrate` reads it |
| `binds` in `celis.pattern` or `celis.decision` | a pattern label: the pattern was a rule |
| `promoted` in `celis.decision` | a pattern label, later `binds` |
| `situation` in `celis.decision` | a pattern label: the pattern also held where no file shows it ahead |
| `tool` | a rule tag: the other tool calls a rule acted on |
| `cell` on a pattern | a pattern tag: a topic the pattern was in |
| `at:<place>` as the `scope` of a pattern | a scope, now `path` globs; `migrate` reads it |
| `trail:<id>` as the `scope` of a pattern | a scope, now a note on the trail; `migrate` reads it |
| `uses` that names a step | a step tag: a follow of a past step |
| `recalled` | an ask tag: a pattern the agent read for the ask |
| `judge` as the `by` of a `done` | a proof by a reader that did not do the work |
| `celis lesson <id>` · `celis search` | commands, now `celis extract <id>` and `celis patterns` |
| `pattern --binds` · `pattern --cell <topic>` | flags of a rule and a topic |
| `step add --recalled <id>` | a flag of a pattern read for an ask |
| `step done --by judge` | a flag of a proof by the judge |

## Words

| Word | Means |
|---|---|
| `store` | `$CELIS_HOME/trails/`: the records of every project on this machine |
| `record` | one signed nostr note in the store |
| `account` | the key the phrase gives, that signs every record this machine writes |
| `project` | the code an agent works on, named by its repo's first commit |
| `place` | a file, a `file:line`, a folder or a URL that a step or a pattern is at |
| `step` | one part of the work, at a place |
| `trail` | one ask, and the steps under it |
| `ask` | a step with no `under`: one thing you asked for |
| `session` | one run of an agent |
| `recall` | a read of a whole pattern for the work of a step |
| `prompt` | what you wrote to the agent while a trail was open |
| `pattern` | what to choose in a context, and why, that holds in its scope until another replaces it |
| `note` | what holds for the work of one trail, on a step of it |
| `live` | a pattern no other pattern replaces and no drop names |
| `revised` | done work that a later step replaces |
| `blob` | the bytes of a file, named by their sha256 |
| `cell` | a named set of patterns of one account, that a project turns on or off |
| `view` | the patterns of a project that no cell that is off holds |
