#!/usr/bin/env python3
"""Gives each drift rule of a memory of 1000 the trigger that the agent who wrote it would give it, and
prints how many rules each kind of trigger holds.

  triggers.py          asks Opus for the triggers not yet in data/triggers.json, then prints
  triggers.py --show   prints only

A trigger names the files a rule acts on (`paths`), what their new text must hold (`content`), the shell
commands it acts on (`command`), and whether it acts on a kind of work that no file or command shows
ahead (`situation`).
"""

import json
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import run
import scope

HERE = Path(__file__).resolve().parent
TRIGGERS = HERE / "data" / "triggers.json"
SIZE = 1000
BATCH = 60
ASK = """You are a coding agent. You wrote each rule below with celis while you worked in a project, and
celis now asks you for the trigger of each rule: when the rule must come to the next agent, right before
it acts. A rule that never comes at the right moment is broken; a rule that comes when it does not apply
is noise. The rule may come from another project than this one:

{tree}

A trigger is a JSON object:
- "paths": globs of the files the rule acts on, from the repo root, such as "**/*.py", "tests/*.py" or
  "CHANGELOG.md"; [] when it acts on no file;
- "content": a Python regex that the new text written to such a file must match for the rule to act, such
  as "logging|_LOGGER" or "datetime"; "" when any change of the file is enough;
- "command": a Python regex of the shell commands the rule acts on, such as "^git commit"; "" for none;
- "situation": true when the rule acts on a kind of work that neither a file nor a command shows before
  the agent breaks it, such as a file the change must also touch; else false.

Give each rule below its trigger. Answer only with one JSON object from rule id to trigger, no other text.

{rules}"""


def rules():
    """The (id, text) of each rule a memory of SIZE holds, the replaced versions left out."""
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    filler = json.loads((HERE / "data" / "filler.json").read_text())
    by_text = {t["text"]: t["id"] for t in targets} | {r["text"]: r["id"] for r in filler["rules"]}
    olds = {t["old"] for t in targets if "old" in t}
    return [(by_text[t], t) for t in run.memory(SIZE, targets, filler)[0] if t not in olds]


def ask(batch):
    """The triggers Opus gives `batch`, a list of (id, text); none when its answer is no JSON, so a rerun asks
    again."""
    prompt = ASK.format(tree=scope.tree(), rules="\n".join(f"{i}: {t}" for i, t in batch))
    out = subprocess.run(["claude", "-p", prompt, "--model", run.MODELS["opus"], "--output-format", "json",
                          "--setting-sources", "project", "--strict-mcp-config", "--no-session-persistence"],
                         cwd=HERE, capture_output=True, text=True).stdout
    try:
        text = json.loads(out)["result"]
        return json.loads(text[text.index("{"):text.rindex("}") + 1])
    except (ValueError, KeyError):
        return {}


def show(triggers, todo):
    """Prints how many rules have each kind of trigger, and how many have none yet."""
    got = [triggers[i] for i, _ in rules() if i in triggers]
    count = lambda f: sum(map(f, got))
    print(f"rules\t{len(got)}\nno trigger yet\t{len(todo)}\npaths\t{count(lambda t: bool(t['paths']))}\n"
          f"content\t{count(lambda t: bool(t['content']))}\ncommand\t{count(lambda t: bool(t['command']))}\n"
          f"situation\t{count(lambda t: t['situation'])}")


def main():
    triggers = json.loads(TRIGGERS.read_text()) if TRIGGERS.exists() else {}
    todo = [r for r in rules() if r[0] not in triggers]
    if "--show" not in sys.argv:
        with ThreadPoolExecutor(10) as pool:
            for got in pool.map(ask, [todo[i:i + BATCH] for i in range(0, len(todo), BATCH)]):
                triggers.update(got)
        TRIGGERS.write_text(json.dumps(triggers, indent=1) + "\n")
        todo = [r for r in rules() if r[0] not in triggers]
    show(triggers, todo)


if __name__ == "__main__":
    main()
