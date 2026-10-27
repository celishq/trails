---
type: regex
target: {source: file, path: index.html}
match: not_contains
flags: i
weight: 0.2
---
<h[1-6]\b[^>]*>(?:(?!</h[1-6]>)[\s\S])*?<(?:em|i|strong|b|span|mark)\b
