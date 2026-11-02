---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Fix greet.py. Never use print in the .py files of this project; always log with the logging module. And greet('') raises ValueError('bad'), which tells me nothing: from now on, in any work, an error message says what went wrong and what to do next.
