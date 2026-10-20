"""Where items are kept."""

from stock.storage.json_store import JsonStore

BACKENDS = {"json": JsonStore}


def open_store(kind, path):
    if kind not in BACKENDS:
        raise ValueError(f"Unknown store {kind}")
    return BACKENDS[kind](path)
