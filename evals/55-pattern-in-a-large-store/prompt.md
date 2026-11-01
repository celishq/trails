---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Make `parse_quantity(text)` in `order.py` refuse a quantity below 1 with a ValueError. Add a unittest test in `test_order.py` and run it with `python3 -m unittest`.
