"""The remove command."""

from stock.cli import out
from stock.const import EXIT_OK
from stock.errors import NotFound


def cmd_remove(store, args):
    items = store.load()
    kept = [item for item in items if item.sku != args.sku]
    if len(kept) == len(items):
        raise NotFound(f"No item {args.sku}")
    store.save(kept)
    out(f"removed {args.sku}")
    return EXIT_OK
