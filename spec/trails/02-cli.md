---
cip: 2
title: The command line
status: draft
depends: [CIP-01]
---

# CIP-02 — The command line

`celis` writes and reads the records of the other CIPs. This CIP is the one place for its commands: their
usage, what they print, and their errors. The other CIPs state the records and what they mean.

## Commands

```
celis decide [--scope <scope>] [--replaces <id>] -- <text>
                                                       a decision, in its scope
celis tree [<id>] [--full] [--all]                    a trail and its steps; again on each change
celis past [--at <place>]...                          this project's trails and decisions, past work
celis stats [<id>]                                    what later work reused, and what it saved
celis report <id>...                                  the trails as Markdown, for a pull request
celis issue [<id>]... [-- <text>]                     what went wrong, as Markdown for the celis team
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>]... [--check <command>] [--new] [--session <id>] -- <text>
                                                       leave a step
celis step start <id>                                 take up a step, and see its decisions
celis step done <id> [--changed <text>] [--as-planned] [--pitfall <text>] -- <proof>
                                                       close a step with its proof
celis step drop <id> -- <reason>                      close a step whose work is no longer wanted
celis step edit <id> [--at <place>]... [--check <command>] [-- <text>]
                                                       give a step a new title, places or check
celis prompt                                          a person's prompt, from a hook
celis auth login                                      keep the phrase stdin gives
celis auth status                                     the account's npub
celis auth phrase                                     print the phrase
celis update [<version>] [--dry-run]                  this binary, to the latest release or a version

$CELIS_HOME   where celis keeps the phrase and the store
              default: ~/Library/Application Support/Celis
```

**Rules**

- Off macOS, `$CELIS_HOME` MUST default to `~/.local/share/celis`.
- `decide` MUST write a decision on the working folder's project (CIP-01, "Decisions"), with
  `--scope` and `--replaces` as its tags. With `--scope project`, or no `--scope`, it MUST write no
  `scope` tag.
- `decide` MUST write the whole id of the trail that `--scope trail:<id>` names.
- `decide` MUST refuse empty text, and a scope that is not `me`, `project`, `trail:<id>` or `at:<place>`,
  as a usage error. It MUST refuse `trail:<id>` with `no-trail` when no trail of the project has the id,
  or that trail is done or dropped. It MUST refuse a `--replaces` no live decision has, with `no-decision`.
- A flag no command takes is a usage error.

```
decision  text  <id, first 8> <scope> <text>
          json  {"id", "created_at", "project", "scope", "replaces", "content"}
```

**Examples**

```
celis decide --scope at:shelf/prices.py -- "When you change prices: keep them in cents, because floats lost a cent."
celis decide --scope me -- "When you write a commit message: use plain words, because the team reads them."
```

## Help

The page is the reference for a person.

```
celis  ·  celis help  ·  celis --help     the page
celis <command> --help                    the command page
celis step --help · celis auth --help     the group page
celis --version                           celis <version> (<commit, first 8>)
```

The page:

```
celis keeps how you do your tasks. A trail is one task you give, the steps
agents take toward it, and the decisions made on the way. Your agents use it
through the celis plugin.

Commands
  decide   a decision, in its scope
  tree     a trail and its steps, with their grades
  past     this project's active trails and decisions, every project's past work
  stats    what later work reused, and what it saved
  report   trails as Markdown, for a pull request
  issue    what went wrong, as Markdown for the celis team
  step     the steps of a trail: add, start, done, drop, edit
  auth     the account's phrase: login, status, phrase
  update   this binary, to the latest release or a version

$CELIS_HOME holds the phrase and the store.

See `celis <command> --help`.
```

A command page:

```
<each usage line of the command; a line past 80 characters wraps before a flag, and indents>

<what the command does, in one to three sentences>

  <argument or flag>    <what it is>     one line each

Examples
  <a command line>                       one or more
```

**Rules**

- `celis`, `celis help` and `celis --help` MUST print the page and exit 0.
- `--help` anywhere in a command MUST print the command page its words name, and else the page.
- `--help` after only a group's name, `step` or `auth`, MUST print the group page. It MUST list each
  command of the group with the first sentence of its summary, wrapped under its column.
