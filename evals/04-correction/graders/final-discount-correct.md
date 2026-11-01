---
type: llm
focus: {source: file, path: price.py}
---
Check each claim; pass only if all hold:
1. `discount(price)` returns the price with 15% off when the price is over 100.
2. It returns the price with 10% off when the price is 100 or less.
