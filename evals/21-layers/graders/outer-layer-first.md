---
type: llm
focus: trace
---
Pass only if the first job the agent added under the ask made the whole tool work end to end at low detail (each command exists and runs, even if bare), and later jobs added detail to all of it, such as tags, errors and tests; fail if the agent built one command or one part in full detail before the others ran at all.
