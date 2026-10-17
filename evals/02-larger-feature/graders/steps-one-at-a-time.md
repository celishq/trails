---
type: llm
focus: trace
---
The trace starts with the celis card that the session start shows; ignore it. Look only at the `celis` commands the agent ran. Check each claim; pass only if all hold:
1. The ask is added with `celis step add`, and 2 to 5 steps are added `--under` it (directly or nested). Doing the whole ask as one step with nothing under it fails this claim.
2. Steps added together at the start have a title only (an `expect:` line is fine), with no `--at` and no `--check`, except the one step the agent takes first.
3. A step gets `--at` and `--check` (when added, or later with `celis step edit`) only when the agent takes it as one action, right before `celis step start` on it.
4. No step gets `--at` and `--check` before the previous step the agent took is closed with `celis step done`.
5. Each `step done` proof names something a person can check (a command and its result, a file and what it holds). "done" or "works" fails.
6. The ask is closed with `step done` at the end.