- A command page MUST give a line to each argument and flag its usage names, and at least one example.
- Every help page MUST exit 0, and print to stdout.
- A line of a help page MUST NOT be longer than 80 characters.
- A help page MUST NOT name a spec file.
- `celis --version` MUST print `celis <version>`, and the commit it was built from, when the build had one.
- Each command line of the celis plugin's card MUST name the flags its command's usage names,
  and no other.

## Output and errors

**Rules**

- A command MUST take an id whole, or as its first 4 or more characters when only one record's id
  starts with them.
- Every command MUST accept `--json`, and then print one JSON object a line, and nothing else, to stdout.
- A command that writes a record MUST print its id. With `--json`, it prints `{"id"}`.
- A command MUST NOT ask a question. On a terminal, `auth login` asks for the phrase.
- A usage error MUST name the narrowest help page that covers it: `celis <command> --help`, or else
  `celis help`.
- An unknown command MUST name the known one nearest it, when two edits or fewer turn one into the other.
- A command MUST exit 0 when done, 2 on a usage error, and 1 on any other error.
- Without `--json`, an error MUST be one line on stderr: `celis: <message>`.
- A command whose reader closes stdout MUST stop, and print no message.
- The message of `no-step`, `no-decision`, `no-trail`, `closed` and `open-steps` MUST name a `celis`
  command that shows or fixes what went wrong.
- With `--json`, an error MUST be one object on stderr: `{"error": "<name>", "message": "<text>"}`.
- An error MUST be named `usage`, `no-decision`, `no-trail`, `no-step`, `closed`, `open-steps`,
  `check-failed`, else `error`.

**Examples**

```
$ celis pats
celis: no command "pats"; did you mean "past"? See `celis help`.
```

## Update

`update` replaces the running `celis` with the build of a release for this system and processor:
`celis-<cpu>-<system>.tar.gz`, as `install.sh` downloads it.

```
$CELIS_BASE   where the releases are
              default: https://github.com/celishq/trails/releases
```

**Rules**

- With no `<version>`, `update` MUST take the version of the release that `$CELIS_BASE/latest` redirects
  to, `…/tag/v<version>`. When that version is installed, it MUST download nothing and say so.
- With `<version>`, `update` MUST install `v<version>`, also when it is installed. A leading `v` of
  `<version>` MUST NOT count.
- `update` MUST download `$CELIS_BASE/download/v<version>/<archive>` and `<archive>.sha256`. When the
  archive does not match its checksum, or the new `celis --version` fails, it MUST fail and leave the
  installed file as it was.
- `update` MUST replace the file the running `celis` resolves to with one rename in its folder.
- With `--dry-run`, `update` MUST print the installed and the new version, and change nothing.

```
update  text  Updated celis <from> to <to> at <path>.
              celis <version> is the latest.
              celis <from> is installed; celis update installs <to>.      (--dry-run)
        json  {"from", "to", "path", "updated"}
```

**Examples**

```
celis update
celis update 0.4.0
celis update --dry-run
```

## Auth

`auth` keeps the account's phrase (CIP-01, "The account").

**Rules**

- `auth login` MUST read the phrase from stdin. On a terminal, it MUST NOT echo the phrase.
- `auth login` MUST refuse a phrase given as words of the command, as a usage error.
- `auth login` MUST refuse a text that is not a BIP-39 phrase, and keep the phrase it had.
- `auth login` MUST remove `$CELIS_HOME/key`, the key of older versions.
- `auth status` MUST NOT print the phrase. With `--json`, it MUST print `{"npub", "path"}`.
- `auth status` and `auth phrase` MUST refuse when no phrase is kept, and MUST NOT mint one.
- `auth` MUST have only `login`, `status` and `phrase`. You forget a phrase by removing its file.

**Examples**

You copy the phrase from the Celis app and run `celis auth login`. The steps you add now count in the
app's account. The steps the old phrase signed still count.

## Step

`step` writes the records of CIP-03.

**Rules**

- A command MUST refuse a start that more than one step's id has, with `no-step`.
- `add` MUST refuse empty text as a usage error.
- `add` MUST refuse an `--under` that no step has, with `no-step`, and one that is closed, with `closed`.
- `add` MUST print `ahead` on stderr when its `under` is below an ask and holds an open step that has
  not started. It MUST still write the step.
- `add` MUST carry the whole id of `--under` and `--replaces`, each `--at` and the `--check` as their
  tags. It MUST write each `--at` as CIP-01 "Places" states.
