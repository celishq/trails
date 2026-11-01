---
max_turns: 90
timeout_seconds: 1500
allowed_tools: [Read, Write, Edit, Agent, Bash, Skill]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
/celis:tidy — tidy this project only, and say yes to all in each part. Do not ask me anything.
