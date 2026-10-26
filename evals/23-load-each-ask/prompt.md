---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add `refund(cents, percent)` to `cart.py`: it returns the part of a payment to give back. Add a unittest test in `test_cart.py` and run it with `python3 -m unittest`.
