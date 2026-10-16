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
celis tree [<id>] [--full] [--all] [--demo]           a trail and its steps; again on each change
celis past [--at <place>]...                          this project's trails and decisions, past work
celis stats [<id>] [--demo]                           what later work reused, and what it saved
celis report <id>... [--demo]                         the trails as Markdown, for a pull request
celis issue [<id>]... [-- <text>]                     what went wrong, as Markdown for the celis team
celis link [<folder>]                                 keep this project's trails in a folder
celis unlink [<folder>]                               stop keeping them in a folder
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
celis update [<version>]                              this binary, to the latest release or a version

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
celis step  ·  celis auth                 the group page
celis help <command>                      what celis <command> --help prints
celis --version                           celis <version> (<commit, first 8>)
```

The page:

```
celis remembers how your agents do your tasks. Each task you give becomes a
trail: the steps your agents take and the decisions they make along the way.
Your agents use celis through the celis plugin.

Record work (your agents run these)
  step     Add, start, finish, drop, or edit a step
  decide   Record a decision

Read
  tree     Show a trail and its steps
  past     Show this project's active trails, decisions, and past work
  stats    Show what reused work saved
  report   Show trails as Markdown for a pull request

Set up
  link     Keep this project's trails in a folder, like the repo or a drive
  unlink   Stop keeping this project's trails in a folder
  auth     Manage your recovery phrase
  update   Update celis to the latest version, or to the one you choose
  issue    Write a problem report for the celis team

celis keeps your recovery phrase and records in $CELIS_HOME.

To learn more about a command, run `celis <command> --help`.
```

The group page of `step`:

```
celis step <command>

Add steps to a trail, then start, finish, drop, or edit them.

Commands
  add      Add a step to a trail, or start a new trail
  start    Start a step and show the decisions at its places
  done     Finish a step with proof that it works
  drop     Drop a step you no longer need, and the open steps under it
  edit     Change a step's title, places, or check

To learn more about a command, run `celis step <command> --help`.
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

- `celis`, `celis help`, `celis --help` and `celis -h` MUST print the page and exit 0.
- `--help` or `-h` anywhere in a command MUST print the command page its words name, and else the page.
- A group's name alone, `step` or `auth`, with or without `--help`, MUST print the group page. It MUST
  list each command of the group with what it does.
- `celis help <words>` MUST print what `celis <words> --help` prints.
- The page and a group page MUST describe each command with a verb, in sentence case.
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
- A usage error MUST name the narrowest help page that covers it: `celis <command> --help`, else
  `celis <group> --help`, or else `celis help`.
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

`update` replaces the running `celis` with the build of a release: it runs the `install.sh` of the latest
release, `$CELIS_BASE/latest/download/install.sh`, with `sh`.

```
$CELIS_BASE   where the releases are
              default: https://github.com/celishq/trails/releases
```

**Rules**

- `update` MUST run `install.sh` with `CELIS_UPDATE=1`, `CELIS_BASE`, `CELIS_VERSION` set to `<version>`
  or `latest`, and `CELIS_INSTALL_DIR` set to the folder of the file the running `celis` resolves to.
- With no `<version>`, `install.sh` MUST take the version of the release that `$CELIS_BASE/latest`
  redirects to, `…/tag/v<version>`. When that version is installed, it MUST download nothing and say so.
- With `<version>`, `update` MUST install `v<version>`, also when it is installed. A leading `v` of
  `<version>` MUST NOT count.
- When the archive does not match its checksum, `update` MUST fail and leave the installed file as it was.
- When `install.sh` fails, `update` MUST exit with its code and print no error of its own.
- With `CELIS_UPDATE` set, `install.sh` MUST NOT change PATH or print the next steps of a first install.

```
update  text  the output of install.sh
```

**Examples**

