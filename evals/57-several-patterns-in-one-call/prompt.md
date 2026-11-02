---
max_turns: 40
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: sonnet
runs: 5
plugins: [../../plugins/celis]
---
Make `parse_quantity(text)` in `order.py` refuse a quantity below 1 with a ValueError. Add a unittest test in `test_order.py` and run it with `python3 -m unittest`.
