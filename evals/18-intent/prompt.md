---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add `discount(price_cents, percent)` to `shop.py`: it returns the price in integer cents after the discount, rounded down, and refuses a percent outside 0 to 100. Add a unittest test in `test_shop.py` and run it with `python3 -m unittest`.
