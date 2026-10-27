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
celis pattern [--scope <scope>] [--replaces <id>] [--path <glob>]... [--within <id>]... [--good <example>]... [--bad <example>]... [--project <id>] -- <text>
                                                       a pattern, in its scope
celis patterns [<id>] [<file>]... [--markdown] [--tidy]
                                                       the index of the patterns in scope, one whole, or what needs tidy
celis tree [<id>]... [--graph] [--full] [--all] [--demo]
                                                       a trail and its steps; again on each change
celis grep <grep option>... <pattern>                 every trail, through the system grep
celis stats [<id>] [--demo]                           the use of the patterns, or a step's grade
celis extract <id>                                    what an ask or a step taught, to extract as patterns
celis issue [<id>]... [-- <text>]                     what went wrong, as Markdown for the celis team
celis link [<folder>] [--off]                         keep this project's trails in a folder, or stop
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--misled <id>]... [--at <place>]... [--check <command>] [--session <id>] [--project <id>] -- <text>
                                                       leave a step
celis step start <id>                                 take up a step, and see the patterns and notes its plan links
celis step done <id> [--changed <text>] [--as-planned] [--by person] [--misled <id>]... [--file <path>]... -- <proof>
                                                       close a step with its proof
celis step drop <id> -- <reason>                      end a step, a done trail or a pattern no longer wanted
celis step edit <id> [--at <place>]... [--check <command>] [-- <text>]
                                                       give a step a new title, places or check
celis step note <id> -- <text>                        leave a note on a step, for the work of its trail
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
pattern  json  {"id", "created_at", "project", "scope", "replaces", "content",
                "paths"}
```

**Rules**

- Off macOS, `$CELIS_HOME` MUST default to `~/.local/share/celis`.
- A command that "Commands" does not name, and a flag no command takes, is a usage error.
- `search`, which older plugins give, MUST refuse as a usage error with a message that names
  `celis patterns`.
- `pattern` MUST write a pattern of the working folder's project (CIP-05), with `--scope` and
  `--replaces` as its tags, and no `scope` tag for `project`; each `--path`, from the project's top
  folder, as a `path` tag, each `--within` as a `within` tag (CIP-05, "Graph"),
  refusing an id that no live pattern of the store has with `no-pattern`, and each `--good` and `--bad`
  as an example (CIP-05, "Examples"): a value with `://` as a URL, a file of the working folder as its
  blob, else the whole id of a step, refusing a step the store lacks with `no-step`.
- `pattern` MUST accept `--binds` and `--cell <topic>`, which older plugins give, and write nothing for
  them.
- `--project` on `pattern` and `step add` MUST write the records of the project it names in place of the
  working folder's: a project id that a record of the store has, with that record's project name. It MUST refuse any other id as a usage error.
- `pattern` MUST refuse empty text, and a scope that is not `me` or `project`, as a usage error, with a
  message that names `step note` for `trail:<id>`; and a `--replaces` that no live pattern of the
  project has, with `no-pattern`.

**Examples**

```
celis pattern --path shelf/prices.py -- "When you change prices
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
- A command MUST NOT ask a question, but `auth login`.
- A command MUST exit 0 when done, 2 on a usage error, and 1 on any other error.
- A command MUST take each deprecated command and flag of the README, and, without `--json`, MUST print one
  line on stderr that names it as deprecated.
- An error MUST be named `usage`, `no-pattern`, `no-step`, `closed`, `open-steps`, `check-failed`,
  `secret` (CIP-01, "The account"), else `error`. With `--json`, it MUST be one object on stderr,
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
step  json  {"id", "created_at", "project", "links": [{"to", "kind": "under | replaces | uses | misled"}], "places": [],
            "check", "content"}
note  json  {"id", "created_at", "step", "content"}
```

**Rules**

- `add` MUST write the whole id of `--under`, `--replaces`, each `--uses` and each `--misled`, each
  `--at` (CIP-01, "Places") and the `--check` as tags.
- `add` and `done` MUST write each `--misled` as CIP-06, "Misled", states, and refuse an id that no
  pattern of the store has, with `no-pattern`. `add` MUST refuse `--misled` without a `--replaces` of a
  done step as a usage error.
- `add` MUST write no step when an open step of the same project and `under` has the same text, and
  print that step's id. With `--uses`, or a `--replaces` of a done step, it MUST refuse it as a usage
  error.
