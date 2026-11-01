---
max_turns: 40
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: sonnet
runs: 5
plugins: [../../plugins/celis]
---
In `cart.py`, make `total(prices, coupon=None)` take 10% off the sum when `coupon` is `"SAVE10"`, and add a unittest test in `test_cart.py`. Run it with `python3 -m unittest`.
