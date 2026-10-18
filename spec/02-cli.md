---
cip: 2
title: The command line
status: draft
depends: [CIP-01]
---

# CIP-02 — The command line

`celis` writes and reads the records of the other CIPs. This CIP gives what the plugin, its hooks and
scripts depend on: the usage of each command, the records it writes, its errors, and the output they
read. What a command shows a person is not part of it.

## Commands

```
celis decide [--scope <scope>] [--replaces <id>] -- <text>
                                                       a decision, in its scope
celis tree [<id>] [--graph] [--full] [--all] [--demo]
                                                       a trail and its steps; again on each change
celis grep <grep option>... <pattern>                 every trail, through the system grep
celis stats [<id>] [--demo]                           what later work reused
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

```
decision  json  {"id", "created_at", "project", "scope", "replaces", "content"}
```

**Rules**

- Off macOS, `$CELIS_HOME` MUST default to `~/.local/share/celis`.
- A flag no command takes is a usage error.
- `decide` MUST write a decision of the working folder's project (CIP-05), with `--scope` and
  `--replaces` as its tags, and no `scope` tag for `project`.
- `decide` MUST refuse empty text, and a scope that is not `me`, `project`, `trail:<id>` or `at:<place>`,
  as a usage error; a `trail:<id>` that no open trail of the project has, with `no-trail`; and a
  `--replaces` that no live decision has, with `no-decision`.

**Examples**

```
celis decide --scope at:shelf/prices.py -- "When you change prices: keep them in cents, because floats lost a cent."
celis decide --scope me -- "When you write a commit message: use plain words, because the team reads them."
```

## Help

**Rules**

- `celis <command> --help` MUST print a page that starts with the command's usage lines from "Commands".
- Each command line of the plugin's card MUST name the flags its command's usage names, and no other.
- The first line of `celis --version` MUST be `celis <version>`, then a space, or its end.

## Output and errors

**Rules**

- A command MUST take an id whole, or as its first 4 or more characters when only one record's id starts
  with them. It MUST refuse a step id that no step has, or that starts more than one step's id, with
  `no-step`.
- A command that writes records MUST print the id of each, its main record first.
- With `--json`, a command but `grep` MUST print one JSON object a line, and nothing else, to stdout. A
  printed id is `{"id"}`.
- Only `decide`, `step` and `prompt` MAY write a record. A command that refuses MUST write nothing.
- A command MUST NOT ask a question, but `auth login` on a terminal.
- A command MUST exit 0 when done, 2 on a usage error, and 1 on any other error.
- An error MUST be named `usage`, `no-decision`, `no-trail`, `no-step`, `closed`, `open-steps`,
  `check-failed`, else `error`. With `--json`, it MUST be one object on stderr,
  `{"error": "<name>", "message": "<text>"}`.

## Update

`update` runs the latest release's `$CELIS_BASE/latest/download/install.sh` with `sh`.

```
$CELIS_BASE   where the releases are; default: https://github.com/celishq/trails/releases
```

**Rules**

- `update` MUST run `install.sh` with `CELIS_UPDATE=1`, `CELIS_BASE`, `CELIS_VERSION` set to `<version>`
  or `latest`, and `CELIS_INSTALL_DIR` set to the folder of the file the running `celis` resolves to.
- When `install.sh` fails, `update` MUST exit with its code.
- With `CELIS_UPDATE` set, `install.sh` MUST NOT change PATH.

## Auth

**Rules**

- `auth login` MUST read the phrase from stdin, refuse one given as words of the command as a usage
  error, and refuse a text that is not a BIP-39 phrase, keeping the phrase it had.
- `auth status` and `auth phrase` MUST NOT mint a phrase.

## Step

`step` writes the records of CIP-03, CIP-04 and CIP-06.

```
step  json  {"id", "created_at", "project", "links": [{"to", "kind": "under | replaces | uses"}], "places": [],
            "check", "content"}