- `add` MUST NOT write a step when an open step of the same project and `under` has the same text. It
  MUST print the id of that step.
- `add --replaces` MUST refuse a trail as a usage error, and a dropped step with `closed`.
- `add --uses` MUST write the whole id of each, as CIP-06 "Uses" states. It MUST refuse an id no live
  decision and no step has, with `no-step`, and a step of the new step's trail, as a usage error.
- `add` of a session's first ask MUST print the ask's id first, then the trail's (CIP-04, "Sessions").
- `add` MUST print `joined` on stderr when a session's step joins its trail (CIP-04, "Sessions").
- `start` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`. On a working
  step, it MUST write nothing and print the id of the step's first `start`.
- `start` MUST then print the live decisions at the step's places (CIP-01, "Reading"), each once.
- `start` MUST NOT run the step's check, and MUST print nothing on stderr.
- `done` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`.
- `done` MUST refuse an empty proof, as a usage error.
- `done` MUST take `--changed <text>` or `--as-planned`. It MUST refuse neither or both as a usage error.
- `done --changed` MUST carry its text as the `changed` tag.
- `done` with `--pitfall` MUST also write it as a decision on the step's project (CIP-01, "Decisions").
  The scope of that decision MUST be `at:<the step's at>`, else `trail:<id of the step's trail>`.
- `done` MUST refuse a step with an open step under it, with `open-steps`, and write nothing.
- `done` MUST run the step's `check` in the project's top folder, when it has one. When the check fails,
  it MUST refuse with `check-failed`, print the end of the check's output, and write nothing.
- `done` MUST stop a check that runs past `CELIS_CHECK_SECONDS`, 600 when not set, and count it as failed.
- `done` MUST print the step's review on stderr (CIP-03, "Reviewing"), every time, and exit 0.
- The next move of an ask MUST be its `ask` line. Else, a done `--changed` MUST print the `changed`
  line, and any other done the `planned` line. A trail's review MUST NOT print a next move.
- `drop` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`. It MUST refuse
  an empty reason as a usage error.
- `edit` MUST refuse an id no step has, with `no-step`. It MUST refuse an edit with no text, `--at` or
  `--check`, as a usage error. It MUST write each `--at` as CIP-01 "Places" states.
- The JSON of a step MUST carry its `under`, `replaces` and each `uses` as `links`, in that order. A step
  with none MUST carry an empty `links`.
- `add`, `start`, `done`, `drop` and `edit` MUST print the id of each record they wrote, the step's own
  first. With `--json`, each prints `{"id"}`, one a line.

```
step    text  <id, first 8> [<project name>] <title>               a step
              <id, first 8> [<project name>] <title> · <at>         a step with a place: its first
        json  {"id", "created_at", "project", "links": [{"to", "kind": "under | replaces | uses"}], "places": [],
              "check", "content"}
ahead   celis: <that step, first 8> has not started; keep one step ahead: take it first, or drop it.
joined  celis: the step joined the trail <id, first 8>; edit its title when it no longer covers the work.
review  celis: <id, first 8> is done; compare the result with its plan:
          expect  <the text after expect:>
          planned <n> steps
          added   <id, first 8> <title>
          dropped <id, first 8> <title> · <reason>
          changed <id, first 8> <text>
          asked   <text>
          open    <id, first 8> <title>
        celis: <the next move>

next move  ask      celis: write what a later plan should keep as rules with celis decide.
           changed  celis: it went otherwise than planned; replace or drop each open step it voids,
                    then add one step under <under, first 8>.
           planned  celis: take an open step beside it, add one step under <under, first 8>, or close
                    <under, first 8> when its work is done.
```

**Examples**

```
celis step add --under 3fa2c1d0 --at shelf/prices.py:40 \
  -- "Write a test: a senior pays 80% of the price."
celis step add --under 3fa2c1d0 --at https://figma.com/design/Ab12/Shop \
  -- "The price list frame shows the senior tier."
