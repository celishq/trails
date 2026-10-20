"""The list command."""

from stock.cli import out
from stock.const import EXIT_OK


def cmd_list(store, args):
    for item in sorted(store.load(), key=lambda i: i.sku):
        out(f"{item.sku}\t{item.name}\t{item.qty}\t{item.price:.2f}")
    return EXIT_OK
