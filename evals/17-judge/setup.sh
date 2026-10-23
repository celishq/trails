#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
cat > cart.py <<'PY'
def total(prices):
    """The total of `prices`, in integer cents."""
    return sum(prices)
PY
cat > slug.py <<'PY'
def slug(text):
    """`text` in lower case, with each run of spaces as one dash."""
    return "-".join(text.lower().split())
PY
cat > test_slug.py <<'PY'
import unittest
from slug import slug


class SlugTest(unittest.TestCase):
    def test_spaces_become_one_dash(self):
        self.assertEqual(slug("Hello   Big World"), "hello-big-world")


if __name__ == "__main__":
    unittest.main()
PY
cat > logo.svg <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><circle cx="32" cy="32" r="28" fill="#f2a900"/></svg>
SVG
git init -q && git add -A && git -c user.name=e -c user.email=e@e commit -q -m base
