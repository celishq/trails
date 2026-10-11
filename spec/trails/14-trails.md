---
cip: 14
title: Trails
status: draft
depends: [CIP-10, CIP-11, CIP-12]
---

# CIP-14 — Trails

## Trails

A trail is one task you give, in your words. It is a step with no `under` (CIP-12, "Steps"). The steps
under it, and the steps under those, are its tree.

**Rules**

- A reader MUST count a step with no `under` as a trail.
- A reader MUST count a step in the tree of the trail at the top of its `under` chain.

**Examples**

```
"The price list shows a senior tier at 80% of the price, and the receipt shows it."
"Make the badges orange."
```

## Sessions

A session is one run of an agent. Claude Code names it in `CLAUDE_CODE_SESSION_ID`, and a subagent shares
it. Many sessions can work on one trail. The hook writes the person's prompts on the session's trail.

```
["session", "<$CLAUDE_CODE_SESSION_ID>"]     a step's tag, optional: the session that wrote it

prompt kind 1985, a prompt while no step works
{ "kind": 1985, "content": "<the prompt's first line, 200 characters at most>",
  "tags": [["L", "celis.step"], ["l", "prompt", "celis.step"], ["e", "<trail id>"]] }

$CELIS_TRAIL    a step id, or a start of it: where `step add` with no `--under` puts the step
```

**Rules**

- `step add` MUST write a `session` tag with `$CLAUDE_CODE_SESSION_ID` on each step, when it is set.
- A trail with a session MUST hold each ask as a step under it, the first ask too. So `step add` MUST write
  the trail, then the ask under it with the same text, and print the ask's id first.
- A session's open trail MUST be the trail of its newest step, while that trail is not done or dropped.
- `step add` with no `--under` or `--new` MUST write the step under `$CELIS_TRAIL` when it is set.
- Else, while the session has an open trail, it MUST write the step under it.
- Else, with a session, it MUST write the step under the newest active trail of the project.
- With `--new`, `step add` MUST write a new trail.
- `prompt` MUST read a hook's `{"session_id", "prompt"}` on stdin. While that session has an open trail,
  it MUST write the prompt on it, and print the record's id. Else it MUST write and print nothing.
- The prompt MUST be a steer on the oldest working step of the trail. With no working step, it MUST be a
  prompt mark on the trail. Its text MUST be the prompt's first line, cut to 200 characters.
- A reader MUST NOT count a prompt mark as a steer.
- `prompt` MUST write nothing for a prompt that starts with `<task-notification>`. The harness wrote it,
  not the person.
- With a working step, `prompt` MUST then name it on stderr. The line asks for a step under it, or a
  replacement, when the next work does not fit it.
- The hook MUST give that line to the agent.

**Examples**

```
CLAUDE_CODE_SESSION_ID=95e9be99 celis step add -- "Make the badges orange."    a trail, the ask under it
CLAUDE_CODE_SESSION_ID=95e9be99 celis step add -- "Now the buttons too."       a step under it
CLAUDE_CODE_SESSION_ID=95e9be99 celis step add --new -- "Fix the login page."  a second trail
CLAUDE_CODE_SESSION_ID=4c1d22e0 celis step add -- "Check the colours."        another session's step goes there
CELIS_TRAIL=3fa2c1d0 celis step add -- "Check the badge on the phone."         a step of any agent there
echo '{"session_id": "95e9be99", "prompt": "Use the shared Badge."}' | celis prompt
```

## Rules

A trail's rules are the live decisions with the scope `trail:<id>` (CIP-10, "Scope"). Each is one
sentence a check can fail.

**Rules**

- A reader MUST count a decision as a rule of a trail only when its scope names that trail.
- `done` on an ask MUST print its retro on stderr, so the agent writes the rules it shows.
- The retro MUST count the steps under the ask written before the first of them started.
- It MUST then list the steps added after, those dropped with their reason, and each `changed`.
- It MUST then list each prompt and steer on the ask's tree, and on its trail since the ask.

**Examples**

```
celis decide --scope trail:4c1d22e0 -- "A senior pays 80% of the price, rounded down to the cent."
```

## State

