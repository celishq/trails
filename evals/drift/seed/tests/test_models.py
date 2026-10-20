import unittest

from stock.models import Item


class TestItem(unittest.TestCase):
    def test_value(self):
        self.assertEqual(Item("a", "A", 3, 2.5).value(), 7.5)


if __name__ == "__main__":
    unittest.main()
