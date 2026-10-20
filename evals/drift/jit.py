#!/usr/bin/env python3
"""The jit arm's Claude Code hook: gives the agent the rules of a file when it reads or writes it, and the
rules of what it changed when it closes a step.

  jit.py read    PostToolUse on Read, Write, Edit and MultiEdit
  jit.py done    PostToolUse on Bash; acts only on `celis step done`
  jit.py ask     UserPromptSubmit; names the pick agents to run, one for each JIT_PAGE `me` rules
  jit.py pick    UserPromptSubmit; runs a Haiku picker on each JIT_PAGE `me` rules in parallel, cuts their
                 pick to KEEP rules with one more Haiku call, and gives only those to the agent; with
                 JIT_STAGE=file it keeps the whole pick for the session, and each read gives the file up to
                 KEEP of the picked rules that one more Haiku call finds act on it

A rule's place is a file, a folder or a glob. A `me` rule holds where it shares MATCH words with the file;
with JIT_ME=off, the hook gives no `me` rule.
A session sees each rule once on a read and once on a review.
One message holds up to BUDGET characters, the rules of the deepest places first, or with JIT_RANK=words the
rules that share the most words with the file; `me` rules come after place rules; the rest wait for the next.
Each call appends a line to $CELIS_HOME/jit.log.
"""

import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from fnmatch import fnmatch
from pathlib import Path

BUDGET = 9000
KEEP = 20
PICK = """A coding agent got this ask:

{ask}

Below are rules, one a line. Copy each rule that the work of this ask must keep: the rules about the
files, the language and the kind of code it will write or change, and about the way of work it will do
(tests, changelog, docs). Leave out rules about tools, files or work the ask does not touch. Answer only
with the lines you copied, unchanged, or "nothing".

{rules}"""
FILE = """A coding agent got this ask:

{ask}

It is about to change the file `{path}`, which begins:

{head}

Below are rules for the ask, one a line. Copy up to {keep} rules that act on this file and the change the
agent will make in it. Answer only with the lines you copied, unchanged, or "nothing".

{rules}"""
MERGE = """A coding agent got this ask:

{ask}

Below are rules that may apply to it, one a line. Copy the {keep} rules that the work of this ask most
needs to keep, the ones it is most likely to break without being told. Answer only with the lines you
copied, unchanged.

{rules}"""
WORD = re.compile(r"[a-z_][a-z0-9_]{2,}")
MATCH = 3
LINE = re.compile(r"\[([^]]*)\] rule ([0-9a-f]{8}) live \{(?:at:)?([^}]+)\} (.*)$")


def rules(cwd):
    """The live place rules and `me` rules, as (id, place, text), where a `me` rule's place is `me`; none of
    the bench's `other` project."""
    out = subprocess.run(["celis", "grep", "-E", r" live \{(at:|me\})"], cwd=cwd, capture_output=True,
                         text=True).stdout
    return [m.groups()[1:] for m in map(LINE.search, out.splitlines()) if m and m[1] != "other"]


def holds(place, path):
    """Whether a rule at `place` holds for the file at `path`."""
    return (path == place or path.startswith(place.rstrip("/") + "/") or fnmatch(path, place)
            or fnmatch(path, place.replace("**/", "")))


def changed(cwd):
    """The files the work changed since the first commit, and the new ones."""
    git = lambda *a: subprocess.run(["git", *a], cwd=cwd, capture_output=True, text=True).stdout.split()
    root = git("rev-list", "--max-parents=0", "HEAD")[-1:]
    return set(git("diff", "--name-only", *root) + git("ls-files", "--others", "--exclude-standard"))


def depth(place):
    return place.count("/") + (not place.endswith("/") and "." in place)


def fit(found, text=""):
    """The rules of `found` that fit in one message, the first by the rank of JIT_RANK."""
    words = set(WORD.findall(text.lower()))
    shared = lambda r: len(set(WORD.findall(r[2].lower())) & words)
    found = [r for r in found if r[1] != "me" or os.environ.get("JIT_ME") != "off" and (not text or shared(r) >= MATCH)]
    me = lambda r: r[1] == "me"
    rank = ((lambda r: (me(r), -shared(r), -depth(r[1]))) if os.environ.get("JIT_RANK") == "words"
            else lambda r: (me(r), -depth(r[1]), -shared(r)))
    kept, size = [], 0
    for rule in sorted(found, key=rank):
        size += len(rule[2]) + 3
        if size > BUDGET:
            break
        kept.append(rule)
    return kept


def ask(cwd):
    """The note that names a celis:pick agent for each page of JIT_PAGE `me` rules."""
    out = subprocess.run(["celis", "grep", "-c", " live {me} "], cwd=cwd, capture_output=True, text=True).stdout
    count, page = int(out.strip() or 0), int(os.environ.get("JIT_PAGE", "100"))
    pages = ", ".join(f"{i + 1},{min(i + page, count)}" for i in range(0, count, page))
    return (f"celis: with the celis:past agent, run a celis:pick agent for each of these pages, all in one "
            f"message, each with the ask, the places it may touch and its page: {pages}. Keep each rule "
            f"they bring.") if count else ""