celis step start 4c1d22e0
celis step done 4c1d22e0 --as-planned -- "python3 -m unittest: test_senior failed before, passes now."
celis step drop 3fa2c1d0 -- "The person chose the shared price table instead."
```

## Prompt

The prompt hook of each agent gives the person's prompt to `prompt`, and gives its lines to the agent.

**Rules**

- `prompt` MUST read a hook's `{"session_id", "prompt"}` on stdin. A `conversation_id` (Cursor) MUST name
  the session in place of `session_id`.
- `prompt` MUST write the prompt mark as CIP-04 "Sessions" states, and print the record's id.
- With no open trail, `prompt` MUST print one line on stderr, when `past` prints anything. The line MUST
  ask to run the `celis:past` agent with the ask before any other tool, and to wait for it.
- `prompt` MUST print that line once in a session. A hook's `{"session_id", "source": "compact"}` on
  stdin MUST let the session print it again, and MUST write and print nothing. The session start hook
  MUST give its JSON to `prompt`.
- With no open trail and no line for the past agent, it MUST print the join line when a trail of any
  project is one `tree` with no id prints. Else it MUST print nothing.
- With a working step, `prompt` MUST name it on stderr. The line asks for a step under it, or a
  replacement, when the next work does not fit it.
- With an open trail, `prompt` MUST then print the join line on stderr, with the trail's id.
- The join line asks to add a new ask under the session's open trail when it fits, else under a related
  trail that `tree` with no id prints, else as a new trail.

**Examples**

```
echo '{"session_id": "95e9be99", "prompt": "Use the shared Badge."}' | celis prompt
```

## Tree

`tree` shows a trail, its rules and its tree (CIP-04).

```
celis tree [<step id>] [--full]   a trail, its rules and its tree; with no id, today's trails of every project
celis tree --all [--full]         every trail in the store, by day
```

**Rules**

- `tree` MUST refuse an id no step has, with `no-step`.
- `tree` with no id MUST print each trail of every project that is active or has a record from today,
  newest first, in any folder. A day MUST start at midnight, local time.
- With no id and no trail to print, `tree` MUST print `No active trails. Run \`celis tree --all\` to see
  every trail.`, and with `--json` print nothing.
- `tree --all` MUST print every trail in the store, of every project, newest first, as `tree` with no id
  prints them. Before the first trail of each day, it MUST print the day, as `Oct 9`.
- `tree --all` with an id MUST refuse as a usage error.
- With no id, and with `--all`, the tree MUST print each trail's project name after its id.
- In the short tree with no id, a trail that is not active MUST print only its own line. Its details MUST
  hold `1 step` or `<n> steps` when steps are under it.
- `tree` MUST print the step with its state and its rules; `--full` adds its proof and grade when done.
  An ask MUST print its trail's rules. Then come the steps under it, oldest first, each followed by its
  own steps.
- `tree` MUST mark a done step `✓`, a dropped one `✗`, a working one `◐`, an open one `○`, and an
  expired one `!` (CIP-03, "Starting"; CIP-04, "State"). With `--full`, under a step come the lines of its
  text after its title. Then come the proof, what changed and the grade of a done step (CIP-03,
  "Grades"), or a dropped one's reason.
- `tree` MUST NOT write a record.
- `tree` MUST print the short tree, with no line under a step. With `--full`, it MUST print the full
  tree, with the proof, grade and reason lines.
- The short tree MUST fit each line to the width of the terminal. Off a terminal, it MUST use `COLUMNS` as
  the width, and cut nothing when `COLUMNS` is not set.
- The short tree MUST cut a step's text and end it with `…`, and keep its id and details whole.
- `tree` MUST print in colour when stdout is a terminal, or when `CLICOLOR_FORCE` is set and not `0`. It
  MUST NOT when `NO_COLOR` is set.
- Without colour, `tree` MUST print the same lines, with no colour codes.
- Each line of a step MUST start with its mark. A trail MUST be marked `●` when active, `✓` when
  reached or revised, `✗` when dropped, and `!` when expired.
- When a trail's first ask has the trail's text, the tree MUST NOT print the ask, except with `--json`. It
  MUST print the ask's steps first under the trail, then the trail's other asks.
- The tree MUST draw a step under its parent with `├─` before its mark, and the last one with `└─`.
  A `│` MUST run from a step down to its next sibling.
- A step line MUST end with dim details: the last part of its place. Then come how long it has worked
  or took, and when it started.
- A trail line MUST end with dim details: the last part of its place, and its steps when folded. Then
  come how long its tree ran, and when it started. Its mark MUST be its only sign of its state.
- The tree MUST print the details in four columns, in that order, two spaces apart, and the same for
  every line. A place and the steps MUST align left, a time right. A column no line fills MUST be left
  out.
