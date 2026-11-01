---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
`show_price(1990)` in `price.py` gives `$19.9`, and `show_price(100)` gives `$1.0`. Fix it so it always shows two decimals, such as `$19.90`, and add a unittest test in `test_price.py`. Run it with `python3 -m unittest`.
