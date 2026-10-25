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
celis pattern [--scope <scope>] [--replaces <id>] [--path <glob>]... [--binds] [--within <id>]... [--good <example>]... [--bad <example>]... [--project <id>] -- <text>
                                                       a pattern, in its scope
celis patterns [--markdown]                           the patterns, the rules first, and their examples
celis tree [<id>]... [--graph] [--full] [--all] [--demo]
                                                       a trail and its steps; again on each change
celis grep <grep option>... <pattern>                 every trail, through the system grep
celis search [<file>]... [--kind <kind>] [--limit <n>] -- <ask>
                                                       the rules, the other patterns and the past asks that fit an ask
celis setup [--model <model>]                         the model that search uses
celis stats [<id>] [--demo]                           what later work reused
celis issue [<id>]... [-- <text>]                     what went wrong, as Markdown for the celis team
celis link [<folder>] [--off]                         keep this project's trails in a folder, or stop
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>]... [--check <command>] [--new] [--session <id>] [--project <id>] -- <text>
                                                       leave a step
celis step start <id>                                 take up a step, and see the patterns its plan links
celis step done <id> [--changed <text>] [--as-planned] [--by <who>] [--file <path>]... -- <proof>
                                                       close a step with its proof
celis step drop <id> -- <reason>                      end a step, a done trail or a pattern no longer wanted
celis step edit <id> [--at <place>]... [--check <command>] [-- <text>]
                                                       give a step a new title, places or check
celis hook start <agent> <plugin folder>              what an agent reads at a session start
celis hook ask <agent>                                a person's prompt, and what the agent reads on it
celis auth login                                      keep the phrase stdin gives
celis auth status                                     the account's npub
celis auth phrase                                     print the phrase
celis update [<version>]                              this binary, to the latest release or a version

$CELIS_HOME   where celis keeps the phrase and the store
              default: ~/Library/Application Support/Celis
```

```
decision  json  {"id", "created_at", "project", "scope", "replaces", "content",
                 "paths", "binds"}
```

**Rules**

- Off macOS, `$CELIS_HOME` MUST default to `~/.local/share/celis`.
- A flag no command takes is a usage error.
- `decide` MUST run as `pattern`, as a command and after `help`.
- `pattern` MUST write a pattern of the working folder's project (CIP-05), with `--scope` and
  `--replaces` as its tags, and no `scope` tag for `project`; each `--path`, from the project's top
  folder, as a `path` tag, `--binds` as its label, each `--within` as a `within` tag (CIP-05, "Graph"),
  refusing an id that no live pattern of the store has with `no-decision`, and each `--good` and `--bad`
  as an example (CIP-05, "Examples"): a value with `://` as a URL, a file of the working folder as its
  blob, else the whole id of a step, refusing a step the store lacks with `no-step`.
- `--project` on `pattern` and `step add` MUST write the records of the project it names in place of the
  working folder's: a project id that a record of the store has, with that record's project name. It MUST refuse any other id as a usage error.
- `pattern` MUST refuse empty text, and a scope that is not `me`, `project` or `trail:<id>`,
  as a usage error; a `trail:<id>` that no open trail of the project has, with `no-trail`, unless
  `--replaces` names a live pattern of that trail; and a
  `--replaces` that no live pattern of the project has, with `no-decision`.

**Examples**

```
celis pattern --path shelf/prices.py --binds -- "When you change prices
Forces: floats are simple, but they lose cents
Do: keep prices in cents
Because: floats lost a cent"
celis pattern --scope me -- "When you write a commit message
Forces: detail helps, but the team reads them fast
Do: use plain words
Because: the team reads them"
celis pattern --path "**/prices*.py" -- "When you change prices: keep them in cents, because floats lost a cent."
```

## Help

**Rules**

- `celis <command> --help` MUST print a page that starts with the command's usage lines from "Commands".
- Each command line of the plugin's card MUST name only flags its command's usage names.
- The first line of `celis --version` MUST be `celis <version>`, then a space, or its end.

## Output and errors

**Rules**

- A command MUST take an id whole, or as its first 4 or more characters when only one record's id starts
  with them. It MUST refuse a step id that no step has, or that starts more than one step's id, with
  `no-step`.
- A command that writes records MUST print the id of each, its main record first.
- With `--json`, a command but `grep` MUST print one JSON object a line, and nothing else, to stdout. A
  printed id is `{"id"}`.
- Only `pattern`, `step` and `hook ask` MAY write a record. A command that refuses MUST write nothing.
- A command MUST NOT ask a question, but `auth login` and `setup` on a terminal.
- A command MUST exit 0 when done, 2 on a usage error, and 1 on any other error.
- An error MUST be named `usage`, `no-decision`, `no-trail`, `no-step`, `closed`, `open-steps`,
  `check-failed`, `unjudged`, else `error`. With `--json`, it MUST be one object on stderr,
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
  pattern and no step has, with `no-step`; and a `--uses` of a step of its own trail, or that starts both
  a pattern's and a step's id, as a usage error.