- With a width, the short tree MUST end the details at the right edge. It MUST leave out the start time
  when a title would be cut below 20 characters. Else they MUST start two spaces after the widest title.
  A time MUST print as `14:02` on the day it is, else as `Oct 9`.
- The tree MUST draw each rule of a trail as a branch before its steps: `§`, its id and its text, with
  no details. The short tree MUST print a rule without its reason: the text from `, because` on.
- With no id, the tree MUST pad each project name to the widest, so each trail's title starts in the
  same column.
- In the tree, each line under a step MUST start at the column of the step's title. A label (`done`,
  `took`, `dropped`) MUST end one space before that column.
- With a width, the full tree MUST wrap a rule and a line under a step between words. Each wrapped line
  MUST start at the same column, after the step's branch lines.
- In colour, the tree MUST underline the shortest start of each id, 4 characters or more, that no other
  step's id has. It MUST colour each mark.

```
tree   text  <mark> <id, first 8> <title>   <place>  <n> steps  <ran>  <started>
                        <the next line of its text>
                   done <proof>
                changed <what changed>
                   took <its grade, without "took">
                follows <id, first 8> <title>                 each step it uses
                 reused <n>[ · saved <time>]                  when steps follow it at any depth
                dropped <reason>
               ├─ § <id, first 8> <text>
               ├─ <mark> <id, first 8> <title>   <place>  <took>  <started>
               │           <the lines under it, as under the trail>
               └─ <mark> <id, first 8> <title>   ...
       json  {"trail": {"id", "created_at", "project", "content", "state", "links", "places", "proof", "grade",
             "reused", "reason"}}
             · {"step": {<as "Step" gives it>, "state": "done | dropped | working | open | expired",
             "proof", "changed", "grade", "reused", "reason"}}, one a line
grade  text  took <45s | 1m 30s | 2h 5m | 3d 2h>[ · revised by <id, first 8>, ...]
       json  "grade": {"took": <seconds>, "revised": [<id>]}
```

**Examples**

```
celis tree 3fa2c1d0

● 3fa2c1d0 Add a senior tier to the price list                                12m  14:02
  ├─ § 4c1d22e0 A senior pays 80% of the price, rounded down to the cent.
  ├─ ✓ 9b0e11aa A senior pays 80%      prices.py:40                   1m 30s  14:03
  ├─ ◐ 5d7f3c21 The frame shows the tier                              7m 12s  14:05
  │  └─ ○ 0a4e9d10 Add the badge to the frame
  └─ ○ 6c1b8e52 Round the price down

celis tree

● 3fa2c1d0 shop Add a senior tier to the price list                12m  14:02
  …
✓ 1c9e44b2 crm  Fix the cart total                    3 steps      8m  11:20
```

## Watching

On a terminal, `tree` shows the trails while the agents work on them, so the person sees each step start
and close.

**Rules**

- On a terminal, `tree` with no `--json` MUST watch. Else it MUST print once. `tree` MUST NOT take a
  `--watch` flag.
- A watch MUST print what `tree` with the same words prints off a terminal. It MUST refuse an id no step
  has, with `no-step`, also with no store.
- With no id and no trail to print, it MUST print what `tree` prints.
- It MUST print the tree again as soon as the terminal's width changes. It MUST print it again within
  one second after a time it shows changes.
- Between two trees, it MUST sleep until a record lands, a time it shows changes, or the width
  changes. With no time that runs, it MUST NOT wake on a timer.
- It MUST print the tree again within one second after a record lands in the store, and only when
  the tree changed.
- It MUST clear the screen and its scrollback, and print each tree from the top left.
- It MUST run until it is stopped, and MUST NOT write a record.

**Examples**

```
celis tree 3fa2c1d0
```

## Past

`past` prints what a new ask of the project must keep, then the past work of every project (CIP-04,
"The past").

**Rules**

- `past` MUST print the working folder's project's active trails, oldest first, each with its rules.
- Then it MUST print the live decisions at each `--at` place, then the project's live `project` and `me`
  decisions (CIP-01, "Reading"). It MUST NOT print a decision at a place unless an `--at` place is at it.
- Then it MUST print the past work of every project, newest first.
- Each past work MUST print as one line: its id, project, title and place, its grade or else its state,
  and its day.
- With `--json`, each object MUST be named `past`, with its id, project, title, places, state, grade and
  day.
