"""The add command."""

from stock.cli import out
from stock.const import EXIT_OK
from stock.models import Item


def cmd_add(store, args):
    items = store.load()
    if any(item.sku == args.sku for item in items):
        raise ValueError(f"Item {args.sku} already exists")
    items.append(Item(args.sku, args.name, args.qty, args.price))
    store.save(items)
    out(f"added {args.sku}")
    return EXIT_OK
