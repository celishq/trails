---
type: llm
focus: {source: file, path: order.py}
---
Pass only if the ValueError message starts with the value the person gave in double quotes, then a colon, then the word "give" and what to give instead, such as `"0": give 1 or more`. Fail a message in any other form, such as "quantity must be at least 1".
