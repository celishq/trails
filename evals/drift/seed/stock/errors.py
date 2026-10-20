"""Errors the package raises."""


class StockError(Exception):
    """Base error of the package."""


class NotFound(StockError):
    """An item that does not exist."""


class InvalidItem(StockError):
    """An item with a wrong field."""
