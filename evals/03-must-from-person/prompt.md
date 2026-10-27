---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
From now on, every Python function in this project has type hints. Write `slugify(text)` in `slug.py`: it lowercases the text and replaces each run of non-alphanumeric characters with one `-`.
