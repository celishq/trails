---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: sonnet
runs: 5
plugins: [../../plugins/celis]
---
Add a function farewell(name) to greet.py that returns "bye " + name and reports the call.
