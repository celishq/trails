"""The interface of a store."""

from abc import ABC, abstractmethod


class Store(ABC):
    def __init__(self, path):
        self.path = path

    @abstractmethod
    def load(self):
        """All items in the store."""

    @abstractmethod
    def save(self, items):
        """Replaces the items in the store."""