```
celis update
celis update 0.4.0
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
- `add --replaces` MUST read a trail with a first ask (CIP-04, "Sessions") as that ask. It MUST refuse a
  trail with no first ask as a usage error, and a dropped step with `closed`.
- `add --uses` MUST write the whole id of each, as CIP-06 "Uses" states. It MUST refuse an id no live
  decision and no step has, with `no-step`, a step of the new step's trail, and a start that both a
  decision's and a step's id have, as a usage error.
- `add` with `--uses`, or a `--replaces` of a done step, MUST refuse an open step of the same project,
  `under` and text as a usage error, and write nothing.
- `add` of a session's first ask MUST print the ask's id first, then the trail's (CIP-04, "Sessions"). It
  MUST name which is the ask and which is the trail on stderr.
- `add` MUST print `joined` on stderr when a session's step joins its trail (CIP-04, "Sessions"), and
  not for a step that replaces another.
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

## Link

`link` links the working folder's project to a folder (CIP-01, "Links"), and syncs it with the store.
`unlink` takes the link out.

**Rules**

- `link` and `unlink` MUST take `<folder>` from the working folder, and `.celis/trails` in the top
  folder when none is given.
- `link` MUST create the folder, and its `README.md` when it is missing.
- It MUST add the link to `$CELIS_HOME/links` when it is not there.
- It MUST copy each record of the store that the folder holds and does not have into the folder. It MUST
  copy each record of the folder that the store does not have into the store.
- It MUST print the folder as `$CELIS_HOME/links` names it, the count of records it copied out and the
  count it copied in.
- A second `link` MUST copy nothing.
- `unlink` MUST take the link out of `$CELIS_HOME/links`, keep the folder, and copy nothing.
- `link` and `unlink` MUST NOT write a record.

```
link    text  <folder> · <out> out · <in> in
        json  {"folder", "out", "in"}
unlink  text  <folder> · unlinked
        json  {"folder", "unlinked": true}
```

**Examples**

```
celis link
celis link ~/Drive/celis/shop
celis unlink ~/Drive/celis/shop
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
- With an open trail that holds done steps, `prompt` MUST then print a line on stderr that asks to add a
  change to done work with `step add --replaces`, and then one line for each done step of the trail, with
  its id and title, oldest first.
- With an open trail, `prompt` MUST then print the join line on stderr, with the trail's id.
- When the working folder's top folder has `.celis/trails/` and its link is not on (CIP-01, "Links"),
  `prompt` MUST print a line on stderr, once in a session, also with no store. It MUST NOT print it on a
  prompt that prints the line for the past agent. The line MUST ask the agent to ask the person whether
  to link the project, and say that the person runs `celis link`.
- The join line asks to add a new ask under the session's open trail when it fits, else under a related
  trail that `tree` with no id prints, else as a new trail.
- The line for the past agent and the join line MUST end by asking to write each rule the prompt states
  (review feedback, a convention, a correction) with `decide` before any file changes.

**Examples**

```
echo '{"session_id": "95e9be99", "prompt": "Use the shared Badge."}' | celis prompt
```

## Tree

`tree` shows the progress of the agents' work and its lessons: each trail, its steps and its rules
(CIP-04). Off a terminal it prints the agent view: all of it in one call, ids on every line, plain and
compact. On a terminal it watches (see "Watching").

```
celis tree [<step id>] [--full]   a trail, its rules and its steps; with no id, today's trails of every project
celis tree --all [--full]         every trail in the store, by day
celis tree --demo [--full] [--all]   made-up trails in every state, as tree prints or watches them
```

**Rules**

- `tree` MUST refuse an id no step has, with `no-step`. `tree --all` with an id MUST refuse as a usage
  error.
- `tree` with no id MUST print each trail of every project that is active or has a record from today,
  in any folder. A day MUST start at midnight, local time. `tree --all` MUST print every trail in the
  store, of every project.
- With no id and no trail to print, `tree` MUST print `No active trails. Run \`celis tree --all\` to see
  every trail.`, and with `--json` print nothing.
- With no id, `tree` MUST start each day with a line `<weekday>, <month> <day> · <n> active · <n> done`,
  as `Tue, Oct 14 · 2 active · 12 done`: today's with no `--all`, else each day's of the trails it
  started, newest first. `done` counts the trails that are not active.
