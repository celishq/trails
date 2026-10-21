# CIPs — Trails

These specs hold the contract of the trail, the crates `celis-trails` and `celis`. A trail is one task the
person gives, broken into steps that agents take and close, with the rules learned on the way. The trails
of every project on the machine share one store. Records are nostr notes your account signs.

## How a CIP works

A CIP is one concept in one numbered file, `NN-name.md`, opening with frontmatter: `cip`, `title`,
`status` (draft or final) and `depends`. Then come one paragraph on what it defines, and its sections. A
section opens with a JSON example of its record, then one line for each tag, then **Writers** and
**Readers**: the rules on each. CIP-02 gives **Rules** on commands, then their output.

Every change of `spec/` MUST keep these rules:

1. **A CIP holds a rule only when another end breaks if it changes:** another `celis` version that reads
   the same store or a linked folder, the plugin with its hooks and `past.md`, or a script that reads the
   `--json` of a record. What only `celis` reads or shows — text for a person or an agent, and values it
   works out, such as states by time, grades or reuse — is the app's, and its tests own it.
2. **Data first.** A section opens with the record, then one line for each tag.
3. **Writers and Readers.** A reader rule exists only when two readers must agree.
4. **MUST only where interop breaks.** Else plain words, or MAY.
5. **One concept a CIP.** A CIP cites what another owns, and never repeats it.
6. **The base CIP says once that a reader ignores what it does not know.**
7. **This README is the one registry** of kinds, tags, scopes, errors and deprecated names.
8. **Examples are real records or commands,** not stories.
9. **A rule states what, never how or why.** How belongs in the code, why in the plan.
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
| 03 | Steps | the step, its start, done, drop and edit | CIP-01 |
| 04 | Trails | the trail, its sessions and prompts, its rules | CIP-01, 03, 05 |
| 05 | Rules | the rule and its scope | CIP-01 |
| 06 | Reuse | the steps and rules a step uses, revisions | CIP-03, 04, 05 |
| 07 | Names | the name of a trail, skill projects, the rules of a name, overrides | CIP-01, 03, 04, 05, 06 |

## Names

| Name | What | CIP |
|---|---|---|
| `project:<id>` · `project` | anchor of a project, its `K`, and its name tag | 01 |
| `figma:<key>` · `web:<host>` | the project id of a record outside git | 01 |
| `skill:<name>` | the project id of a trail from a skill file | 07 |
| `$CELIS_HOME` · `$CELIS_HOME/trails/` · `$CELIS_HOME/phrase` · `$CELIS_HOME/links` | the home, the store, the phrase, the links | 01, 02 |
| `$CELIS_HOME/given` | the log of the rules `rules` gave, and of compactions | 02 |
| `celis` and its commands and flags, `rules` among them | the command line | 02 |
| `usage` · `no-decision` · `no-trail` · `no-step` · `closed` · `open-steps` · `check-failed` · `error` | errors | 02 |
| 1111 with `step` in `celis.step` | kind: a step (NIP-22) | 03 |
| `under` · `replaces` · `at` · `check` | step tags | 03 |
| 1985 with `start` · `done` · `drop` · `edit` in `celis.step` | kinds: a start, a done, a drop, an edit (NIP-32) | 03 |
| `changed` | done tag | 03 |
| `session` | step tag | 04 |
| 1985 with `prompt` in `celis.step` | kind: a person's prompt, on the trail (NIP-32) | 04 |
| 1111 with `decision` in `celis.decision` | kind: a rule (NIP-22) | 05 |
| `scope` · `replaces` | rule tags | 05 |
| `path` · `content` · `command` · `tool` | rule tags: the trigger | 05 |
| `situation` · `promoted` in `celis.decision` | rule labels | 05 |
| `me` · `project` · `trail:<id>` · `at:<place>` | the scopes of a rule | 05 |
| `uses` | step tag | 06 |
| 1985 with `name` in `celis.step` | kind: the name of a trail (NIP-32) | 07 |

**Deprecated.** A reader ignores each of these. Older versions wrote them.

| Name | Was |
|---|---|
| `file` | a step tag, now `at` |
| 1985 with `steer` in `celis.step` | a steer, now a prompt |
| 1111 in `celis.cairn` · kind 7 · `celis.outcome` · `celis.source` · splits | cairns, ratings, outcomes, sources, splits |
| a `done` with a `step` tag in place of `e` | a done of a step in the code |
| `$CELIS_HOME/key` | the key, now the phrase; `auth login` removes it |

## Words

| Word | Means |
|---|---|
| `store` | `$CELIS_HOME/trails/`: the records of every project on this machine |
| `record` | one signed nostr note in the store |
| `account` | the key the phrase gives, that signs every record this machine writes |
| `project` | the code an agent works on, named by its repo's first commit |
| `place` | a file, a `file:line`, a folder or a URL that a step or a rule is at |
| `step` | one part of the work, at a place |
| `trail` | one task you give: a step with no `under`, and the steps under it |
| `ask` | a step right under a trail: one thing you asked for |
| `session` | one run of an agent |
| `prompt` | what you wrote to the agent while a trail was open |
| `rule` | one sentence that holds in its scope until another replaces it |
| `live` | a rule no other rule replaces and no drop names |
| `revised` | done work that a later step replaces |
| `named trail` | a trail that holds a name |
| `override` | a rule that replaces a trail's rule of another project, in its own project only |
