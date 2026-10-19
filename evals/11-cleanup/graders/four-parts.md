---
type: llm
focus: trace
---
The trace starts with the celis card that the session start shows; ignore it. Look only at the `celis` commands the agent ran. Check each claim; pass only if all hold:
1. The rule at `old/prices.py` is dropped with `celis step drop <its id> -- "<reason>"`, because the project has no such file.
2. Of the two rules that keep money in integer cents, one is dropped with a reason that names the other or a new rule (for example "merged into <id>"), or both are replaced by one new rule. One cents rule stays live.
3. Of the two rules on Python indents (tabs and four spaces), one is dropped, or a new rule replaces one with `--replaces` and the other is dropped. Only one indent rule stays live.
4. A named trail is written for the four changelog trails: a `celis step add --project skill:<name> --new` with `--uses` on each of the four trails, steps under its ask, and `celis name <trail id> <name>`.
5. The parts run in this order: stale, merge, conflicts, templates.
