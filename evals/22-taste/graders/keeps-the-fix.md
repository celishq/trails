---
type: llm
focus: {source: file, path: cart.py}
---
The store held two patterns. One has an example: the person rewrote a float total into integer cents with integer math. The other no file shows: a refund rounds down to the cent. Pass only if `refund` takes and returns integer cents, uses only integer math, with no float literal, no `float(...)` and no `/` on an amount, and rounds down, never up or to the nearest.
