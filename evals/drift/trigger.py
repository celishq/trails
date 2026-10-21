#!/usr/bin/env python3
"""The triggered and pull arms' prompt hook; the triggered arm gets the rules of each tool call from the
plugin's rules.sh.

  trigger.py ask    UserPromptSubmit; says how rules come, and gives the situation rules that a Haiku
                    picker finds the ask must keep, up to jit.KEEP
  trigger.py pull   UserPromptSubmit; says how to search the rules, and gives none

Each call appends a line to $CELIS_HOME/jit.log.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

import jit

HERE = Path(__file__).resolve().parent
ASK = "celis gives you the rules of past work here, and before each tool call they act on. Keep each one."
PULL = ("celis holds the rules of past work, and none of them is in your context. Search them before you write "
        "code, and again before each new kind of change: celis grep -i -E ' rule [0-9a-f]{8} live .*(<word>|<word>)', "
        "with the words of the files, the code and the work. Keep each rule it gives that applies.")


def triggers():
    """The trigger of each rule, by its text."""
    data = lambda name: json.loads((HERE / "data" / name).read_text())
    by_id = data("triggers.json")
    texts = [(t["id"], t["text"]) for t in data("targets.json")] + [(r["id"], r["text"]) for r in data("filler.json")["rules"]]
    return {text: by_id[i] for i, text in texts if i in by_id}


def live(cwd):
    """The live rules, as (id, text)."""
    return [(m[2], m[4]) for m in map(jit.LINE.search, subprocess.run(
        ["celis", "grep", " live {me} "], cwd=cwd, capture_output=True, text=True).stdout.splitlines()) if m]


def situation(cwd, ask, home, known):
    """The situation rules a Haiku picker finds `ask` must keep, up to jit.KEEP."""
    lines = [f"- {t}" for _, t in live(cwd) if known.get(t, {}).get("situation")]
    if not lines:
        return []
    answer = jit.haiku(jit.PICK.format(ask=ask, rules="\n".join(lines)), home)
    return list(dict.fromkeys(l.strip()[2:] for l in answer.splitlines() if l.strip() in lines))[:jit.KEEP]


def main():
    mode, event = sys.argv[1], json.load(sys.stdin)
    cwd, home = event.get("cwd", "."), Path(os.environ.get("CELIS_HOME", "."))
    known = triggers()
    if mode == "pull":
        note, event_name, found = PULL, "UserPromptSubmit", []
    else:
        found = situation(cwd, event.get("prompt", ""), home, known)
        note, event_name = ASK, "UserPromptSubmit"
    with (home / "jit.log").open("a") as log:
        log.write(json.dumps({"mode": mode, "rules": len(found), "texts": found}) + "\n")
    text = note + "".join(f"\n- {t}" for t in found)
    print(json.dumps({"hookSpecificOutput": {"hookEventName": event_name, "additionalContext": text}}))


if __name__ == "__main__":
    main()
