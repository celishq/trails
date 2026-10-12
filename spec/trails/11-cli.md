---
cip: 11
title: The command line
status: draft
depends: [CIP-10]
---

# CIP-11 — The command line

## Commands

```
celis read [--at <place>]...                          this project's active trails, open steps and decisions
celis decide [--scope <scope>] [--replaces <id>] -- <text>
                                                       a decision, in its scope
celis tree [<id>] [--full] [--all]                    a trail and its steps (CIP-14)
celis past                                            every project's past work, one line each (CIP-14)
celis step add | start | done | steer | drop | edit   the steps of a trail (CIP-12)
celis watch [<id>] [--full]                           the active trees, again on each change (CIP-14)
celis home                                            the folder of the store, to open it
celis prompt                                          a person's prompt, from a hook (CIP-14)
celis auth login                                      keep the phrase stdin gives (CIP-10)
celis auth status                                     the account's npub (CIP-10)
celis auth phrase                                     print the phrase (CIP-10)

$CELIS_HOME   where celis keeps the phrase and the store
              default: ~/Library/Application Support/Celis
$CELIS_TRAIL  the trail new steps go under (CIP-14)
```

**Rules**

- Off macOS, `$CELIS_HOME` MUST default to `~/.local/share/celis`.
- `read` MUST print the working folder's project's active trails, oldest first, each with its rules. Then
  come its ended trails with a record from today that are not done or dropped, each with its state and
  rules. Then come its open steps that are no trail, oldest first. Then come the live decisions at each
  `--at` place, then its live `project` and `me` decisions (CIP-10, "Reading"), 30 at most.
- `read` MUST NOT print a decision at a place unless an `--at` place is at it. `step start` prints those.
- When it leaves decisions out, `read` MUST then print `<n> more decisions`, or `{"more": <n>}` with
  `--json`.
- `read` MUST NOT print a rule of a done or dropped trail, or write a record.
- `home` MUST print the full path of the store, `$CELIS_HOME/trails`, and write nothing. With `--json`, it
  MUST print `{"path"}`.
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
read      text  <id, first 8> [<project name>] <its title>[ · <place>]      each active trail
                  rule <id, first 8> <text>                                 each of its rules
                <id, first 8> [<project name>] <its title>[ · <place>] · <state>   each of today's ended trails
                  rule <id, first 8> <text>                                 each of its rules
                <an open step, as CIP-12 "The commands" gives it>
                <a decision>
          json  {"trail": {"id", "created_at", "project", "content", "rules": []}}
                · {"recent": {"id", "created_at", "project", "content", "rules": [], "state"}} · {"step": {...}}
                · {"decision": {...}}, one a line, each as its own json
```

**Examples**

```
celis decide --scope at:shelf/prices.py -- "When you change prices: keep them in cents, because floats lost a cent."
celis decide --scope me -- "When you write a commit message: use plain words, because the team reads them."
celis read --at shelf/prices.py
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
agents take toward it, and the decisions made on the way. Use it through the
/celis:trail skill.

Commands
  read     this project's active trails, open steps and decisions
  decide   a decision, in its scope
  tree     a trail and its steps, with their grades
  past     every project's past work, one line each
  step     the steps of a trail: add, start, done, steer, drop, edit
  watch    a trail's tree, printed again on each change
  home     the folder of the store, to open it
  auth     the account's phrase: login, status, phrase

$CELIS_HOME holds the phrase and the store. $CELIS_TRAIL names the trail new
steps go under.

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
- Each command line of the `/celis:trail` card MUST name the flags its command's usage names,
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
  `check-failed` (CIP-12), else `error`.

**Examples**

```
$ celis raed
celis: no command "raed"; did you mean "read"? See `celis help`.
```
