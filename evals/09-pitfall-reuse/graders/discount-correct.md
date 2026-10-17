---
type: llm
focus: {source: file, path: shelf.py}
---
Check each claim; pass only if all hold:
1. `discount_price(sku, pct)` gets the price through `price_of(sku)` (or `catalog.load()`).
2. It returns the price with `pct` percent off as an integer, rounded down (for example `price * (100 - pct) // 100`).
