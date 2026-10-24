---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Add a cart to `shop.py`, so the shop can sell to a customer: `Cart.add(sku, price_cents, qty)`, `Cart.remove(sku)`, `Cart.total()` in integer cents, and `Cart.apply(code)` for the codes `TEN` (10% off, rounded down) and `SHIP` (free shipping; shipping is 500 cents under a 5000-cent total, else 0). An unknown code raises `ValueError`. Add unittest tests in `test_shop.py` and run them with `python3 -m unittest`.
