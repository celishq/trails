---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add `slug.py` with `slugify(text)`: lower case, words joined by `-`, and only letters, digits and `-` kept. Add a unittest test in `test_slug.py` and run it with `python3 -m unittest`.
