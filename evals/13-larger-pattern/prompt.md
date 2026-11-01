---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add `clients_csv(clients)` to `report.py`: the clients as CSV text, one row each, with a header row. Each client is a dict with `id` and `name`. Add a test for it to `test_report.py`, and run the tests with `python3 -m unittest`.
