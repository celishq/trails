---
cip: 5
title: Reports
status: draft
depends: [CIP-01, CIP-02, CIP-03, CIP-04]
---

# CIP-05 — Reports

## The report

A report gives the work of trails as Markdown, so a person can add it to a pull request. The agent finds
the trails the work holds. The report only reads the store.

```
celis report <id>...   the trails the ids name, as Markdown
```

**Rules**

- `report` MUST take each id as CIP-02 "Output and errors" states, and refuse an id no step has, with
  `no-step`. It MUST refuse no id as a usage error.
- `report` MUST print the trail of the step each id names, once, in the order of the ids. A blank line
  MUST come between two trails.
- `report` MUST print a trail's title as a heading. Then come its id, project, state, the count of steps
  under it, and how long its tree ran. Then come the lines of its text after its title, and its proof
  when done.
- `report` MUST print the trail's rules, and then the steps under it, oldest first, each followed by its
  own, as a Markdown list. A step MUST start with its mark, as `tree` marks it (CIP-04, "The commands").
  A section with no line MUST be left out.
- When a trail's first ask has the trail's text, `report` MUST NOT print the ask. It MUST print the ask's
  steps first, then the trail's other asks.
- Under a step come the lines of its text after its title, its places, and its check. The check MUST read
  `check passed` on a done step, because `step done` runs it (CIP-03, "Closing"). Then come its proof and
  what changed when done, or its reason when dropped.
- `report` MUST NOT run a check, and MUST NOT write a record.

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

## The log

The log keeps the runs of `celis` that failed, so a person can show the celis team what went wrong. It
leaves out the text a person gives, because that text holds their work.

**Rules**

- A run of `celis` that fails MUST add one line to `$CELIS_HOME/log`: the local time, the exit code,
  `celis` with the words of the command, and the error message.
- The words MUST leave out the flags, their values and the text after `--`.
- A run that succeeds MUST NOT write the log. A run MUST NOT fail because it cannot write the log.

```
log  <yyyy-mm-dd> <hh:mm> <exit code> celis <words> · <message>
```

**Examples**

```
2026-10-13 14:02 1 celis step done ffffffff · no step has the id ffffffff; find it with `celis tree --all`
```

## The issue

An issue report gives the celis team what they need to find a fault. The agent writes what went wrong,
shows the report to the person, and sends it only the way the person chooses. The report only reads.

```
celis issue [<id>]... [-- <text>]   what went wrong, with the facts of this machine, as Markdown
```

**Rules**

- `issue` MUST print the first line of the text after `--` as a heading, and the rest of the text under
  it. With no text, the heading MUST be `celis issue`.
- `issue` MUST print the facts: the version as `celis --version` prints it, the OS and CPU, the names of
  the session variables that are set (CIP-04, "Sessions"), the working folder's project, `$CELIS_HOME`
  with the home folder as `~`, whether the phrase exists, and the count of record files in the store,
  of those that read, and of steps.
- `issue` MUST print the last 50 lines of the log (CIP-05, "The log") in a code block, and leave the
  section out when the log is empty.
- `issue` MUST print the trails the ids name, as `report` prints them. It MUST refuse an id no step has,
  with `no-step`.
- `issue` MUST NOT print the phrase, and MUST NOT write a record.

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
