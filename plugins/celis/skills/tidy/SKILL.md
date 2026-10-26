---
name: tidy
description: Tidies the celis trails and patterns. Drops stale trails and patterns, merges and settles patterns, promotes strong patterns into the project's own files, narrows or widens a When from its recalls and uses, and draws new patterns from past trails. Use when the person runs /celis:tidy, when celis asks to offer it, or when the person asks to tidy, clean up or prune their trails or patterns.
---

You tidy the trails in seven parts, in this order: stale, merge, conflicts, promote, the When, replaced
again, new patterns. celis keeps guides; a must goes into the project's own files. An older celis wrote
some patterns as rules; their grep fields say ` old-rule`.

1. Ask the person what to clean: this project, this project and the `me` patterns, or all projects.
2. Add one ask titled `Tidy the celis trails`, so that celis knows when the last tidy was, and one step
   under it for each part. Take the parts one at a time.
3. Read the evidence once:
   - `celis patterns` gives the index: each live pattern in scope, as its id and its When.
   - `celis grep -E " (trail|pattern) [0-9a-f]{8} "` gives each trail and pattern line. A pattern's
     fields give its state (`live`, `replaced-by <id>` or `dropped`), ` old-rule` and ` recalled:<day>`.
   - `celis stats --json` gives how often each pattern was used, and how often each pattern in scope of
     the working folder was recalled.
4. For each change, show the person the records it touches, with their ids, and ask: yes, no, or yes to
   all in this part. Write nothing, in celis or in a file, before a yes.
5. To write into another project, add `--project <its id>` to `pattern` and `step add`; the id is the
   `project` that `celis tree <a trail of it> --json` gives.

## Stale

Drop with `celis step drop <id> -- "<why>"`:

- a trail with stale steps, or that is open and untouched for weeks;
- a pattern that names a place, code or a command the project no longer has;
- a pattern of revised work (`revised-by`) that the new work contradicts;
- a one-off note about one event;
- a pattern older than 60 days that no ask recalled in the last 60 days, when asks have named their
  recalls for 60 days or more.

## Merge

Find live patterns of one project and scope that say the same thing, or whose good examples make one
point. Keep the one that says it best, or write one in the four lines with `pattern --replaces <the
newest>`; drop each other with `-- "merged into <id8>"`. A merged pattern gives `--good` and `--bad`
again for the examples of each pattern it does not replace. When
two or more projects hold the same `project` pattern, offer one `me` pattern in its place.

## Conflicts

Find live patterns that hold together for the same case and say to do different things. Ask the
person which one resolves the forces better, or for a new one that resolves both, and drop the other
with `-- "conflicts with <id8>"`.

## Promote

Offer to promote each ` old-rule` pattern first, then each pattern used in most of its recalls and never
replaced. A promoted pattern becomes a must in the place's own files:

- a check of a tool, when one can hold it, such as a test, the linter, CI or a Figma project's styles;
- else a line in the project's agent file, `AGENTS.md` or `CLAUDE.md` (`AGENTS.md` when it has none), for
  a `project` pattern, or in the person's global `CLAUDE.md` for a `me` pattern.

After a yes, write the file and run the check. Then drop the pattern with `-- "promoted to <file>"`, so
that it leaves recall.

## The When

- Narrow the When of a pattern recalled often but seldom used: it comes into asks it does not help.
- Widen the When of a pattern seldom recalled but used each time: asks that need it do not find it.

Write the new When with `pattern --replaces <id>`, and keep its Forces, Do, Because and examples.

## Replaced again

Find each pattern replaced again and again: its chain of `replaced-by` holds three patterns or more.
Show the person the chain, and ask what they want the pattern to say.

## New patterns

Find the choices and corrections in past trails that came back, in `changed` lines, notes, drops and
revised steps of two trails or more, and that no live pattern holds. Offer each as a pattern in the
four lines, with the narrowest `--path`.

## End

Close each part's step with how many trails and patterns it dropped, merged, promoted and wrote. Close
the ask with the size of the index, as `<n> patterns in the index`. Tell the person what each part
changed, and how many patterns the index holds now and at the last tidy, so they see when it grows.