def haiku(prompt, home):
    """The text of one tool-less Haiku answer to `prompt`; logs its cost."""
    out = subprocess.run(["claude", "-p", prompt, "--model", "claude-haiku-5-5", "--tools", "", "--output-format",
                          "json", "--no-session-persistence", "--strict-mcp-config", "--setting-sources", "project"],
                         capture_output=True, text=True, cwd=home).stdout
    try:
        answer = json.loads(out)
    except ValueError:
        return ""
    with (home / "jit.log").open("a") as log:
        log.write(json.dumps({"mode": "haiku", "cost": answer.get("total_cost_usd", 0)}) + "\n")
    return answer.get("result", "")


def pick(cwd, ask, home, state):
    """The note that gives the KEEP `me` rules that pickers and a merge chose for `ask`; with JIT_STAGE=file,
    the note that rules come at each file, with the whole pick kept in `state`."""
    out = subprocess.run(["celis", "grep", " live {me} "], cwd=cwd, capture_output=True, text=True).stdout
    lines = [l.strip() for l in out.splitlines() if " live {me} " in l]
    page = int(os.environ.get("JIT_PAGE", "100"))
    pages = ["\n".join(lines[i:i + page]) for i in range(0, len(lines), page)]
    with ThreadPoolExecutor(len(pages) or 1) as pool:
        answers = list(pool.map(lambda p: haiku(PICK.format(ask=ask, rules=p), home), pages))
    known = set(lines)
    chosen = list(dict.fromkeys(l.strip() for a in answers for l in a.splitlines() if l.strip() in known))
    if os.environ.get("JIT_STAGE") == "file":
        state.write_text(json.dumps({"read": {}, "review": [], "ask": ask, "pool": chosen}))
        with (home / "jit.log").open("a") as log:
            log.write(json.dumps({"mode": "pool", "rules": len(chosen), "texts": [LINE.search(l)[4] for l in chosen]}) + "\n")
        return "celis: you get the rules of each file when you read it, and of a new file when you write it."
    if len(chosen) > KEEP:
        merged = haiku(MERGE.format(ask=ask, keep=KEEP, rules="\n".join(chosen)), home)
        chosen = [l.strip() for l in merged.splitlines() if l.strip() in known][:KEEP]
    with (home / "jit.log").open("a") as log:
        log.write(json.dumps({"mode": "pick", "rules": len(chosen), "picked": len(known & set(
            l.strip() for a in answers for l in a.splitlines())), "texts": [LINE.search(l)[4] for l in chosen]}) + "\n")
    rules = "".join(f"- {LINE.search(l)[4]}\n" for l in dict.fromkeys(chosen))
    return f"celis rules for this ask. Keep each one in what you write:\n{rules}" if rules else ""


def staged(seen, path, text, home):
    """The picked rules of the session that act on the file at `path`, up to KEEP, as (id, place, text)."""
    pool = [l for l in seen.get("pool", []) if LINE.search(l)[2] not in seen["read"]]
    if not pool:
        return []
    answer = haiku(FILE.format(ask=seen["ask"], path=path, head=text[:2000], keep=KEEP, rules="\n".join(pool)), home)
    known = set(pool)
    return [LINE.search(l.strip()).groups()[1:] for l in answer.splitlines() if l.strip() in known][:KEEP]


def main():
    mode, event = sys.argv[1], json.load(sys.stdin)
    cwd, tool = event.get("cwd", "."), event.get("tool_input", {})
    if mode in ("ask", "pick"):
        home = Path(os.environ.get("CELIS_HOME", "."))
        state = home / f"jit-{event.get('session_id', 'none')}.json"
        note = ask(cwd) if mode == "ask" else pick(cwd, event.get("prompt", ""), home, state)
        if note:
            print(json.dumps({"hookSpecificOutput": {"hookEventName": "UserPromptSubmit", "additionalContext": note}}))
        return
    home = Path(os.environ.get("CELIS_HOME", "."))
    state = home / f"jit-{event.get('session_id', 'none')}.json"
    seen = json.loads(state.read_text()) if state.exists() else {"read": {}, "review": []}
    if mode == "read":
        path = os.path.relpath(tool.get("file_path", ""), cwd)
        file = Path(cwd, path)
        text = path + "\n" + (file.read_text(errors="replace") if file.is_file() else "")
        found = fit([r for r in rules(cwd) if r[0] not in seen["read"] and (r[1] == "me" or holds(r[1], path))],
                    text)
        if os.environ.get("JIT_STAGE") == "file" and (tool.get("file_path") and (file.is_file() or "content" in tool)):
            found = fit(found + staged(seen, path, text + tool.get("content", ""), home))
        seen["read"].update({i: p for i, p, _ in found})
        head = f"celis rules for `{path}`. Keep each one in what you write here:"
    else:
        if "step done" not in tool.get("command", ""):
            return
        files = changed(cwd)
        found = fit([r for r in rules(cwd) if r[0] in seen["read"] and r[0] not in seen["review"]
                     and (r[1] == "me" or any(holds(r[1], f) for f in files))])
        seen["review"] += [i for i, _, _ in found]
        head = "celis: before you go on, check your change against each of these rules, and fix what breaks one:"
    state.write_text(json.dumps(seen))
    with (home / "jit.log").open("a") as log:
        log.write(json.dumps({"mode": mode, "path": tool.get("file_path"), "rules": len(found)}) + "\n")
    if found:
        text = head + "\n" + "".join(f"- {t}\n" for _, _, t in found)
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": text}}))


if __name__ == "__main__":
    main()