- Under a day, its active trails MUST come first, newest first, each with its project. Then the others
  MUST group by project: a line with the project's name, then its trails, newest first. The projects MUST
  follow the newest trail of each. A blank line MUST come before each day, the first project and each
  project after it, but the first line. In the agent view, an active trail's first line MUST end with
  ` · <project>`.
- A trail whose first ask holds its text MUST print as one step with that ask: the ask's proof, grade
  and reuse as the trail's, and the ask's steps first under it (CIP-04, "Sessions"). A trail that is done
  or dropped itself MUST show its own done or drop. Its ask MUST NOT print, except with `--json`.
- `tree` MUST mark a step or a trail `○` when open, `◐` when working, `✓` when done, `✗` when dropped,
  and `!` when expired (CIP-03, "Starting"; CIP-04, "State"). Its mark MUST be its only sign of its
  state. Its flags MUST follow its mark: `↻` when it is revised (CIP-06, "Revisions"), then `~` when
  its done has a `changed` tag. A trail takes its first ask's `changed`, and is revised when a step of
  its tree is. Flags MUST NOT change the mark. With `--json`, a step MUST hold `revised_by`, the ids of
  the steps that revise it, and a trail `revised`, the ids of the revised steps of its tree.
- The worked time of a step MUST be the sum, over each step of its tree with no step under it, of the
  time a done one took, and of the time since a working one started. A worked time of none
  MUST be left out.
- The agent view MUST print each trail as a block: with no id and no `--all`, every trail; `tree <id>`
  its step; `tree --all`, with `--full` only. Else a trail MUST print as one line: `<mark> <id, first 8>
  <title>`, then ` · ` and its progress and worked time, as on a block's second line.
- A block MUST start with `<mark> <id, first 8> <title>`. Two spaces in, then come: `<done>/<all> steps`
  (`step` for one) when steps are under it and `<time> worked` when it has a worked time, ` · ` apart, on
  one line; then the steps under it that are open or working; each rule as `§ <id, first 8> <text>`; then
  the other steps under it. Each step MUST be followed by its own, two spaces further in, in the same
  order. Steps of one group MUST print newest first. An ask MUST print its trail's rules.
- A step MUST print as `<mark> <id, first 8> <title>`, then ` · <place>` with its first place whole and
  ` · <worked>` with its worked time.
- Two spaces further in than a step come its lines: `~ <changed>` for a done step with a `changed` tag,
  and `dropped <reason>` for a dropped one. With `--full`, they are the lines of its text after its
  title, then `done <proof>`, `changed <what changed>` and `took <its grade>` (CIP-03, "Grades"),
  `follows <id, first 8> <title>` for each step it follows, `reused <n>[ · saved <time>]`, and `dropped
  <reason>`, each label padded to 8 characters. A trail's own lines come under its title the same way.
- The agent view MUST NOT cut or wrap a line, MUST NOT pad one, and MUST NOT print colour.
- `tree` MUST NOT write a record.
- `tree --demo` MUST print, or watch, made-up trails as `tree` does: today's in two projects, and one of
  a past day, with a step or a trail in each state, two working steps of two projects, a `changed` tag, rules, a dropped reason, a done step
  with steps under it, and a working one. It MUST write them in a folder of its own, which it MUST
  remove when it ends, and MUST NOT read or write the person's store. `tree --demo` with an id MUST
  refuse as a usage error. The made-up trails MUST also hold a trail a day from 27 to 3 days ago, which
  use rules, follow the steps of a week before, and change and revise less in the later two weeks.

```
tree   text  <weekday>, <month> <day> · <n> active · <n> done
             
             <project>
             <mark> <id, first 8> <title>
               <done>/<all> steps · <worked> worked
               done    <proof>                                   --full, and changed, took, follows, reused
               <mark> <id, first 8> <title> · <place> · <worked>  open or working, newest first
                 <mark> <id, first 8> <title> · <worked>
               § <id, first 8> <text>                            each rule
               <mark> <id, first 8> <title> · <place> · <worked>  the others, newest first
                 ~ <what changed>
                   dropped <reason>
             <mark> <id, first 8> <title> · <done>/<all> steps · <worked> worked      --all
       json  {"trail": {"id", "created_at", "project", "content", "state", "links", "places", "proof", "grade",
             "revised_by", "reused", "revised", "reason"}}
             · {"step": {<as "Step" gives it>, "state": "open | working | done | dropped | expired",
             "proof", "changed", "grade", "revised_by", "reused", "reason"}}, one a line
grade  text  took <45s | 1m 30s | 2h 5m | 3d 2h>[ · revised by <id, first 8>, ...]
       json  "grade": {"took": <seconds>}
```

