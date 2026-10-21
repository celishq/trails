"""The command line of stock."""

import argparse
import sys

from stock.config import setting
from stock.const import DEFAULT_PATH, DEFAULT_STORE, EXIT_FAILURE
from stock.errors import StockError


def out(text):
    """Writes one line to the terminal."""
    print(text)


def parser():
    p = argparse.ArgumentParser(prog="stock", description="Keep an inventory.")
    sub = p.add_subparsers(dest="command", required=True)
    add = sub.add_parser("add", help="Add an item.")
    add.add_argument("sku")
    add.add_argument("name")
    add.add_argument("--qty", type=int, default=0, help="How many there are.")
    add.add_argument("--price", type=float, default=0.0, help="Price of one.")
    sub.add_parser("list", help="List the items.")
    remove = sub.add_parser("remove", help="Remove an item.")
    remove.add_argument("sku")
    return p


def main(argv=None):
    from stock.commands import COMMANDS
    from stock.storage import open_store

    args = parser().parse_args(argv)
    store = open_store(setting("store", DEFAULT_STORE), setting("path", DEFAULT_PATH))
    try:
        return COMMANDS[args.command](store, args)
    except StockError as e:
        out(f"error: {e}")
        return EXIT_FAILURE


if __name__ == "__main__":
    sys.exit(main())
