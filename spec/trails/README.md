# CIPs — Trails

These specs hold the contract of the trail, the crate `celis-trails`. A trail is one task the person gives,
broken into steps that agents take and close, with the decisions made on the way, each at its place. The
trails of every project on the machine share one store, so later work can follow the paths that worked.
Records are nostr notes your account signs. They follow the vault's CIP rules
(`spec/vault/README.md`, "How a CIP works"), and take numbers from 10 so the two registries never share one.

The conventions in `spec/AGENTS.md` apply here unchanged.

## The CIPs

| CIP | Name | Holds | Depends |
|---|---|---|---|
| 10 | Decisions | the store, projects, the key, decisions, their scope, how they are read | CIP-01 |
| 11 | The command line | `celis`: commands, help, output and errors | CIP-10 |
| 12 | Steps | the step, its project, closing, steering, grades, `celis step` | CIP-10, 11 |
| 14 | Trails | the trail, its rules, its state, `celis tree`, the past, `celis watch` | CIP-10, 11, 12 |

## Names

| Name | What | CIP |
|---|---|---|
| 1111 with `decision` in `celis.decision` | kind: a decision (NIP-22) | 10 |
| `project:<id>` · `project` | anchor of a project, and its `K` | 10 |
| `scope` · `replaces` | decision tags: where it holds, the decision it replaces | 10 |
| `me` · `project` · `trail:<id>` · `at:<place>` | the scopes of a decision | 10 |
| `$CELIS_HOME` | where celis keeps the key and the store | 11 |
| `no-decision` · `no-trail` | errors | 11 |
| 1111 with `step` in `celis.step` | kind: a step (NIP-22) | 12 |
| 1985 with `done` in `celis.step` | kind: a done (NIP-32) | 12 |
| 1985 with `start` in `celis.step` | kind: a start (NIP-32) | 12 |
| 1985 with `steer` in `celis.step` | kind: a steer (NIP-32) | 12 |
| 1985 with `drop` in `celis.step` | kind: a drop (NIP-32) | 12 |
| `under` · `fixes` · `follows` · `at` · `file` · `check` | step tags | 12 |
| `figma:<key>` · `web:<host>` | project id of a step outside git | 12 |
| `no-step` · `closed` · `open-steps` · `check-failed` | errors | 12 |

## Words

| Word | Means |
|---|---|
| `store` | `$CELIS_HOME/trails/`: the records of every project on this machine |
| `project` | the code an agent works on, named by its repo's first commit |
| `account` | the key that owns the store and signs every record in it (CIP-01) |
| `agent` | one session of one agent that works for you, and signs as your account |
| `record` | one signed nostr note in the store |
| `trail` | one task you give, in your words: a step with no `under`, and the steps under it |
| `rule` | a live decision whose scope is a trail |
| `scope` | where a decision holds: a place, a trail, the project, or every project |
| `step` | one part of the work toward a trail, at a place: open, working, done or dropped |
| `steer` | what you asked for while a step was open, written by the agent |
| `decision` | one sentence an agent writes, that holds in its scope until another replaces it |
| `live` | a decision no counted decision replaces |
| `rework` | a step that fixes the work of a done step |
| `place` | a file, a `file:line`, a folder or a URL that a step or a decision is at |

## A decision, end to end

```
1  you give a task; the agent reads the past trails like it          CIP-14 — The past
2  the agent starts a trail, and records the rules you state          CIP-14 — Trails, Rules
3  it adds the first steps under the trail, at their places          CIP-12 — The commands
4  agent A finds the tests need SHELF_OFFLINE=1, and decides it       CIP-10 — Decisions, Scope
   at tests/test_prices.py
5  agent B works at the same file; its read gives A's decision first  CIP-10 — Reading
6  `celis tree` shows the trail, its steps and their grades           CIP-14 — The commands
```
