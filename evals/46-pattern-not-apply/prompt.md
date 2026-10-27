---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Write `fx_rate(usd_cents, eur_cents)` in `fx.py`: it returns how many euros one dollar buys, as a float ratio for a chart, such as `0.92`. Add a unittest test in `test_fx.py` and run it with `python3 -m unittest`.
