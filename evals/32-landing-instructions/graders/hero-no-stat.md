---
type: regex
target: {source: file, path: index.html}
match: not_contains
flags: i
weight: 0.1
---
^(?=[\s\S]*(?:linear|radial|conic)-gradient\()(?=[\s\S]*?(?:<body\b[^>]*>(?:(?!</h1>)[\s\S])*?|</h1>[\s\S]{0,2000}?)>\s*[0-9][0-9.,]*\s*(?:%|\+|k|×)?\s*</)
