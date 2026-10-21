import os
import tempfile
import unittest

from stock.models import Item
from stock.storage.json_store import JsonStore


class TestJsonStore(unittest.TestCase):
    def test_round_trip(self):
        with tempfile.TemporaryDirectory() as d:
            store = JsonStore(os.path.join(d, "s.json"))
            store.save([Item("a", "A", 1, 1.0)])
            self.assertEqual(store.load(), [Item("a", "A", 1, 1.0)])

    def test_missing_file_is_empty(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertEqual(JsonStore(os.path.join(d, "none.json")).load(), [])