**Examples**

```
celis tree | cat

Tue, Oct 14 · 1 active · 1 done

◐ 3fa2c1d0 Add a senior tier to the price list · shop
  1/4 steps · 8m 42s worked
  ○ 6c1b8e52 Round the price down
  ◐ 5d7f3c21 The frame shows the tier · 7m 12s
    ○ 0a4e9d10 Add the badge to the frame
  § 4c1d22e0 A senior pays 80% of the price, rounded down to the cent.
  ✓ 9b0e11aa A senior pays 80% · shelf/prices.py:40 · 1m 30s

crm
✓ 1c9e44b2 Fix the cart total
  3/3 steps · 8m worked
  ...
```

## Watching

On a terminal, `tree` shows the trails while the agents work on them, so the person sees each step start
and close, and opens what they want to read.

**Rules**

- On a terminal, `tree` with no `--json` MUST watch. Else it MUST print once. `tree` MUST NOT take a
  `--watch` flag. A watch MUST refuse an id no step has, with `no-step`, also with no store.
- A watch MUST print the trails `tree` with the same words prints, in its days and projects, but as the
  rules below say. With no id and no trail to print, it MUST print what `tree` prints.
- A trail of a day with no `--all` that is active MUST print as a block, and every other trail as one
  line. With `--full`, every trail MUST print as a block. `tree <id>` MUST print its step as a block.
- A trail's line MUST start with `▸ `, and a block with `▾ `, then the step's mark and title. A block's
  title MUST wrap between words at the width, and MUST NOT be cut; an active trail's project MUST end its
  first line. Under it, at the title's column after a dim `│ `, come its lines: `took <worked time>`,
  `active <time> ago` from the newest record of its tree while it is active, the project with an id, then the lines the agent view prints with `--full`
  but its `took`. Then comes its tree, under its mark: its steps that are open or working, newest first,
  each followed by its own; a line `§ <n> rules` (`rule` for one) when it has rules; then a line
  `<marks> <n> closed steps` (`step` for one) when its other steps are done, dropped or expired. A step
  MUST print the steps under it the same way, with no rules line. A blank line MUST come between
  a block and the trail before or after it.
- The tree MUST draw each line of a level with a dim `├─ ` before it, and the last one with `└─ `. A
  line that opens MUST put `▸` in place of the `─` while closed and `▾` while open. A level down MUST
  start under its parent's mark, after a dim `│  ` while the parent has a line after it in its level,
  else after three spaces. A line under a step MUST start at its title's column, after a dim `│` under
  its mark when steps print under it.
- A step and a trail MUST print their flags after the mark, as `tree` does (Tree): `↻` in green, `~`
  in yellow.
- A line opens when it hides or shows something: a trail, the rules line, the closed line, a step with
  lines under it, and a done or dropped step with steps under it.
- The `§ <n> rules` line MUST be open with `--full`, else closed. Open, it MUST hold a line under it for
  each rule, `§ <text>`, the text wrapped between words.
- The closed line MUST hold the done, dropped and expired steps of its level, and print them, newest
  first, only when it is open. Its marks MUST be `✓` when one is done, then `✗` when one is dropped,
  then `!` when one is expired. It MUST be open with `--full`, else closed.
- A done step with steps under it MUST print them only when it is open; closed, its line
  MUST hold `<n> steps` for the steps of its tree.
- A step line MUST end with its details: `<n> steps` when it is closed, and its worked time while it is
  not done or dropped. It MUST NOT print its places. A trail's line MUST end with `<n> steps`
  when every step is done, else `<done> of <all>`, then `1 rule` or `<n> rules`, then its worked time,
  each in its own column. The steps and the rules MUST align their numbers right, `1 step ` and `1 rule `
  with a space for the missing `s`.
