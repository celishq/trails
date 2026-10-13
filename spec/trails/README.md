# CIPs — Trails

These specs hold the contract of the trail, the crates `celis-trails` and `celis`. A trail is one task the person gives,
broken into steps that agents take and close, with the decisions made on the way, each at its place. The
trails of every project on the machine share one store, so later work can follow the paths that worked.
Records are nostr notes your account signs.

## How a CIP works

A CIP is one feature in one numbered file, `NN-name.md`, opening with frontmatter: `cip`, `title`,
`status` (draft or final) and `depends`.

- **A CIP adds. It never changes another CIP.** A new meaning for a name that exists is a new CIP.
- **Each name has one owner,** listed below. A CIP that needs another's name cites it.
- **A rule states what a program does, never how.** How belongs in the code.
- **A final CIP is frozen.** Changed behaviour is a new number.

## The CIPs

| CIP | Name | Holds | Depends |
|---|---|---|---|
| 01 | Decisions | the store, projects, the account, decisions, their scope, how they are read | — |
| 02 | The command line | `celis`: commands, help, output and errors | CIP-01 |
| 03 | Steps | the step, its project, closing, steering, grades, `celis step` | CIP-01, 02 |
| 04 | Trails | the trail, its rules, its state, `celis tree`, the past, watching | CIP-01, 02, 03 |
| 05 | Reports | `celis report`: trails as Markdown for a pull request; the log; `celis issue` | CIP-01, 02, 03, 04 |

## Names

| Name | What | CIP |
|---|---|---|
| 1111 with `decision` in `celis.decision` | kind: a decision (NIP-22) | 01 |
| `project:<id>` · `project` | anchor of a project, and its `K` | 01 |
| `scope` · `replaces` | decision tags: where it holds, the decision it replaces | 01 |
| `me` · `project` · `trail:<id>` · `at:<place>` | the scopes of a decision | 01 |
| `$CELIS_HOME` | where celis keeps the phrase and the store | 02 |
| `$CELIS_HOME/log` | the runs of `celis` that failed | 05 |
| `$CELIS_HOME/phrase` · `celis auth` | the account's phrase, and the commands that keep it | 01 |
| `no-decision` · `no-trail` | errors | 02 |
| 1111 with `step` in `celis.step` | kind: a step (NIP-22) | 03 |
| 1985 with `done` in `celis.step` | kind: a done (NIP-32) | 03 |
| 1985 with `start` in `celis.step` | kind: a start (NIP-32) | 03 |
| 1985 with `steer` in `celis.step` | kind: a steer (NIP-32) | 03 |
| 1985 with `drop` in `celis.step` | kind: a drop (NIP-32) | 03 |
| 1985 with `edit` in `celis.step` | kind: an edit (NIP-32) | 03 |
| 1985 with `prompt` in `celis.step` | kind: a prompt while no step works (NIP-32) | 04 |
| `under` · `replaces` · `at` · `check` | step tags; an old `file` reads as an `at` | 03 |
| `figma:<key>` · `web:<host>` | project id of a step outside git | 03 |
| `no-step` · `closed` · `open-steps` · `check-failed` | errors | 03 |

## Words

| Word | Means |
|---|---|
| `store` | `$CELIS_HOME/trails/`: the records of every project on this machine |
| `project` | the code an agent works on, named by its repo's first commit |
| `account` | the key the phrase gives, that signs every record this machine writes (`celis-nostr`) |
| `agent` | one session of one agent that works for you, and signs as your account |
| `record` | one signed nostr note in the store |
| `trail` | one task you give, in your words: a step with no `under`, and the steps under it |
| `rule` | a live decision whose scope is a trail |
| `scope` | where a decision holds: a place, a trail, the project, or every project |
| `step` | one part of the work toward a trail, at a place: open, working, done or dropped |
| `steer` | what you asked for while a step was open, written by the prompt hook |
| `decision` | one sentence an agent writes, that holds in its scope until another replaces it |
| `live` | a decision no other decision replaces |
| `place` | a file, a `file:line`, a folder or a URL that a step or a decision is at |

## A decision, end to end

```
1  you give a task; an agent finds the past trails like it          CIP-04 — The past
2  the agent starts a trail, and records the rules you state          CIP-04 — Trails, Rules
3  it adds the first steps under the trail, at their places          CIP-03 — The commands
4  agent A finds the tests need SHELF_OFFLINE=1, and decides it       CIP-01 — Decisions, Scope
   at tests/test_prices.py
5  agent B works at the same file; its read gives A's decision first  CIP-01 — Reading
6  `celis tree` shows the trail, its steps and their grades           CIP-04 — The commands
```