- `add` MUST accept `--new` and `--recalled <id>`, which older plugins give, and write nothing for them.
- `add` MUST refuse empty text as a usage error; a closed `--under`, and a `--replaces` of a dropped step,
  with `closed`; a `--uses` that no live pattern and no step has, with `no-step`; and a `--uses` that
  names a step, or that starts both a pattern's and a step's id, as a usage error.
- `start` and `done` MUST refuse a closed step, with `closed`. `start` of a step that a `start` names
  MUST write nothing and print the id of its first `start`.
- `drop` MUST take the id of a step or a pattern. It MUST refuse a dropped step, a done step that is not a
  trail, and a pattern that is not live, with `closed`; and an id that starts both a pattern's and a
  step's id as a usage error.
- `done` MUST take one of `--changed <text>` and `--as-planned`, and a proof, else refuse as a usage
  error. It MUST refuse a step with an open step under it, with `open-steps`.
- The changed files of a `done` are each file that `git diff --name-only HEAD` or
  `git ls-files --others --exclude-standard` gives in the working folder, and each `--file`.
- `done` MUST run the step's `check` in the project's top folder, and refuse with `check-failed` when it
  fails or runs past `CELIS_CHECK_SECONDS`, 600 when not set.
- `done --changed` MUST write its text as `changed`, and `done --by` its value as `by`; `--by judge`, which
  older plugins give, MUST write no `by`, and any other `--by` is a usage error. Each `done --file` MUST be kept as a blob and named by an
  `imeta` tag (CIP-08), and a file that is not there is a usage error.
- `done` of a step that is not an ask MUST print each live pattern in scope that a changed file matches
  (CIP-05, "Paths"), whole with its examples, once, as a list, by id.
- `drop` MUST refuse an empty reason, and `edit` one with no text, `--at` or `--check`, as usage errors.
- `note` MUST write its text as a note on the step (CIP-03, "Notes"), and refuse empty text as a usage
  error.
- `start` MUST print the notes of the step's trail (CIP-04, "Notes").

The output of `done` of an ask begins with the learn brief, before its total line. The
output of `done` of another step that fixes done work (CIP-06, "Revisions") ends with it, before the next
move:

```
celis: before you answer, learn from <id8>. …           what the agent keeps, and how
  changed <id8> <text>                                    each `changed` of the ask and of its steps
  note    <text>                                          each note of its trail (CIP-04, "Notes")
  asked   <text>                                          each prompt on its trail (CIP-04, "Sessions") since
                                                          the newest one before the ask, that one too
  pattern <id8> <Name>                                    each live pattern a step of its ask's tree uses
                                                          (CIP-06, "Uses"), once each
```

**Examples**

```
celis step add --under 3fa2c1d0 --at shelf/prices.py:40 -- "Write a test: a senior pays 80% of the price."
celis step start 4c1d22e0
celis step done 4c1d22e0 --as-planned -- "python3 -m unittest: test_senior failed before, passes now."
celis step drop 3fa2c1d0 -- "The person chose the shared price table instead."
celis step note 3fa2c1d0 -- "The person wants the old table kept until the release."
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

`patterns` gives the index of the live patterns in scope in the working folder's project (CIP-05). An
agent reads it, and reads the whole of a pattern with `patterns <id8>`.

```
<id8> <scope> <Name>     each pattern; its Name is the text after `Name:` on the first line of its
                         text, and with none, its When: the first line, and of a text of one line,
                         the part before its first `:`; the When of a named text is its next line
  path   <glob>          each `path` of it (CIP-05, "Paths")
  match  <file>          each `<file>` that one of its paths matches (CIP-05, "Paths")
  within <id8>           each larger pattern it completes (CIP-05, "Graph")