- The details MUST print in columns, two spaces apart, the same for every line of a day, ending at the
  right edge. A time and a line's last detail MUST align right, the others left. A column no line fills MUST be left out, and so
  MUST the columns a line leaves empty after its last detail, so its details end at the right edge. A
  line's title MUST be cut only at its first detail, so it runs into the columns it leaves empty before
  it, and end with `…` when cut.
- Under 80 columns, while a title would be cut below 20 characters, it MUST leave out the first column,
  then the time.
- With `--full` or when a step is open, its title wraps between words, and under it come the lines the
  agent view prints with `--full`, each after its label padded to 8 characters and wrapped at the width,
  each line at the same column. A closed step MUST NOT print a line under it.
- It MUST print in colour, but not when `NO_COLOR` is set; with no colour, it MUST print the same lines.
- In colour, a working step MUST print its mark in yellow and its title bold. A done step
  MUST print its mark in green and its title in normal weight. A dropped or expired step MUST print its
  mark in red and its title dim. A block's title, a day's line and a project's name MUST print bold. A step or trail whose newest record is less than 2 seconds old MUST print its title bold.
  Details, tree lines and the lines under a step MUST print dim. Each `§` MUST print magenta.
- A working step's or trail's mark MUST be a spinner, `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`, one frame each 100 ms, in
  bright yellow.
- With no id, the last line MUST be `▸ Show more`, after a blank line, or `▾ Show less` with `--all`.
- It MUST print the tree again as soon as the terminal's width changes. It MUST print it again within
  one second after a time it shows changes.
- Between two trees, it MUST sleep until a record lands, a time it shows changes, the width changes, or
  the spinner's next frame. With no time that runs, it MUST NOT wake on a timer.
- It MUST print the tree again within one second after a record lands in the store, or in a linked
  folder that is on (CIP-01, "Links"), and only when the tree changed. For a record in the store, it
  MUST read only the records it has not read.
- It MUST draw on the terminal's alternate screen. It MUST draw each tree from the top left over the last
  one, each line cut to the width and cleared to its end, and clear the rest of the screen, with no clear
  first. No line MUST reach the last column, so clearing to its end never erases its last character.
- It MUST NOT print ids. It MUST start each line with a column of two characters: `▌ ` on each line of
  the selected trail, rules line, closed line, rule, step or `Show more` line, up to the next one or a
  blank line, else two spaces. The `▌` MUST print in the text's colour, and the selected one's title
  bold; its dim lines MUST stay as they are.
- When the selected line leaves the tree, the watch MUST select the nearest line it was under that is
  still in the tree, else the first line, and show it.
- A tree taller than the rows above the key line MUST scroll. A first row `↑ <n> more` MUST stand for
  the lines above, and a last row `↓ <n> more` for the lines below. The view MUST move to show the
  selected line when the selection moves, and keep its place when the tree is drawn again.
- It MUST run until it is stopped, and MUST NOT write a record. On an interrupt it MUST leave the
  alternate screen, print nothing, and exit 0.
- When stdin is a terminal, a watch MUST read keys, with no echo: `↑` and `↓` select the line before or
  after the selected one, of the trails, rules lines, closed lines, rules, steps and `Show more` in the
  order it prints them, and the first `↓` the first trail. `→` opens the selected one: a trail prints as
  a block, a rules line shows its rules, a closed line its steps, a step wraps its title and shows the
  steps and the lines under it, and `Show more` turns `--all` on. `←` closes it; on one that is closed,
  `←` selects the line it is under. `enter` opens the selected one when it is closed, and else closes it. `*`
  opens the selected one and every rule and step under it, and closes them all when all are open. `c`
  copies the first 8 characters of the selected trail's, rule's or step's id with the first of
  `pbcopy`, `wl-copy` and `xclip -selection clipboard` that runs. `m` copies the selected one as Markdown
  the same way, as `celis report` writes it: a trail all of `celis report <id>`, a step the table of it
  and the steps under it, a rule `- <text>`. `PgUp` and `PgDn`, or `space` for down,
  scroll one page; `g` and `G` scroll to the top and the end; `f` turns `--full` on and off; `a` turns
  `--all` on and off with no id; `q` ends it as an interrupt does.
