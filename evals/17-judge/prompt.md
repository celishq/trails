---
max_turns: 30
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Three results in this folder are done, and I want each judged before I close them. Run the celis:judge agent once for each, and tell me each verdict as it gave it. Do not change any file.

1. `cart.py`. for: the shop shows prices with tax. expect: `total(prices)` returns the total of integer cents with 8% tax added. not: no floats.
2. `slug.py`. for: readable links. expect: `slug(text)` is lower case, with each run of spaces as one dash; `python3 -m unittest` passes.
3. `logo.svg`. for: a new brand. expect: the logo feels friendly and reads well at 16 px on a phone.
