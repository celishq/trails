---
type: llm
focus: trace
---
The trace starts with the celis card that the session start shows; ignore it. Look only at the `celis` commands the agent ran. Check each claim; pass only if all hold:
1. The ask is added with `celis step add`. Steps under it are optional.
2. Each `celis step done` proof names something a person can check, such as a command and its result, or a file and what it holds. "done" or "works" fails.
3. The ask is closed with `celis step done` at the end.
