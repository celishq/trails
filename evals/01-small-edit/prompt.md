---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Create `greet.py` with a function `greet(name)` that returns `Hello, <name>!`, and a unittest test for it in `test_greet.py`. Run the test with `python3 -m unittest`.
