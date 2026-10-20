#!/usr/bin/env python3
"""Gives each drift rule the scope an agent would give it from its text, and prints the rules per file.

  scope.py          asks Haiku for the scopes not yet in data/scopes.json, then prints
  scope.py --show   prints only

A scope is a place glob of seed/, `me` for a convention of any project, or `other` for a rule about code that
seed/ does not have. Each replaced version takes the scope of its target.
"""

import json
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import jit
import run

HERE = Path(__file__).resolve().parent
SCOPES = HERE / "data" / "scopes.json"
BATCH = 80
ASK = """You write rules for a coding agent with celis, and each rule gets one scope:
- a place: a file, a folder or a glob of this repo, the narrowest one where the rule acts, such as
  `stock/storage/*.py`, `tests/*.py`, `stock/*.py` or `README.md`;
- `me`: a convention about this kind of work that holds in any project, not tied to this repo's files;
- `other`: the rule is about code, tools or files this repo does not have and would not add.

The repo:
{tree}

Give each rule below its scope. Answer only with one JSON object from rule id to scope, no other text.

{rules}"""


def tree():
    return "\n".join(sorted(str(p.relative_to(HERE / "seed")) for p in (HERE / "seed").rglob("*")
                            if "__pycache__" not in p.parts))


def ask(batch):
    """The scopes Haiku gives `batch`, a list of (id, text); none when its answer is no JSON, so a rerun asks
    again."""
    prompt = ASK.format(tree=tree(), rules="\n".join(f"{i}: {t}" for i, t in batch))
    out = subprocess.run(["claude", "-p", prompt, "--model", run.MODELS["haiku"], "--output-format", "json",
                          "--setting-sources", "project", "--strict-mcp-config", "--no-session-persistence"],
                         cwd=HERE, capture_output=True, text=True).stdout
    try:
        text = json.loads(out)["result"]
        return json.loads(text[text.index("{"):text.rindex("}") + 1])
    except (ValueError, KeyError):
        return {}


def show(scopes, targets, filler):
    """Prints, for each size, the place rules per file of seed/ and the share of `me` and `other` rules."""
    files = [p for p in tree().splitlines() if "." in p]
    by_text = {t["text"]: t["id"] for t in targets} | {t["old"]: t["id"] for t in targets if "old" in t}
    by_text |= {r["text"]: r["id"] for r in filler["rules"]}
    print("rules\tper file (mean, max)\tme\tother")
    for n in (40, 100, 400, 1000, 1600):
        got = [scopes.get(by_text[t], "other") for t in run.memory(n, targets, filler)[0]]
        places = [s for s in got if s not in ("me", "other")]
        per = [sum(jit.holds(p, f) for p in places) for f in files]
        print(f"{n}\t{sum(per) / len(per):.0f}, {max(per)}\t{got.count('me')}\t{got.count('other')}")


def main():
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    filler = json.loads((HERE / "data" / "filler.json").read_text())
    scopes = json.loads(SCOPES.read_text()) if SCOPES.exists() else {}
    if "--show" not in sys.argv:
        todo = [(t["id"], t["text"]) for t in targets + filler["rules"] if t["id"] not in scopes]
        with ThreadPoolExecutor(10) as pool:
            for got in pool.map(ask, [todo[i:i + BATCH] for i in range(0, len(todo), BATCH)]):
                scopes.update(got)
        SCOPES.write_text(json.dumps(scopes, indent=1) + "\n")
    show(scopes, targets, filler)


if __name__ == "__main__":
    main()
