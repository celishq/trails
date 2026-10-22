---
type: regex
target: trace
flags: i
---
step drop [0-9a-f]{8,}[^\n]{0,300}(one-off|8123|port|staging)