- `past` MUST NOT write a record.

```
past  text  <id, first 8> [<project name>] <its title>[ · <place>]          each active trail
              rule <id, first 8> <text>                                     each of its rules
            <a decision, as "Commands" gives it>
            <id, first 8> [<project name>] <its title>[ · <place>] · <grade | state> · <yyyy-mm-dd>
      json  {"trail": {"id", "created_at", "project", "content", "session", "rules": []}}
            · {"decision": {...}} · {"past": {"id", "created_at", "day", "project", "content",
            "places", "state", "grade"}}
```

**Examples**

```
celis past --at shelf/prices.py
```

## Stats

`stats` prints what CIP-06 "Derived" gives.

**Rules**

- `stats` MUST refuse an id no step has, with `no-step`. An id of a trail with a twin ask MUST read as
  the ask.
- With no id, it MUST print each followed step, most followers at any depth first, then each used live
  decision, most used first, then the total. Ties MUST print newest first, then by id.
- With no id and nothing reused, it MUST print `Nothing was reused yet.`, and with `--json` nothing.
- With an id, it MUST print the step, then each step it follows, then each step that follows it, at any
  depth, depth first, two spaces further in a level. A saving MUST be against the source on that path.
- `stats` MUST NOT write a record.

```
stats  text  <id, first 8> [<project>] <title> · followed <n>[ · saved <time>][ · revised]
             decision <id, first 8> <brief> · used <n>
             total <n> follows[ · saved <time>] · <n> uses                    1 follow, 1 use for one
       id    <id, first 8> [<project>] <title>[ · <grade>]
               follows <id, first 8> [<project>] <title>[ · <grade>]
               followed <id, first 8> [<project>] <title>[ · <grade>][ · saved <time>]
       json  {"followed": {"id", "project", "content", "followed", "saved", "revised"}}
             · {"used": {"id", "content", "used"}} · {"total": {"follows", "saved", "uses"}}
             · {"step" | "follows" | "followed": {"id", "project", "content", "grade", "saved", "depth"}}
```

**Examples**

```
celis stats
celis stats 9b0e11aa
```

## Report

`report` prints the trails the ids name as Markdown (CIP-05, "The report").

**Rules**

- `report` MUST refuse an id no step has, with `no-step`. It MUST refuse no id as a usage error.
- `report` MUST print the trail of the step each id names, once, in the order of the ids. A blank line
  MUST come between two trails.
- `report` MUST print a trail's title as a heading, and its steps as a Markdown list, each with its mark
  as `tree` marks it. A section with no line MUST be left out.

```
report  text  ## <title of the trail>
              `<id, first 8>` · <project name> · <state> · <n> steps · took <ran>

              <the next line of its text>

              done: <proof>

              **Rules**

              - <text>                                                    each rule

              **Steps**

              - <mark> <title>                                            each step, two spaces a level
                - <the next line of its text>
                - at `<place>`
                - check passed `<command>` | check `<command>`
                - done: <proof>
                - changed: <what changed>
                - dropped: <reason>
        json  {"report": "<the text>"}
```

**Examples**

```
celis report 4c1d22e0 9e717e6f
```

## Issue

`issue` prints what went wrong as Markdown (CIP-05, "The issue").

**Rules**

- `issue` MUST print the first line of the text after `--` as a heading, and the rest of the text under
  it. With no text, the heading MUST be `celis issue`.
- `issue` MUST print the last 50 lines of the log in a code block, and leave the section out when the
  log is empty.
- `issue` MUST print the trails the ids name, as `report` prints them. It MUST refuse an id no step has,
  with `no-step`.

```
issue  text  ## <first line of the text | celis issue>

             <the next line of the text>

             **Facts**

             - version: celis <version> (<commit, first 8>)
             - system: <os> <cpu>
             - sessions: <the names of the set session variables | none>
             - project: <name> `<id, first 8>`
             - home: `<$CELIS_HOME, with ~>`
             - phrase: kept | missing
             - records: <n> files, <n> read, <n> steps

             **Log**

             ```
             <the last 50 lines of the log>
             ```

             <each trail, as report prints it>
       json  {"issue": "<the text>"}
```

**Examples**

```
celis issue -- "celis prompt fails in each Cursor session"
celis issue 4c1d22e0 -- "The tree shows the step as open after done."
```
