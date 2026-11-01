---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Make `parse_age(text)` in `age.py` refuse a negative age with a ValueError. Add a unittest test in `test_age.py` and run it with `python3 -m unittest`.
