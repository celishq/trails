---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash, Skill]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
/celis:train the error messages of a CLI. Do not wait for me: when you show the 4 variants, take these marks. The 1st and the 4th variant are good, because they say what to do next. The 2nd is bad, because it blames the user. The 3rd is neither.
