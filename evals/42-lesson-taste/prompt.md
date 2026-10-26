---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
The ask in `ask.id` is done. Keep what it taught: run `celis lesson` on that id and give the brief to the celis:lesson agent.
