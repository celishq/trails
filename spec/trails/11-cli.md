---
cip: 11
title: The command line
status: draft
depends: [CIP-10]
---

# CIP-11 — The command line

## Commands

```
celis read [--at <place>]... [--like <text>]          this project's active trails, open steps and decisions
celis decide [--scope <scope>] [--replaces <id>] -- <text>
                                                       a decision, in its scope
celis tree [<id>] [--full]                            a trail and its steps (CIP-14)
celis step add | start | done | steer | drop | edit   the steps of a trail (CIP-12)
celis watch [<id>] [--full]                           a trail's tree, again on each change (CIP-14)
celis home                                            the folder of the store, to open it
celis prompt                                          a person's prompt, from a hook (CIP-14)
celis end                                             the end of a session, from a hook (CIP-14)
celis join <id>                                       join this session to a trail (CIP-14)

$CELIS_HOME   where celis keeps the key and the store
              default: ~/Library/Application Support/Celis
$CELIS_TRAIL  the trail new steps go under (CIP-14)
```

**Rules**

- Off macOS, `$CELIS_HOME` MUST default to `~/.local/share/celis`.
- `read` MUST print the working folder's project's active trails, oldest first, each with its rules. The
  pitfall of each done step of its tree MUST come next. Then come its open steps that are no trail, oldest
  first. Then come its live decisions that are no rule
  (CIP-10, "Reading"), 30 at most. The decisions at each `--at` place MUST come first.
- When it leaves decisions out, `read` MUST then print `<n> more decisions`, or `{"more": <n>}` with
  `--json`.
- `read` MUST NOT print a rule of a reached trail, or write a record.
- `home` MUST print the full path of the store, `$CELIS_HOME/trails`, and write nothing. With `--json`, it
  MUST print `{"path"}`.
- With `--like`, `read` MUST then print the past trails like the text (CIP-14, "The past").
- `decide` MUST write a decision on the working folder's project (CIP-10, "Decisions"), with
  `--scope` and `--replaces` as its tags. With `--scope project`, or no `--scope`, it MUST write no
  `scope` tag.
- `decide` MUST write the whole id of the trail that `--scope trail:<id>` names.
- `decide` MUST refuse empty text, and a scope that is not `me`, `project`, `trail:<id>` or `at:<place>`,
  as a usage error. It MUST refuse `trail:<id>` when no active trail of the project has the id, with
  `no-trail`. It MUST refuse a `--replaces` no live decision has, with `no-decision`.
- A flag no command takes is a usage error.

```
decision  text  <id, first 8> <scope> <text>
          json  {"id", "created_at", "project", "scope", "replaces", "content"}
read      text  <id, first 8> [<project name>] <the trail's first line>      each active trail
                  rule <id, first 8> <text>                                 each of its rules
                  pitfall <id, first 8> <text>                              each pitfall of its tree
                <an open step, as CIP-12 "The commands" gives it>
                <a decision>
                <a past trail, as `tree` prints it>                         with `--like`
          json  {"trail": {"id", "created_at", "project", "content", "rules": [], "pitfalls": [{"id",
                "text"}]}} · {"step": {...}}
                · {"decision": {...}} · {"past": {...}}, one a line, each as its own json
```

**Examples**

```
celis decide --scope at:shelf/prices.py -- "When you change prices: keep them in cents, because floats lost a cent."
celis decide --scope me -- "When you write a commit message: use plain words, because the team reads them."
celis read --at shelf/prices.py --like "Add a senior tier to the price list"
```

## Help

The page is the reference for a person.

```
celis  ·  celis help  ·  celis --help     the page
celis <command> --help                    the command page
celis --version                           celis <version> (<commit, first 8>)
```

The page:

```
celis keeps how you do your tasks. A trail is one task you give, the steps
agents take toward it, and the decisions made on the way. Use it through the
/celis:trail skill.

Commands
  read     this project's active trails, open steps and decisions
  decide   a decision, in its scope
  tree     a trail and its steps, with their grades
  step     the steps of a trail: add, start, done, steer, drop, edit
  watch    a trail's tree, printed again on each change
  home     the folder of the store, to open it
  prompt   a person's prompt, from a hook
  end      the end of a session, from a hook
  join     join this session to a trail

$CELIS_HOME holds the key and the store. $CELIS_TRAIL names the trail new steps
go under.

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
- A command page MUST give a line to each argument and flag its usage names, and at least one example.
- Every help page MUST exit 0, and print to stdout.
- A line of a help page MUST NOT be longer than 80 characters.
- A help page MUST NOT name a spec file.
- `celis --version` MUST print `celis <version>`, and the commit it was built from, when the build had one.

## Output and errors

**Rules**

- A command MUST take an id whole, or as its first 4 or more characters when only one record's id
  starts with them.
- Every command MUST accept `--json`, and then print one JSON object a line, and nothing else, to stdout.
- A command that writes a record MUST print its id. With `--json`, it prints `{"id"}`.
- A command MUST NOT ask a question.
- A usage error MUST name the narrowest help page that covers it: `celis <command> --help`, or else
  `celis help`.
- An unknown command MUST name the known one nearest it, when two edits or fewer turn one into the other.
- A command MUST exit 0 when done, 2 on a usage error, and 1 on any other error.
- Without `--json`, an error MUST be one line on stderr: `celis: <message>`.
- With `--json`, an error MUST be one object on stderr: `{"error": "<name>", "message": "<text>"}`.
- An error MUST be named `usage`, `no-decision`, `no-trail`, `no-step`, `closed`, `open-steps`,
  `check-failed` (CIP-12), else `error`.

**Examples**

```
$ celis raed
celis: no command "raed"; did you mean "read"? See `celis help`.
```
