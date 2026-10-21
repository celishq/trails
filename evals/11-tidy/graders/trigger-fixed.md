---
type: regex
target: trace
---
decide(?=[^\n]*--replaces [0-9a-f]{8,})(?=[^\n]*--path[ =]['"]?(?!test/\*\.py)[^'" ]*test)[^\n]*one assert
