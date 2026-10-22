#!/usr/bin/env python3
"""The pull arm's prompt hook: says how to search the rules, and gives none.

  trigger.py pull   UserPromptSubmit

Each call appends a line to $CELIS_HOME/jit.log.
"""

import json
import os
import sys
from pathlib import Path

PULL = ("celis holds the rules of past work, and none of them is in your context. Search them before you write "
        "code, and again before each new kind of change: celis grep -i -E ' rule [0-9a-f]{8} live .*(<word>|<word>)', "
        "with the words of the files, the code and the work. Keep each rule it gives that applies.")


def main():
    mode = sys.argv[1]
    home = Path(os.environ.get("CELIS_HOME", "."))
    with (home / "jit.log").open("a") as log:
        log.write(json.dumps({"mode": mode, "rules": 0, "texts": []}) + "\n")
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "UserPromptSubmit", "additionalContext": PULL}}))


if __name__ == "__main__":
    main()
