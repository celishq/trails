#!/usr/bin/env python3
"""Scores one extension folder against its base commit: each rule broken or kept, and whether it builds and lints.

  check.py <repo> [base]    prints one JSON line; base is the repo's first commit when left out
"""

import json
import re
import subprocess
import sys
from pathlib import Path

TEST = re.compile(r"(^|/)(tests?|__tests__)/|\.(test|spec)\.[cm]?[jt]sx?$")
LOADS = re.compile(r"\bawait\b|\bfetch\(|\.then\(")
FAILURE = re.compile(r"showToast\(\s*(\{[^}]*)?Toast\.Style\.Failure")


def git(repo, *args):
    return subprocess.run(["git", *args], cwd=repo, capture_output=True, text=True).stdout


def added(repo, base):
    """The lines each file gains against base, untracked files included, node_modules and dist left out."""
    git(repo, "add", "-A", "-N", ".")
    lines, path = {}, None
    for line in git(repo, "diff", base, "--", ".", ":!node_modules", ":!dist", ":!package-lock.json").splitlines():
        if line.startswith("+++ "):
            path = line[6:] if line.startswith("+++ b/") else None
            lines.setdefault(path, [])
        elif path and line.startswith("+") and not line.startswith("+++"):
            lines[path].append(line[1:])
    return {p: ls for p, ls in lines.items() if p}


def effects(source):
    """The bodies of the useEffect calls in the source, up to their closing line."""
    bodies, depth, body = [], 0, None
    for line in source:
        if body is None and "useEffect(" in line:
            body, depth = [], 0
        if body is not None:
            body.append(line)
            depth += line.count("(") - line.count(")")
            if depth <= 0:
                bodies.append("\n".join(body))
                body = None
    return bodies


def rules(repo, base):
    """The rules the work breaks: rule id → the evidence."""
    diff = added(repo, base)
    src = {p: ls for p, ls in diff.items() if p.startswith("src/")}
    broken = {}
    headings = [l for l in diff.get("CHANGELOG.md", []) if l.startswith("## ")]
    if not any(l.rstrip().endswith("{PR_MERGE_DATE}") for l in headings) or any(
            re.search(r"\d{4}-\d\d-\d\d", l) for l in headings):
        broken["changelog"] = headings or ["no new CHANGELOG.md entry"]
    if hits := [p for p, ls in src.items() if FAILURE.search("\n".join(ls))]:
        broken["failure-toast"] = hits
    if hits := [p for p, ls in src.items() if any(re.search(r"console\.log\(", l) for l in ls)]:
        broken["console"] = hits
    if hits := [p for p, ls in src.items() if any(LOADS.search(b) for b in effects(ls))]:
        broken["data-hooks"] = hits
    tests = [p for p in diff if TEST.search(p)]
    if any(re.search(r'"test"\s*:', l) for l in diff.get("package.json", [])):
        tests.append("package.json test script")
    if tests:
        broken["no-tests"] = tests
    return broken


def tool(repo, *args):
    """Whether a ray command passes in the repo, and the tail of its output."""
    result = subprocess.run(["npx", "--no-install", "ray", *args], cwd=repo, capture_output=True, text=True,
                            timeout=600)
    return {"ok": result.returncode == 0, "tail": (result.stdout + result.stderr)[-400:]}


def check(repo, base=None):
    repo = Path(repo)
    base = base or git(repo, "rev-list", "--max-parents=0", "HEAD").split()[0]
    return {"broken": rules(repo, base), "build": tool(repo, "build", "-e", "dist", "-o", "dist"),
            "lint": tool(repo, "lint")}


if __name__ == "__main__":
    print(json.dumps(check(*sys.argv[1:])))