no pattern yet           when there is none
```

```
celis patterns --tidy
<id8> <reason>[,<reason>]... <Name>     each live pattern in scope that needs tidy
<n> patterns in the index               last, with `1 pattern` for one; the only line when none needs tidy
```

A pattern's reasons, in this order:

- `misled`: it misled 2 or more steps (CIP-06, "Misled").
- `near:<id8>`: the When of another live pattern in scope shares 2 or more words with its When, as the
  `like` lines of `pattern` count them; the id is of the one that shares the most.
- `unused`: it was made 60 or more days ago, and no step used it in the last 60 days (CIP-06, "Uses"),
  while a step of the project used a pattern.
- `churn`: it ends a chain of 3 or more `replaces` (CIP-05).
- `nameless`: its text has no Name.
- `form`: its When starts with `When:`, or holds `Forces:`, `Do:` or `Because:`.

**Rules**

- `patterns` MUST print each live pattern in scope once, oldest first, so that a new pattern comes last.
- `patterns` MUST print a `match` line under a pattern for each `<file>` that one of its paths matches.
  A `<file>` is a path from the working folder, and a folder stands for each file in it.
- With `--markdown`, `patterns` MUST print each live pattern in scope whole, with its examples, as one
  Markdown page.
- With an `<id>` of a live pattern in scope, a word of 4 to 64 hex characters that names no file,
  `patterns` MUST print that pattern whole, with its examples, as `--markdown` does, and refuse an `<id>`
  that names no live pattern in scope with `no-pattern`. It MUST write a recall of the pattern on the
  session's newest open step (CIP-06, "Recalls"), and nothing when the session has none or the step
  has a recall of it.
- With `--tidy`, `patterns` MUST print a line for each live pattern in scope that has a reason, oldest
  first, then the line of the index size. It MUST refuse `--tidy` with a `<file>` or `--markdown` as a
  usage error.

**Examples**

```
celis patterns
celis patterns shelf/prices.py shelf/
celis patterns 9b0e11aa
celis patterns --markdown > patterns.md
celis patterns --tidy
```

## Stats

`stats` with no `<id>`, off a terminal or with `--json`, gives the use of the live patterns in scope in
the working folder's project (CIP-05).

```
<id8> uses:<n> misled:<n> replaces:<n> used:<yyyy-mm-dd | -> examples:<n> tidy:<reason>[,<reason>]... | - <Name>
                                         each live pattern in scope
total <n> patterns in the index · last 7 days: <n> written, <n> replaced, <n> gaps
                                         last, with `1 pattern` for one
```

```
lifecycle <day | week | month> <yyyy-mm-dd> written:<n> replaced:<n> dropped:<n> promoted:<n> recalled:<n> used:<n> misled:<n>
                                         the last 7 days, 4 weeks and 6 months, oldest first
value last 28 days: asks:<n> covered:<n> corrections:<n> failed:<n> missed:<n> first-pass:<n%|->/<n%|-> recalled:<n> used:<n> held:<n> reach:<n> agents:<name>[,<name>]... | - promoted:<n>
```

```
pattern    json  {"id", "uses", "misled", "replaces", "used", "examples", "tidy": [], "name", "when"}
total      json  {"index", "written", "replaced", "gaps"}
lifecycle  json  {"span", "from", "written", "replaced", "dropped", "promoted", "recalled", "used", "misled"}
value      json  {"asks", "covered", "corrections", "failed", "missed", "with", "without", "recalled",
                  "used", "held", "reach", "agents": [], "promoted"}
```

A pattern's fields: `uses`, how many steps name it in `uses` (CIP-06, "Uses"); `misled`, how many steps
it misled (CIP-06, "Misled"); `replaces`, how many `replaces` its chain holds (CIP-05); `used`, the
day of the newest step that uses it, `-` or `null` when none; `examples`, how many examples it has
(CIP-05, "Examples"); `tidy`, its reasons as `patterns --tidy` gives them ("Patterns"), `-` when none;
`name`, its Name ("Patterns"), `null` when none.

The total's fields: `index`, how many patterns are live in scope; `written`, the patterns in scope made
in the last 7 days; `replaced`, those of them with a `replaces`; `gaps`, the steps of the project made in
the last 7 days that fix done work (CIP-06, "Revisions") in a trail whose steps use no pattern.

The lifecycle's fields, for the live and the ended patterns in scope, in a span: `span` is `day`,
`week` or `month`, and `from` its first day: a day, 7 days that end on today or on a multiple of 7 days
before, or a month of the calendar. `written`, the patterns made in it with no `replaces`; `replaced`,
those made in it with a `replaces`; `dropped`, those a drop named in it; `promoted`, those of them whose
drop reason starts with `promoted to`; `recalled`, the recalls of them written in it (CIP-06,
"Recalls"); `used`, the uses of them by steps made in it (CIP-06, "Uses"); `misled`, the steps they
misled (CIP-06, "Misled"), on the day of the `done` or the step that tells it.

The value's fields, for the asks of the project done in the last 28 days and the patterns in scope:
`asks`, those asks; `covered`, those of them whose tree uses a pattern; `corrections`, the steps of
their trees that revise done work; `failed`, those corrections whose revised step's trail uses a
pattern; `missed`, the others whose own trail has a pattern made since its ask began that replaces a
pattern made before the revised step; `with` and `without`, the percent of done steps of their trees
that no step revised, among those that use a pattern and those that use none, `null` and `-` with no
such step; `recalled`, `used` and `held`, the recalls in their trees, the uses, and the uses by steps
that no step revised and no `misled` names; `reach`, the uses of a `me` pattern by a step of another
project, in the store; `agents`, each `agent` of a step that uses a pattern, `-` with none; `promoted`,
the patterns promoted in that time.

**Rules**

- With no `<id>`, off a terminal or with `--json`, `stats` MUST print a line for each live pattern in
  scope, oldest first, then the total, the lifecycle lines and the value line. With no record in the store, it MUST print `Nothing was reused
  yet.`, and nothing with `--json`.

**Examples**

```
celis stats
celis stats --json
```

## Extract

```json
{"ask": "<id>", "changed": [{"step": "<id>", "text": "<changed>"}], "notes": ["<note>"],
 "asked": ["<prompt>"], "patterns": [{"id": "<id>", "when": "<When>"}]}
