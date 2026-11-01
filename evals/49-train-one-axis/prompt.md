---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash, Skill]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
/celis:train the type of the hero screen of Rucola, a pizza delivery app whose each screen sells one pizza. Do not wait for me: when you show the 4 variants, take these marks. The 1st and the 3rd are good, because the bold name holds its own next to the big pizza. The 2nd is bad, because its thin name fades. The 4th is neither. A bold name holds for anything I make. Rucola's ground is always the green #1E3A2F.
