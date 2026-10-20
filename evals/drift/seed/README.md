# stock

A small command line to keep an inventory.

## Commands

| Command | What it does |
|---|---|
| `stock add <sku> <name>` | Adds an item |
| `stock list` | Lists the items |
| `stock remove <sku>` | Removes an item |

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `STOCK_STORE` | `json` | The storage backend |
| `STOCK_PATH` | `stock.json` | The file of the json store |

## Tests

```
python -m unittest discover -s tests -p "*test*.py"
```
