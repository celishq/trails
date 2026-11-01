---
type: llm
focus: {source: file, path: fx.py}
---
The person asked for a float ratio, such as 0.92, for a chart. A pattern says to keep money amounts as integer cents, but a rate is not an amount, so its forces are not present. Pass only if `fx_rate` returns the ratio as a float, eur_cents / usd_cents, and does not round it to cents or return an integer.
