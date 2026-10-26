---
name: tidy
description: Tidies the celis trails and patterns. Drops stale trails and patterns, merges and settles patterns, promotes strong patterns into the project's own files, narrows a pattern that misled, drops or widens one unused, and draws new patterns from past trails. Use when the person runs /celis:tidy, when celis asks to offer it, or when the person asks to tidy, clean up or prune their trails or patterns.
---

The person wants a small index of patterns that guide, and no musts in celis: a must goes into the
project's own files. Ask what to tidy: this project, this project and the `me` patterns, or all
projects. Do the tidy as one ask titled `Tidy the celis trails`, so that celis knows when the last tidy
was. Start from `celis patterns --tidy`: each pattern that needs tidy, and why. `celis patterns` gives
the index, and `celis grep` gives each pattern with its uses, misled, last use and replaces; an older
pattern that its grep line marks ` old-rule` was written as a rule.

A good tidy:

- drops stale trails and patterns: a trail left open and untouched for weeks, a pattern that names a
  place, code or command the project no longer has, or that later work contradicts, and a one-off note;
- narrows the When of a pattern that misled the work, or replaces it, so that it says what the fixes
  showed;
- drops a pattern that is `unused`, or widens its When when the work it names still comes back;
- merges patterns that say the same thing into one that keeps all their examples, and settles patterns
  that say to do different things in the same case, with the drop reason `merged into <id8>` or
  `conflicts with <id8>`;
- promotes each ` old-rule` pattern, then each pattern that steps used often, that never misled and
  that was never replaced: a check such as a test, a lint or CI when a tool can hold it, else a line in
  the project's `AGENTS.md` or `CLAUDE.md` for a project pattern, or in the person's global `CLAUDE.md`
  for a `me` pattern. Then it drops the pattern with the reason `promoted to <file>`;
- asks the person what a pattern replaced again and again must say;
- offers new patterns for the choices and corrections that came back in two past trails or more and that
  no learn kept.

Show the person each change with the ids it touches, and write nothing, in celis or in a file, before
they agree; they can agree to all of one kind at once. Close the ask with `<n> patterns in the index`,
and tell the person what changed and the size of the index now and at the last tidy.