A trail has no state record. Its state comes from its tree, so the person can see which tasks the agent
reached alone. A trail ends 30 minutes after the newest record of its tree.

```
active              not done, dropped or ended
reached alone       done, or ended with no open step under it; no steer names a step of its tree
reached with help   done, or ended with no open step under it; a steer names a step of its tree
dropped             the trail is dropped
expired             ended with an open step under it
```

**Rules**

- A reader MUST work out a trail's state from its tree, their `start`, `done`, `drop` and steer records,
  and the time.
- The newest record of a tree MUST be the newest of its steps and the marks that name them.
- A trail MUST end 30 minutes after the newest record of its tree.
- A reader MUST NOT count a step of an ended trail as open.

## The commands

```
celis tree [<step id>] [--full]   a trail, its rules and its tree; with no id, today's trails
celis tree --all [--full]         every trail in the store, by day
```

**Rules**

- `tree` MUST take a step id or a start of it, as CIP-11 "Output and errors" states. It MUST
  refuse an id no step has, with `no-step`.
- `tree` with no id MUST print each trail of the working folder's project that is active or has a record
  from today, newest first. A day MUST start at midnight, local time.
- `tree --all` MUST print every trail in the store, of every project, newest first, as `tree` with no id
  prints them. Before the first trail of each day, it MUST print the day, as `Oct 9`.
- `tree --all` with an id MUST refuse as a usage error.
- With `--all`, the tree MUST print each trail's project name after its id, and no project line.
- In the short tree with no id, a trail that is not active MUST print only its own line. Its details MUST
  hold `1 step` or `<n> steps` when steps are under it.
- `tree` MUST print the step with its state and its rules; `--full` adds its proof
  and grade when done. Then come the steps under it, oldest first, each followed by its own steps.
- `tree` MUST mark a done step `✓`, a dropped one `✗`, a working one `◐`, and an open one `○` (CIP-12,
  "Starting"). A step of an ended trail that is not done or dropped MUST be marked `!`, as `expired`. With `--full`, under a step come the lines of its text after its title (CIP-12, "Steps").
  Then come the proof and grade of a done step (CIP-12, "Grades"), or a dropped one's reason.
- `tree` MUST NOT write a record.
- `tree` and `watch` MUST print the short tree, with no line under a step. With `--full`, they MUST
  print the full tree, with the proof, grade and reason lines.
- The short tree MUST fit each line to the width of the terminal. Off a terminal, it MUST use `COLUMNS` as
  the width, and cut nothing when `COLUMNS` is not set.
- The short tree MUST cut a step's text and end it with `…`, and keep its id and details whole.
- `tree` and `watch` MUST print in colour when stdout is a terminal, or when `CLICOLOR_FORCE` is set and
  not `0`. They MUST NOT when `NO_COLOR` is set.
- Without colour, `tree` and `watch` MUST print the same lines, with no colour codes.
- With no id, the tree MUST print the project's name once, as its first line. Its steps MUST NOT carry the
  project name.
- Each line of a step MUST start with its mark. A trail MUST be marked `●` when active, `✓` when
  reached, `✗` when dropped, and `!` when expired.
- When a trail's first ask has the trail's text, the tree MUST NOT print the ask, except with `--json`. It
  MUST print the ask's steps first under the trail, then the trail's other asks.
- The tree MUST draw a step under its parent with `├─` before its mark, and the last one with `└─`.
  A `│` MUST run from a step down to its next sibling.
- A step line MUST end with dim details: the last part of its place and `↳ fixes <id, first 8>`.
  Then come how long it has worked or took, and when it started.
- A trail line MUST end with dim details: the last part of its place, its state, and its steps.
  It MUST leave out the state when active or reached alone, and the steps unless folded. Then come how
  long its tree ran, and when it started.
- The tree MUST print the details in four columns, in that order, two spaces apart, and the same
  for every line. A place and a state MUST align left, a time right. A column no line fills MUST be left out.
- With a width, the short tree MUST end the details at the right edge. It MUST leave out the start time
  when a title would be cut below 20 characters. Else they MUST start two spaces
  after the widest title. A time MUST print as `14:02` on the day it is, else as `Oct 9`.
