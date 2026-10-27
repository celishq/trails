---
type: regex
target: {source: file, path: index.html}
flags: i
weight: 0.4
---
:focus(?:-visible)?\b[^{]*\{[^}]*(?:outline(?!\s*:\s*(?:none|0)\s*[;}])|box-shadow|border|background|text-decoration|color)
