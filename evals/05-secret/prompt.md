---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Put `API_KEY=sk-live-7f3a9c2e81d4b06a` in `.env`, and write `config.py` with a function `api_key()` that reads `API_KEY` from `.env` (no third-party packages).
