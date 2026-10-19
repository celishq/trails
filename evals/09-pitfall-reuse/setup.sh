#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
cat > catalog.py <<'EOF'
import json
import os
import urllib.request

LOCAL = {"A1": 250, "B2": 1200, "C3": 75}


def load():
    if os.environ.get("SHELF_OFFLINE") == "1":
        return dict(LOCAL)
    with urllib.request.urlopen("http://127.0.0.1:9/catalog.json", timeout=2) as response:
        return json.load(response)
EOF
cat > shelf.py <<'EOF'
import catalog


def price_of(sku):
    return catalog.load()[sku]
EOF
cat > test_shelf.py <<'EOF'
import unittest

from shelf import price_of


class PriceTest(unittest.TestCase):
    def test_known_sku(self):
        self.assertEqual(price_of("A1"), 250)

    def test_unknown_sku(self):
        with self.assertRaises(KeyError):
            price_of("Z9")


if __name__ == "__main__":
    unittest.main()
EOF
id=$(celis step add --new --at shelf.py -- "Add price_of(sku) to shelf.py" | grep -E '^[0-9a-f]{64}$' | head -n 1)
celis step start "$id"
celis step done "$id" --as-planned --pitfall "tests need SHELF_OFFLINE=1: without it catalog.py calls the catalog server and every test fails with Connection refused" -- "SHELF_OFFLINE=1 python3 -m unittest: Ran 2 tests, OK"
