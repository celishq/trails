#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
cat > report.py <<'PY'
import csv
import io


def vendors_csv(vendors):
    out = io.StringIO()
    writer = csv.writer(out, lineterminator="\n")
    writer.writerow(["name", "city"])
    for vendor in vendors:
        writer.writerow([vendor["name"], vendor["city"]])
    return out.getvalue()
PY
cat > test_report.py <<'PY'
import unittest

from report import vendors_csv


class VendorsTest(unittest.TestCase):
    def test_quotes_a_name_with_a_comma(self):
        rows = vendors_csv([{"name": "Acme, Inc.", "city": "Oslo"}])
        self.assertEqual(rows, 'name,city\n"Acme, Inc.",Oslo\n')


if __name__ == "__main__":
    unittest.main()
PY
ask=$(celis step add --new --at report.py -- "Add vendors_csv(vendors) to report.py: the vendors as CSV text, with a header row" | grep -E '^[0-9a-f]{64}$' | head -n 1)
step=$(celis step add --under "$ask" --at report.py --check "python3 -m unittest" -- "Write vendors_csv with a header row and a test of a name with a comma" | grep -E '^[0-9a-f]{64}$' | head -n 1)
celis step start "$ask" >/dev/null
celis step start "$step" >/dev/null
celis step done "$step" --changed "a name with a comma broke a plain join into an extra column, so it writes rows with csv.writer" -- "python3 -m unittest: Ran 1 test, OK" >/dev/null
celis step done "$ask" --as-planned -- "python3 -m unittest: Ran 1 test, OK" >/dev/null