- It MUST print on the screen's last row a dim line, cut to the width:
  `↑↓ select · ←→ close open · * open all under · c copy <id, first 8> · m copy markdown · PgUp PgDn scroll
  · f full · a all · q quit`, with no `c copy` and `m copy markdown` while no trail, rule or step is
  selected. After `c` or `m`, until the next key, the line MUST be `copied <id, first 8>` or `copied
  markdown`, or `no pbcopy, wl-copy or xclip to copy with`.

**Examples**

```
celis tree

  Tue, Oct 14 · 1 active · 1 done

  ▾ ◐ Add a senior tier to the price list                                          shop
    │ took    8m 42s
    │ active  1m ago
    ├▸ § 1 rule
    ├▸ ✓ 1 closed step
    ├─ ◐ The frame shows the tier                                              7m 12s
    │  └─ ○ Add the badge to the frame
    └─ ○ Round the price down

  crm
  ▸ ✓ Fix the cart total                                      3 steps                8m

  ▸ Show more
↑↓ select · ←→ close open · * open all under · PgUp PgDn scroll · f full · a all · q quit
```

## Past

`past` prints what a new ask of the project must keep, then the past work of every project (CIP-04,
"The past").

**Rules**

- `past` MUST print the working folder's project's active trails, oldest first, each with its rules.
- Then it MUST print the live decisions at each `--at` place, then the project's live `project` and `me`
  decisions (CIP-01, "Reading"). It MUST NOT print a decision at a place unless an `--at` place is at it.
- Then it MUST print the past work of every project, newest first.
- Each past work MUST print as one line: its id, project, title and place, its grade or else its state
  and ` · revised` when its tree holds a revised step (CIP-06, "Revisions"), and its day. Then it MUST print the rules of the trail it stands for (CIP-04, "The past").
- With `--json`, each object MUST be named `past`, with its id, project, title, places, state, grade,
  revised, day and rules.
- Without `--json`, when it prints nothing else, `past` MUST print one line that says so.
- `past` MUST NOT write a record.

```
past  text  <id, first 8> [<project name>] <its title>[ · <place>]          each active trail
              rule <id, first 8> <text>                                     each of its rules
            <a decision, as "Commands" gives it>
            <id, first 8> [<project name>] <its title>[ · <place>] · <grade | state[ · revised]> · <yyyy-mm-dd>
              rule <id, first 8> <text>                                     each rule of its trail
      json  {"trail": {"id", "created_at", "project", "content", "session", "rules": []}}
            · {"decision": {...}} · {"past": {"id", "created_at", "day", "project", "content",
            "places", "state", "grade", "revised", "rules": []}}
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
- `stats --demo` MUST print or draw, as `stats` does, the made-up trails of `tree --demo`, in a folder it
  removes when it ends. It MUST NOT read or write the person's store. With an id it MUST refuse as a
  usage error.
- On a terminal with no `--json`, `stats` MUST draw its cards on the alternate screen instead, as "Cards"
  gives them, and again when a record lands, a key is read, a minute passes or the size changes. With an
  id, it MUST draw the route of the step instead: each step it follows under `FOLLOWS`, the step as a
  card with how long it took, then each step that follows it under `FOLLOWED BY` with what it saved.
- On a terminal with no `--json` and no id, when no step has closed yet, `stats` MUST draw the header and
  a `NO STATS YET` card that names `celis stats --demo`, with the key line `q quit`, and read no other
  key. With no store, it MUST write nothing and wait for one, as `tree` does. It MUST draw the cards
  when the first step closes.

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

### Cards

Each card shows one kind of number, in the form of its data: a series as columns, a rank as bars, a share
as a strip, a chain as a route. The periods are CIP-06 "Periods".

```
00 EFFECT · LAST 14 DAYS vs 14 BEFORE   as planned, rework, rules reused and time saved; each its value,
                                        its change from the period before and its base
