---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Build `notes.py`, a command-line notebook with no third-party packages: `add <text>`, `list`, `find <word>` and `delete <n>`, kept in `notes.json`, with `--tag <tag>` on `add` and `list`. Add unittest tests in `test_notes.py` and run them with `python3 -m unittest`.
