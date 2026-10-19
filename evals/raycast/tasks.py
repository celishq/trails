#!/usr/bin/env python3
"""Writes the Raycast chain's tasks, in chain order, to data/tasks.json.

  tasks.py    reads the PRs from GitHub through gh, and the clone in cache/extensions

A task is one merged PR that changes one extension. Its base is the parent of the squash merge; a new extension
has no folder there. The prompt is the PR's title and description, without the screencast, the checklist and the
lines that report the author's tests.
"""

import json
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE / "cache" / "extensions"
PRS = [31771, 30017, 30395, 30336, 31801, 30016, 31207, 31850, 30215, 31374]


def run(*args, cwd=None):
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout


def prompt(title, body):
    """The PR's title and its description up to the screencast or checklist, with HTML comments removed."""
    body = re.sub(r"<!--.*?-->", "", body or "", flags=re.S)
    body = re.split(r"^## (Screencast|Checklist)", body, flags=re.M)[0]
    body = re.sub(r"^## Description\s*", "", body.strip())
    body = "\n".join(l for l in body.splitlines() if not re.search(r"\b(vitest|jest|npm test|\d+ tests?)\b", l, re.I))
    return f"{title}\n\n{body.strip()}"


def task(number):
    pr = json.loads(run("gh", "pr", "view", str(number), "-R", "raycast/extensions", "--json",
                        "title,body,mergeCommit,files"))
    merge = pr["mergeCommit"]["oid"]
    extension = pr["files"][0]["path"].split("/")[1]
    base = run("git", "rev-parse", f"{merge}^", cwd=REPO).strip()
    new = subprocess.run(["git", "cat-file", "-e", f"{base}:extensions/{extension}"], cwd=REPO,
                         stderr=subprocess.DEVNULL).returncode != 0
    return {"pr": number, "extension": extension, "base": base, "merge": merge, "new": new,
            "prompt": prompt(pr["title"], pr["body"])}


def prefetch(tasks):
    """Downloads the files of each task's extension at its base and merge, so runs need no network."""
    for t in tasks:
        for commit in (t["base"], t["merge"]):
            run("git", "ls-tree", "-r", "-l", commit, f"extensions/{t['extension']}", cwd=REPO)


def main():
    tasks = [task(n) for n in PRS]
    prefetch(tasks)
    (HERE / "data").mkdir(exist_ok=True)
    (HERE / "data" / "tasks.json").write_text(json.dumps(tasks, indent=2) + "\n")


if __name__ == "__main__":
    main()
