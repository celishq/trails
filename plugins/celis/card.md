celis keeps each ask as a trail: its steps, their proofs, and the rules that later work must keep.

```
celis rules [<file>]... [--all] [--session <id>] [-- "<command>"]
celis grep <grep option>... "<pattern>"
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] [--path <glob>]... [--content <regex>] [--command <regex>] [--tool <regex>] [--situation] [--promoted] [--project <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] [--session <id>] [--project <id>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--pitfall "<rule>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis name <trail id> <name>
celis tree [<id>] [--graph] [--full] [--all] [--demo]
```

1. Ask before you act. At a new ask, when celis asks for the `celis:past` agent, run it as celis says,
   even for a simple ask. With no such agent, search for past asks like this one yourself:
   `celis grep -i -E " (step|rule) [0-9a-f]{8} .*(<word>|<word>)"`, with the words of the ask. Before you
   read or change a file, and before you run a command, run `celis rules` with those files and that
   command, and keep each rule it gives. Run it again on a file after you write it, because some rules
   come only on the new text. When celis already gave the rules of a call, do not ask again. A rule
   that no file or command shows comes only from a search:
   `celis grep -i -E " rule [0-9a-f]{8} live .* situation .*(<word>|<word>)"`. When the ask conflicts
   with a rule, the ask wins: replace the rule.
2. An ask that changes a file, however small, or that works toward a result someone can check, gets a
   step: review feedback to fix too. A question, an explanation, or a review or investigation you only
   report on gets none: answer it. Add the ask with `step add` where the prompt hook says. With no hint,
   add it under the session's trail, else with `--new`. An ask that corrects done work takes
   `--replaces <its id>`. Give the ask the project's own check, and `--uses <id>` for each rule, past
   step or named trail it follows. `--uses` takes a trail's name.
3. A step's title names what the person wants done: a verb and what it acts on, in words that hold in
   another project. On the line after the title, write `expect: <what a person can check when it is
   done>`. A simple ask is one step. Only a hard ask gets 2 to 5 steps under it: split first the step
   with the most doubt, one level at a time, until a step is one action. Only the step you take next
   gets `--at` and a `--check` that proves it. Add the next step after you close this one.
4. For each step: `start`, and keep the rules it prints. Do the work, then `done` with a proof a person
   can check, such as a command and its result: `--as-planned` when the result matches `expect`, else
   `--changed`. A wrong step, or a fix to done work, takes `--replaces <id>`. When a step grows past one
   action, or you try the same thing twice, stop and split it.
5. When something surprises you, or the person corrects you, write the rule that would have saved you
   with `decide`, before you go on. Give it the narrowest trigger that shows its moment: `--path` for
   the files it acts on, with `--content` when only some text in them matters, `--command` for shell
   commands, `--tool` for other tool calls, and `--situation` only when no call shows it ahead. A
   correction replaces the wrong rule with `--replaces <id>`: never keep two rules for one point.
6. Do what celis prints after each command. It names the next move.
7. Close the ask, never the trail. Tell the person about the ask by the steps right under it, and each
   rule you wrote.

A rule reads "When <what you see>: <what to do>, because <the reason>." The "When" names the work the rule
acts on, such as "When a Raycast extension changes", never the review, the person or the project it came
from, so it matches the next task like it. The trigger says when the rule comes; the text stays general.
Its scope is `trail:<id>` when it holds for this trail, `project` (the default) for the repo, and `me`
when it holds for that kind of work in any project: a rule about a framework, a platform or a review's
convention. When the person asks you to remember something, or states how the work must be done, write
each point as one rule before you do the work.

For more: `celis rules <file> --all` gives every rule of a file, `celis grep` searches every trail and
rule, and `celis tree` shows the work. Read the `steps.md` the session start names for moved files, past
work, named trails, subagents, links, and when to stop.
