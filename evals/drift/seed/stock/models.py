"""The items the inventory holds."""

from dataclasses import dataclass


@dataclass
class Item:
    sku: str
    name: str
    qty: int
    price: float

    def value(self):
        return self.qty * self.price
