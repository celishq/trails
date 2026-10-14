---
cip: 2
title: The command line
status: draft
depends: [CIP-01]
---

# CIP-02 — The command line

## Commands

```
celis decide [--scope <scope>] [--replaces <id>] -- <text>
                                                       a decision, in its scope
celis tree [<id>] [--full] [--all]                    a trail and its steps; again on each change (CIP-04)
celis past [--at <place>]...                          this project's trails and decisions, past work (CIP-04)
celis report <id>...                                  the trails as Markdown, for a pull request (CIP-05)
celis issue [<id>]... [-- <text>]                     what went wrong, as Markdown for the celis team (CIP-05)
celis step add | start | done | drop | edit           the steps of a trail (CIP-03)
celis prompt                                          a person's prompt, from a hook (CIP-04)
celis auth login                                      keep the phrase stdin gives (CIP-01)
celis auth status                                     the account's npub (CIP-01)
celis auth phrase                                     print the phrase (CIP-01)
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
- With `<version>`, `update` MUST install `v<version>`, also when it is installed.
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
  `check-failed` (CIP-03), else `error`.

**Examples**

```
$ celis pats
celis: no command "pats"; did you mean "past"? See `celis help`.
```
