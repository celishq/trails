---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Write `parse_port(text)` in `port.py`: it returns the port as an int and raises a ValueError for text that is not a port from 1 to 65535. From now on, in this project, each error message says what to do next, because our support team gets the tickets when it does not. Then add a unittest test in `test_port.py` and run it with `python3 -m unittest`.
