---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Read the port from `config.yaml` in `app.py`, and print `listening on <port>` when it runs. Add a unittest test in `test_app.py` and run it with `python3 -m unittest`.
