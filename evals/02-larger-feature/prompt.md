---
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Build `inventory/`, a small Python package for a shop's stock, with no third-party packages:
- `inventory/models.py`: an `Item` dataclass (sku, name, quantity, price) with validation (no negative quantity or price).
- `inventory/store.py`: load and save items to `stock.json`.
- `inventory/cli.py`: commands `add`, `remove <sku>`, `restock <sku> <n>`, `list`, and `report`, which prints the total stock value and the items with quantity under 5.
- `inventory/export.py`: write the items to `stock.csv`.
Add unittest tests for each module in `tests/`, and run them with `python3 -m unittest discover tests`.
