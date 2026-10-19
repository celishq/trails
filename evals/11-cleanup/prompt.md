---
max_turns: 60
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash, Skill]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
/celis:cleanup — clean up this project only, and say yes to all in each part. Do not ask me anything.
