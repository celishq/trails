---
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
`parse_quantity(text)` in `order.py` will soon refuse a quantity below 1 with a ValueError. What should its error message say? Answer in chat with the message text, and change no file.