- `start` and `done` MUST refuse a closed step, with `closed`. `start` of a working step MUST write
  nothing and print the id of its first `start`.
- `drop` MUST take the id of a step or a pattern. It MUST refuse a dropped step, a done step that is not a
  trail, a trail dropped after done, and a pattern that is not live, with `closed`; and an id that starts
  both a pattern's and a step's id as a usage error.
- `done` MUST take one of `--changed <text>` and `--as-planned`, and a proof, else refuse as a usage
  error. It MUST refuse a step with an open step under it, with `open-steps`.
- The changed files of a `done` are each file that `git diff --name-only HEAD` or
  `git ls-files --others --exclude-standard` gives in the working folder, and each `--file`.
- `done` of an ask, or of another step with steps under it, with no `--by` and a changed file MUST refuse
  with `unjudged`, and print the patterns of its changed files and the brief that the judge needs.
- `done` MUST run the step's `check` in the project's top folder, and refuse with `check-failed` when it
  fails or runs past `CELIS_CHECK_SECONDS`, 600 when not set.
- `done --changed` MUST write its text as `changed`, and `done --by` its value as `by`; a `--by` other
  than `judge` or `person` is a usage error. Each `done --file` MUST be kept as a blob and named by an
  `imeta` tag (CIP-08), and a file that is not there is a usage error.
- `done` of a step that is not an ask MUST print each live pattern in scope that a changed file matches,
  as "Search" gives a pattern, in the order "Search" states.
- `drop` MUST refuse an empty reason, and `edit` one with no text, `--at` or `--check`, as usage errors.

**Examples**

```
celis step add --under 3fa2c1d0 --at shelf/prices.py:40 -- "Write a test: a senior pays 80% of the price."
celis step start 4c1d22e0
celis step done 4c1d22e0 --as-planned -- "python3 -m unittest: test_senior failed before, passes now."
celis step drop 3fa2c1d0 -- "The person chose the shared price table instead."
```

## Prompt

Each agent's hooks give `hook` the JSON of a prompt or of a session start.

```
{"session_id", "prompt"}                 a prompt; Cursor names the session `conversation_id`
{"session_id", "source": "compact"}      a session start after a compaction
```

**Rules**

- `hook` MUST read the session from `conversation_id` when the JSON has one, else from `session_id`.
- On a prompt, it MUST write it as CIP-04 "Sessions" states. On a compaction, it MUST write no record.
- A prompt that names a step with `celis step <id>` is a task for that step.

## Patterns

`patterns` gives the live patterns in scope in the working folder's project (CIP-05).

```
<line>     each pattern, as "Search" gives it, the rules first, newest first within each
none       no pattern yet
```

**Examples**

```
celis patterns
celis patterns --markdown > patterns.md
```

## Hooks

Each agent's hooks give their JSON to `hook`, and the agent reads what it prints.

```
claude, codex   <text>
cursor          {"additional_context": "<text>"}; on ask, {"continue": true}
gemini          {"hookSpecificOutput": {"hookEventName": "SessionStart" | "BeforeAgent", "additionalContext": "<text>"}}
antigravity     {"injectSteps": [{"ephemeralMessage": "<text>"}]}
```

**Rules**

- `hook` MUST read the hook's JSON on stdin, and act on it as "Prompt" states. `hook ask`
  MUST act in the folder the JSON names in `cwd`, else in the first of `workspace_roots`.
- `hook start` MUST read the card from `card.md` and the plugin's version from `.claude-plugin/plugin.json`
  in `<plugin folder>`.
- `hook` MUST print in the shape of `<agent>`, and nothing when it has no text, but Cursor's ask.
- `hook` MUST exit 0 on any error but a usage error.

**Examples**

```
echo '{"session_id": "95e9be99", "cwd": "/repo"}' | celis hook start claude ~/.claude/plugins/celis
echo '{"session_id": "95e9be99", "cwd": "/repo", "prompt": "Use the shared Badge."}' | celis hook ask codex
```

## Grep

`grep` gives every trail of every project to the system `grep`, as lines. The `tidy` skill searches the
trails with it.

```
view    <yyyy-mm-dd> <trail id8 | -> [<project name>] <kind> <id8> <fields> <text>
          trail | step   <open | working | done | done-dropped | dropped | expired>[ revised-by <id>,…][ at:<place>]…
          pattern        <live | replaced-by <id> | dropped> {<me | project | trail:<id8>>}
                           [ trigger][ binds][ recalled:<yyyy-mm-dd>]
          proof | changed | dropped                              the step's or pattern's id, no fields
lines   <id8> [<project name>] <its title> · <yyyy-mm-dd>        each trail of the lines
          trail <id8> <fields>                                    the trail's own line
          <kind> <id8> <fields> <text>                            each other line of the trail
        - patterns of no trail
          [<project name>] pattern <id8> <fields> <text>             each line with no trail
end     project [<project name>]
```

