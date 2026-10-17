---
type: regex
target: {source: file, path: inventory/cli.py}
weight: 0.2
---
(?=[\s\S]*\badd\b)(?=[\s\S]*\bremove\b)(?=[\s\S]*\brestock\b)(?=[\s\S]*\blist\b)(?=[\s\S]*\breport\b)
