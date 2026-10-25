---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Write the cake page in `cake.md`: a short blurb, then a table of the five cakes we sell with their prices in euros: lemon 4, plum 5, carrot 4, chocolate 6, cheese 5.