`<id8>` is the first 8 characters of an id. A text holds each newline as a space. A pattern's fields end with
` trigger` when it has a trigger tag (CIP-05), ` binds` when it is a rule (CIP-05, "Rules"), and
` recalled:<day>` with the day `search` last gave it.

**Rules**

- `grep` MUST give each word after it to the system `grep` as it is, with the view on its stdin.
- The view MUST hold every trail of every project, newest first, and each live or replaced pattern of a
  scope other than `trail:<id>` by when it was made. A trail MUST print as its line, then its patterns,
  newest first, then its steps, each level newest first, each step with its proof, `changed` and drop
  lines, then its own steps.
- `grep` MUST print each matched line under its trail's header, the trails with most lines first, and each
  line with no trail under `- patterns of no trail`. A matched line that is no line of the view MUST NOT
  print. When no matched line is a line of the view, as with `-c` or `-l`, it MUST print what `grep`
  prints, and nothing else.
- `grep` MUST print every matched line that is a line of the view, however long the output.
- `grep` MUST end with the `project` line of the working folder.
- When `grep` exits 1, `grep` MUST print `No line matches.` and exit 0. When it exits 2, it MUST exit 2.

**Examples**

```
celis grep -i -E "price|cents"
celis grep " live {me} "
celis grep -A3 9b0e11aa
```

## Search

`search` gives what fits an ask: the live patterns in scope in the working folder's project (CIP-05), the
rules first, then the other patterns, and the done asks of every project like it (CIP-04).

```
rules
<line>        <id8> <scope> <first line>, each rule that fits (CIP-05, "Rules")
                <each next line of its text>
                next    <id8> <first line>                               a smaller pattern (CIP-05, "Graph")
                good    <id8> <title> · <the first line of its proof>    a step example (CIP-05, "Examples")
                        <the path of each blob of its files>
                good    <path>                                           a file example
                        <the path of its blob>
                bad     <url>                                            a URL example
patterns
<line>        each other pattern that fits, as a rule prints
past
<id8> <title> · <yyyy-mm-dd> [<project name>]     each done ask like it
  step <id8> <state> <title>                      each step right under it
    changed <text>                                when the step went otherwise than planned
none          nothing fits this ask
```

With `--kind trail` or `--kind step`, `search` gives the records of that kind most alike in meaning to
the text, by the model that `setup` chose.

```
line     <score> <grep view line>
json     {"id", "kind", "score", "project", "content"}
no model celis search has no model: search with celis grep, or run celis setup.
```

`<score>` is the cosine of the text and the record, with two decimals.

**Rules**

- Each `<file>` MUST count as that file. A `<file>` that is a folder MUST count as each file that
  `git ls-files --cached --others --exclude-standard -- <folder>` gives in the project.
- A pattern fits the ask when it matches a file, when its first line before its first `:` shares a word
  with the ask, or when the model finds it among the 5 most alike in meaning, at a cosine of 0.3 or more.
- A done ask is like the ask when the model finds it among the 5 most alike in meaning, at a cosine of
  0.3 or more, or when its title and its trail's title share 2 words with the ask. `search` MUST print at most 2.
- `search` MUST print the parts in the order rules, patterns, past, and leave out a part with no line.
  Within a part, a pattern with a `replaces` tag comes first, then the rest; within each, first
  those that match a named file (a `path` with no `*` or `?`), then a glob, then
  those that fit by words or meaning; newest first within each.
- With `--kind rule` or `--kind pattern`, `search` MUST print only that part.
- With `--kind trail` or `--kind step`, `search` MUST print at most `--limit` records, 20 when not
  given, highest score first; with no model, it MUST print the `no model` line and exit 0.
- Under each pattern, `search` MUST print each of its smaller patterns in scope, newest first, at most 5.
- `--limit` MUST cap each part.
- `search` MUST refuse empty text, a `--kind` that is not `rule`, `pattern`, `trail` or `step`, a
  `--limit` that is not a positive number, and `--json` without `--kind trail` or `--kind step`, as a
  usage error.

**Examples**

```
celis search -- "cut the release"
celis search shelf/prices.py -- "round the prices"
celis search --kind trail --limit 5 -- "make retrieval faster"
```

## Setup

`setup` chooses the model that `search` uses, downloads it, and builds its index.

```
model    embeddinggemma | e5-small | none
list     <model> <download size> <speed> <license>[ recommended] <note>
```

**Rules**

- On a terminal with no `--model`, `setup` MUST ask for the model from the list. Else with no `--model`,
  it MUST print the list.
- With a model, `setup` MUST download it from Hugging Face at a pinned revision, check its sha256, and
  index every record that `search` gives. When the download or the check fails, it MUST keep the model
  it had.
- `none` MUST turn `search` off and remove the downloaded models.
- `setup` MUST refuse a model that is not in the list as a usage error.

**Examples**

```
celis setup
celis setup --model embeddinggemma
celis setup --model none
```
