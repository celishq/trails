---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Write `total(prices)` in `cart.py`. It takes a list of item prices and returns the total with 8% tax added. Add a unittest test in `test_cart.py` and run it with `python3 -m unittest`.