```

**Rules**

- `add` MUST write the whole id of `--under`, `--replaces` and each `--uses`, each `--at` (CIP-01,
  "Places") and the `--check` as tags. A `--replaces` that names a trail MUST name its first ask.
- `add` MUST write no step when an open step of the same project and `under` has the same text, and
  print that step's id. With `--uses`, or a `--replaces` of a done step, it MUST refuse it as a usage
  error.
- `add` of a session's first ask MUST print the ask's id, then the trail's.
- `add` MUST refuse empty text as a usage error; a closed `--under`, and a `--replaces` of a dropped step,
  with `closed`; a `--replaces` of a trail with no first ask as a usage error; a `--uses` that no live
  decision and no step has, with `no-step`; and a `--uses` of a step of its own trail, or that starts both
  a decision's and a step's id, as a usage error.
- `start`, `done` and `drop` MUST refuse a closed step, with `closed`. `start` of a working step MUST
  write nothing and print the id of its first `start`.
- `done` MUST take one of `--changed <text>` and `--as-planned`, and a proof, else refuse as a usage
  error. It MUST refuse a step with an open step under it, with `open-steps`.
- `done` MUST run the step's `check` in the project's top folder, and refuse with `check-failed` when it
  fails or runs past `CELIS_CHECK_SECONDS`, 600 when not set.
- `done --changed` MUST write its text as `changed`. `done --pitfall` MUST also write a decision, at
  `at:<the step's at>`, else `trail:<the step's trail id>`.
- `drop` MUST refuse an empty reason, and `edit` one with no text, `--at` or `--check`, as usage errors.

**Examples**

```
celis step add --under 3fa2c1d0 --at shelf/prices.py:40 -- "Write a test: a senior pays 80% of the price."
celis step start 4c1d22e0
celis step done 4c1d22e0 --as-planned -- "python3 -m unittest: test_senior failed before, passes now."
celis step drop 3fa2c1d0 -- "The person chose the shared price table instead."
```

## Prompt

Each agent's hooks give their JSON to `prompt`.

```
{"session_id", "prompt"}                 a prompt; Cursor names the session `conversation_id`
{"session_id", "source": "compact"}      a session start after a compaction
```

**Rules**

- `prompt` MUST read the hook's JSON on stdin, with `conversation_id` in place of `session_id` when it
  has one.
- On a prompt, it MUST write it as CIP-04 "Sessions" states, and print the record's id. On a compaction,
  it MUST write and print nothing.
- A prompt that names a step with `celis step <id>` is a task for that step. With no open trail, `prompt`
  MUST NOT ask for the past agent on it.

**Examples**

```
echo '{"session_id": "95e9be99", "prompt": "Use the shared Badge."}' | celis prompt
```

## Grep

`grep` gives every trail of every project to the system `grep`, as lines. The plugin's `past.md` searches
the trails with it.

```
view    <yyyy-mm-dd> <trail id8 | -> [<project name>] <kind> <id8> <fields> <text>
          trail | step   <open | working | done | dropped | expired>[ revised-by <id>,…][ at:<place>]…
          rule           <live | replaced-by <id>> {<me | project | trail:<id8> | at:<place>>}
          proof | changed | dropped                              the step's id, no fields
lines   <id8> [<project name>] <its title> · <yyyy-mm-dd>        each trail of the lines
          trail <id8> <fields>                                    the trail's own line
          <kind> <id8> <fields> <text>                            each other line of the trail
        - rules of no trail
          [<project name>] rule <id8> <fields> <text>             each line with no trail
cut     celis: <n> more lines in <n> trails match; narrow the search, or search one trail with celis grep " <id> \["
        <id8> [<project name>] <its title, cut at 80> · <n> <kind>s, …   each trail cut
        - rules of no trail · <n> rules                          the lines with no trail, when cut
        … <n> more trails
end     project [<project name>]
```

`<id8>` is the first 8 characters of an id. A text holds each newline as a space.

**Rules**

- `grep` MUST give each word after it to the system `grep` as it is, with the view on its stdin.
- The view MUST hold every trail of every project, newest first, and each live or replaced decision of a
  scope other than `trail:<id>` by when it was made. A trail MUST print as its line, then its rules,
  newest first, then its steps, each level newest first, each step with its proof, `changed` and drop
  lines, then its own steps.
- `grep` MUST print each matched line under its trail's header, the trails with most lines first, and each
  line with no trail under `- rules of no trail`. A matched line that is no line of the view MUST NOT
  print. When no matched line is a line of the view, as with `-c` or `-l`, it MUST print what `grep`
  prints, and nothing else.
- When the output is too long, `grep` MUST cut it at the end of a line, then index the trails it cut.
- `grep` MUST end with the `project` line of the working folder.
- When `grep` exits 1, `grep` MUST print `No line matches.` and exit 0. When it exits 2, it MUST exit 2.

**Examples**

```
celis grep -i -E "price|cents"
celis grep " live {me} "
celis grep -A3 9b0e11aa
```