```

- `changed`: each `changed` of the ask's steps and of the ask.
- `notes`: each note of its trail.
- `asked`: each prompt on its trail since the ask began.
- `patterns`: each live pattern that a step of the ask's tree uses.

**Rules**

- `extract` MUST print the learn brief ("Step") of the ask or step `<id>`, open or closed, and write no
  record. With `--json`, it MUST print the brief as the object above.

**Examples**

```
celis extract 3fa2c1d0
celis extract 3fa2c1d0 --json
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

- `hook` MUST read the hook's JSON on stdin, and act on it as "Prompt" states. It MUST act in the
  folder the JSON names in `cwd`, else in the first of `workspace_roots`, else of `workspacePaths`.
- `hook start` MUST read the card from `card.md` and the plugin's version from `.claude-plugin/plugin.json`
  in `<plugin folder>`.
- After the card, `hook start` MUST print the line `The patterns in scope here:`, then what `patterns`
  prints with no `<file>` ("Patterns"), at a session start and after a compaction alike. It MUST print
  neither when no pattern is live in scope.
- When `patterns --tidy` lists `<n>` patterns, 1 or more, and no ask whose title starts with `Tidy the
  celis trails` began in the last 7 days, `hook start` asks the agent to offer `/celis:tidy` once in the
  session, after the person's first ask, and names `<n>`.
- `hook` MUST print in the shape of `<agent>`, and nothing when it has no text, but Cursor's ask.
- `hook` MUST exit 0 on any error but a usage error.
- `hook ask` MUST keep no prompt that holds a secret (CIP-01, "The account"), and MUST say so to the
  agent.

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
          trail | step   <open | done | dropped>[ revised-by <id>,…][ at:<place>]…
          pattern        <live | replaced-by <id> | dropped> {<me | project>}
                           [ old-rule][ uses:<n>][ misled:<n>][ used:<yyyy-mm-dd>]
          proof | changed | dropped | note                       the step's or pattern's id, no fields
          prompt                                                 the trail's id, no fields (CIP-04, "Sessions")
          good | bad                                             the pattern's id, no fields, and an example:
                                                                 <step id8> <title> · <the first line of its proof>,
                                                                 <path> · <its blob>, or <url> (CIP-05, "Examples")
lines   <id8> [<project name>] <its title> · <yyyy-mm-dd>        each trail of the lines
          trail <id8> <fields>                                    the trail's own line
          <kind> <id8> <fields> <text>                            each other line of the trail
        - patterns of no trail
          [<project name>] pattern <id8> <fields> <text>             each line with no trail
end     project [<project name>]
```

`<id8>` is the first 8 characters of an id. A text holds each newline as a space. After its state and
scope, a pattern's fields give ` old-rule` when it has the `binds` label that older writers wrote (CIP-05,
"Reading"), ` uses:<n>` with how many steps name it in `uses` (CIP-06, "Uses"), when not 0, ` misled:<n>` with how
many steps it misled (CIP-06, "Misled"), when not 0, and ` used:<day>` with the day of the newest step
whose `uses` names it.

**Rules**

- `grep` MUST give each word after it to the system `grep` as it is, with the view on its stdin.
- The view MUST hold every trail of every project, newest first, and each live or replaced pattern by
  when it was made. A trail MUST print as its line, then the notes on it, then its prompts, oldest first,
  then its steps, each level newest first, each step with its proof, `changed`, drop and note lines,
  then its own steps. A pattern MUST print as its line, then a line for each example, then its drop
  line.
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
