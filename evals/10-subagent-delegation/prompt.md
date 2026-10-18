---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Hand this to a subagent with the Agent tool, and do not write the code yourself: write `total(prices)` in `cart.py`. It takes a list of item prices and returns the total with 8% tax added. Add a unittest test in `test_cart.py` and run it with `python3 -m unittest`.
