---
max_turns: 60
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash, Skill]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
/celis:migrate — migrate this project and the `me` rules, with no dry run. When you show me the list, I say yes to all of it. Do not ask me anything.