- The short tree MUST print a rule without its reason: the text from `, because` on.
- In the tree, each line under a step MUST start at the column of the step's title. A label
  (`rule`, `done`, `took`, `dropped`) MUST end one space before that column.
- With a width, the full tree MUST wrap a line under a step between words. Each wrapped line MUST
  start at the same column, after the step's branch lines.
- In colour, the tree MUST underline the shortest start of each id, 4 characters or more, that no other
  step's id has. It MUST colour each mark.

```
tree  text  <mark> <id, first 8> <title>   <place>  <state> · <n> steps  <ran>  <started>
                       <the next line of its text>
                  done <proof>
                  took <its grade, without "took">
               dropped <reason>
                  rule <id, first 8> <text>
              ├─ <mark> <id, first 8> <title>   <place>  ↳ fixes <id, first 8>  <took>  <started>
              │           <the lines under it, as under the trail>
              └─ <mark> <id, first 8> <title>   ...
      json  {"trail": {"id", "created_at", "project", "content", "state", "links", "places", "proof", "grade",
            "reason"}}
            · {"step": {<as CIP-12 "The commands" gives it>, "state": "done | dropped | working | open
            | expired", "proof", "grade", "reason"}}, one a line
```

**Examples**

```
celis tree 3fa2c1d0

● 3fa2c1d0 Add a senior tier to the price list                                12m  14:02
  rule 4c1d22e0 A senior pays 80% of the price, rounded down to the cent.
  ├─ ✓ 9b0e11aa A senior pays 80%      prices.py:40                   1m 30s  14:03
  ├─ ◐ 5d7f3c21 The frame shows the tier                              7m 12s  14:05
  │  └─ ○ 0a4e9d10 Add the badge to the frame
  └─ ○ 6c1b8e52 Round the price down                 ↳ fixes 9b0e11aa

celis tree

shop
● 3fa2c1d0 Add a senior tier to the price list                     12m  14:02
  …
✓ 1c9e44b2 Fix the cart total                         3 steps      8m  11:20
```

## The past

The past shows how a task like a new one was done, so a new plan can follow the path that worked.

**Rules**

- `read --like` MUST print the past work of every project that shares a word with the text, 3 at most.
- Past work MUST be a done ask, or a reached trail. An ask is a step right under a trail.
- A trail with an ask of its own text MUST NOT count as past work, so the ask stands for it.
- A word MUST be 4 or more letters or digits, in lower case.
- A word MUST weigh `ln(1 + (n - m + 0.5) / (m + 0.5))`. Here `n` counts the past work, and `m` the
  past work that holds the word.
- The highest sum of the shared words' weights MUST come first. Then come the fewest steers and rework
  in its tree, then the newest.
- Each MUST print as one line: its id, project, title and place, then its grade, else its state.
  Under it come the rules of its trail, then the live decisions at its place.
- With `--full`, each MUST print as `tree --full` prints it.
- With `--json`, each object MUST be named `past`.

```
celis read --like "Add an Invoices page from the frame"
```

## Watching

`watch` shows the trails while the agents work on them, so the person sees each step start and close.

```
celis watch [<step id>] [--full]   a trail's tree, printed again on each change to the store
```

**Rules**

- `watch <id>` MUST print the tree of the step `<id>` as `tree` prints it. It MUST refuse an id no step
  has, with `no-step`.
- `watch` with no id MUST print every active trail of every project, as `tree --all` prints them. With
  no active trail, it MUST say so and name `/celis:trail`, and with `--json` print nothing. With no
  store, `watch <id>` MUST refuse with `no-step`.
- On a terminal, `watch` MUST print the tree again as soon as the terminal's width changes. It MUST print
  it again within one second after a time it shows changes.
- Between two trees, `watch` MUST sleep until a record lands, a time it shows changes, or the width
  changes. With no time that runs, it MUST NOT wake on a timer.
- `watch` MUST print the tree again within one second after a record lands in the store, and only when
  the tree changed.
- On a terminal, `watch` MUST clear the screen and its scrollback, and print each tree from the top
  left. Else it MUST print an empty line between two trees.
- `watch` MUST run until it is stopped, and MUST NOT write a record.

**Examples**

```
celis watch 3fa2c1d0
```
