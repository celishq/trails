---
type: regex
target: trace
---
step drop [0-9a-f]{8,}[^\n]*(?:ruff|pyproject|lint)[^\n]*holds it
