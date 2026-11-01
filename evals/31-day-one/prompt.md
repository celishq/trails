---
max_turns: 60
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash, Skill]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
/celis:learn STYLE.md. I keep all the drafts; do not ask me.
