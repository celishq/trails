---
type: regex
target: {source: file, path: index.html}
match: not_contains
flags: i
weight: 0.2
---
^(?:(?=[\s\S]*(?:background(?:-color)?|--[\w-]*(?:bg|background|surface|base|paper|cream)[\w-]*)\s*:\s*#(?:e[89a-f]|f[0-9a-f])(?:e[0-9a-f]|f[0-9a])(?:c[89a-f]|d[0-9a-f]|e[0-9a-f]|f0)\b)(?=[\s\S]*#(?:c[0-9a-f]|d[0-9a-f]|e[0-6])[5-8][0-9a-f][3-6][0-9a-f]\b)|(?=[\s\S]*(?:background(?:-color)?|--[\w-]*(?:bg|background|surface|base|paper|ink)[\w-]*)\s*:\s*#(?:(?:[01][0-9a-f]){3}|[01]{3})\b)(?=[\s\S]*#(?:[0-9a-d][0-9a-f](?:e[0-9a-f]|f[0-9a-f])[0-6][0-9a-f]|(?:e[0-9a-f]|f[0-9a-f])[2-5][0-9a-f][0-3][0-9a-f])\b))
