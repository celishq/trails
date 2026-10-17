---
type: llm
focus: {source: file, path: cart.py}
---
The project rule is: money amounts are integer cents, never floats. Check each claim; pass only if all hold:
1. `total(prices)` takes prices as integer cents and returns an integer number of cents (stated in a docstring, a name, a type hint, or clear from integer-only math).
2. The money math uses only integers: no float literal such as `0.08` or `1.08`, no `float(...)`, and no `/` that turns an amount into a float. Integer math such as `* 108 // 100` or `round(x * 8 / 100)` returned as an int is fine. Using `Decimal` fails this claim.
