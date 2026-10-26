---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add `discount(cents, percent)` to `shop/prices.py`: it returns the price in cents after the discount, rounded down. Add a unittest test in `test_prices.py` and run it with `python3 -m unittest`.
