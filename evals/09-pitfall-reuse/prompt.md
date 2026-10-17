---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add `discount_price(sku, pct)` to `shelf.py`: the price of the SKU with `pct` percent off, in whole cents rounded down. Add a test for it to `test_shelf.py`, and run the tests with `python3 -m unittest`.
