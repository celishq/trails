#!/usr/bin/env python3
"""The triggered and pull arms' Claude Code hook.

  trigger.py ask    UserPromptSubmit; says how rules come, and gives the situation rules that a Haiku
                    picker finds the ask must keep, up to jit.KEEP
  trigger.py pre    PreToolUse on Write, Edit, MultiEdit and Bash; gives each rule whose trigger the call
                    matches (triggers.py), once a session; a Bash command acts on each file it names
  trigger.py post   PostToolUse on Bash; gives each rule whose file trigger matches a file the command
                    changed, once a session
  trigger.py pull   UserPromptSubmit; says how to search the rules, and gives none

Each call appends a line to $CELIS_HOME/jit.log.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

import jit

PATH = re.compile(r"[\w./-]+\.\w+")
HERE = Path(__file__).resolve().parent
ASK = "celis gives you the rules of past work here, and before each edit or command they act on. Keep each one."
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


def matches(pattern, text):
    """Whether `text` holds `pattern`; an empty pattern matches anything, and a broken one nothing."""
    try:
        return not pattern or re.search(pattern, text) is not None
    except re.error:
        return False


def written(trigger, path, text):
    """Whether writing `text` to the file at `path` matches `trigger`."""
    return any(jit.holds(p, path) for p in trigger["paths"]) and matches(trigger["content"], text)


def fired(trigger, tool, cwd):
    """Whether the call `tool` makes matches `trigger`."""
    if "command" in tool:
        command = tool["command"]
        return (bool(trigger["command"]) and matches(trigger["command"], command)
                or any(written(trigger, path, command) for path in set(PATH.findall(command))))
    path = os.path.relpath(tool.get("file_path", ""), cwd)
    text = tool.get("content", "") + tool.get("new_string", "") + "".join(e.get("new_string", "") for e in tool.get("edits", []))
    return written(trigger, path, text)


def changed(cwd):
    """The text of each file that differs from the last commit, by its path."""
    return {path: Path(cwd, path).read_text(errors="replace") for path in jit.changed(cwd) if Path(cwd, path).is_file()}


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
    state = home / f"trigger-{event.get('session_id', 'none')}.json"
    seen = set(json.loads(state.read_text())) if state.exists() else set()
    known = triggers()
    if mode == "pull":
        note, event_name, found = PULL, "UserPromptSubmit", []
    elif mode == "ask":
        found = situation(cwd, event.get("prompt", ""), home, known)
        note, event_name = ASK, "UserPromptSubmit"
    elif mode == "post":
        files = changed(cwd)
        found = [t for i, t in live(cwd) if t not in seen and t in known
                 and any(written(known[t], path, text) for path, text in files.items())]
        found = [r[2] for r in jit.fit([("", "", t) for t in found])]
        note, event_name = "celis rules for the files this command changed. Check them and fix what breaks one:", "PostToolUse"
    else:
        tool = event.get("tool_input", {})
        found = [t for i, t in live(cwd) if t not in seen and t in known and fired(known[t], tool, cwd)]
        found = [r[2] for r in jit.fit([("", "", t) for t in found])]
        note, event_name = "celis rules for this call. Keep each one in what you write:", "PreToolUse"
    seen.update(found)
    state.write_text(json.dumps(sorted(seen)))
    with (home / "jit.log").open("a") as log:
        log.write(json.dumps({"mode": mode, "rules": len(found), "texts": found}) + "\n")
    if mode in ("pre", "post") and not found:
        return
    text = note + "".join(f"\n- {t}" for t in found)
    print(json.dumps({"hookSpecificOutput": {"hookEventName": event_name, "additionalContext": text}}))


if __name__ == "__main__":
    main()
