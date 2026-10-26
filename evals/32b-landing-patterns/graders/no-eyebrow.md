---
type: regex
target: {source: file, path: index.html}
match: not_contains
weight: 0.2
---
class\s*=\s*["'][^"']*\b(?:eyebrow|kicker|overline|pre-?title|pre-?heading|super-?title)\b|\.([\w-]+)\s*\{[^}]*text-transform\s*:\s*uppercase[\s\S]*class\s*=\s*["'](?:[^"']*\s)?\1(?:\s[^"']*)?["'][^>]*>[^<]{1,80}</[a-z0-9]+>\s*<h[1-6]\b|>\s*[A-Z][A-Z0-9 &·'’.,/-]{5,}\s*</[a-z0-9]+>\s*<h[1-6]\b
