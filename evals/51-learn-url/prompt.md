---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash, Skill]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
/celis:learn the landing page of Orto at site/index.html, as a web page in a browser. Do not wait for me: keep all the drafts.
