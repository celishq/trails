---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Write `discount(price)` in `price.py` that takes 10% off, with a unittest test in `test_price.py`, and run it with `python3 -m unittest`. When that test passes: I changed my mind, make it 15% off for prices over 100 and keep 10% for the rest. Update the test too.
