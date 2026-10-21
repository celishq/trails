"""Settings read from the environment."""

import os


def setting(name, default=None):
    """The value of STOCK_<NAME>, or default."""
    return os.environ.get(f"STOCK_{name.upper()}", default)