01 REUSE                                the follows and what they saved; the 3 steps followed most
02 RULES USED                           the 5 decisions used most, as bars, and the count of uses
03 WORK · 14 DAYS                       the steps done each local day, as columns, and their total
04 TIME A STEP TOOK                     the median took of the done steps with none under them, and a bar
                                        for each of < 5m, 5-15m, 15m-1h, 1-4h and > 4h
05 OUTCOME                              the closed steps as a strip: done, dropped, expired; revised beside done
06 PROJECTS · TIME                      the took of the done steps with none under them, a project a bar
```

**Rules**

- The effect MUST print across the width, then the other cards three a row at 120 columns or more, two
  at 80 or more, else one. Below 60 columns, numbers MUST print as plain bold text, else three rows high.
- A change MUST print green when better, red when worse, dim when it moved less than 3 points or a
  minute, `· no data before` when the 14 days before count no record, and `· few records` when CIP-06
  "Periods" shows no other change.
- A big value with no digit MUST print on its middle row, and a unit MUST print right after its last
  digit, as `100%`.
- The work card MUST print `first 14 days` in place of its change when the 14 days before hold no done
  step, and a dim `·` for a day with no done step. With no step done in 14 days, the work and time cards
  MUST print `no step done in 14 days`, and the outcome card with no step closed `no step closed in 14
  days`, in place of their charts.
- The picked card MUST print in a heavy frame, the others in a dim one.
- Keys: `←` `→` `↑` `↓` pick a card; `enter` opens it to its table, a row a line, where `↑` `↓` select a
  row, `c` copies the first 8 characters of the selected row's id, and `←` or `enter` goes back. On the
  route, `↑` `↓` select a step, `enter` goes to its route and `←` back. `m` copies the picked card, the
  opened card or the route as a Markdown table under a `###` heading. `q` ends it as an interrupt does.
- The last row MUST print the keys, dim: `←→↑↓ move · enter open · m copy markdown · q quit`, and on an
  opened card or a route the keys it reads there, with `c copy <id, first 8>` when a row has an id.

## Report

`report` prints the trails the ids name as Markdown (CIP-05, "The report").

**Rules**

- `report` MUST refuse an id no step has, with `no-step`. It MUST refuse no id as a usage error.
- `report --demo` MUST print or draw, as `report` does, the active trail of the made-up trails of `tree
  --demo` that has a step in each state, in a folder it removes when it ends. It MUST NOT read or write
  the person's store. With an id it MUST refuse as a usage error.
- `report` MUST print the trail of the step each id names, once, in the order of the ids. A blank line
  MUST come between two trails.
- `report` MUST print a trail's title as a heading, then a table of the trail, then a table of its steps,
  each with its mark as `tree` marks it, two no-break spaces and `└ ` a level in. Then a details block
  of the texts, places and checks of its steps, and one of its rules. A section with no line MUST be left
  out. A `|` in a cell MUST be escaped.
- On a terminal with no `--json`, `report` MUST print a card a trail instead: its id, project and title,
  how long it ran, its counts and state, a strip of a mark a step in its state's colour, a line a step
  under it with how long it took, and its rules and lessons. Then, when stdin is a terminal, the dim line
  `m copy markdown · any key to leave`, and read one key: `m` copies the Markdown as `tree`'s `m` does
  and prints `copied markdown`; any key clears the line and ends.

```
report  text  ## <title of the trail>

              | Id | State | Took | Steps | Rules | Revised | Dropped | Project |
              |----|-------|-----:|------:|------:|--------:|--------:|---------|
              | `<id, first 8>` | <mark> <state> | <ran> | <n> | <n> | <n> | <n> | <project name> |

              <the next line of its text>

              done: <proof>

              | | Step | Took | Proof |
              |-|------|-----:|-------|
              | <mark> | <title> | <took> | <proof>[ · ~ changed: <what changed>] |   each step
              | <mark> | <title> | — | dropped: <reason> |

              <details><summary>Texts, places and checks</summary>

              - <mark> <title>                                            each step with any
                - <the next line of its text>
                - at `<place>`
                - check passed `<command>` | check `<command>`

              </details>

              <details><summary><n> rules</summary>

              - <text>                                                    each rule

              </details>
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
