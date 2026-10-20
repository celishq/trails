"""Items kept in one JSON file."""

import json
import logging
import os

from stock.models import Item
from stock.storage.base import Store

log = logging.getLogger(__name__)


class JsonStore(Store):
    def load(self):
        if not os.path.exists(self.path):
            log.info(f"No store at {self.path}, starting empty")
            return []
        with open(self.path) as f:
            return [Item(**row) for row in json.load(f)]

    def save(self, items):
        with open(self.path, "w") as f:
            json.dump([item.__dict__ for item in items], f)
