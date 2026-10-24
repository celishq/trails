---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add `word_count(text)` to `text.py`: it counts the words of a text, where a word is a run of letters, digits or apostrophes. Add a unittest test in `test_text.py` and run it with `python3 -m unittest`.
