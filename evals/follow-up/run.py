#!/usr/bin/env python3
"""The follow-up bench: one Claude Code session gets an ask, then a follow-up prompt that tests the ask's work,
and passes when the session's store holds one trail, with a step from each prompt.

  run.py <arm> [--model sonnet|opus] [--runs N] [--jobs N]

  arm    base (the installed celis and the plugin of the main checkout), new (the celis in $CELIS_BIN and the
         plugin in $CELIS_PLUGIN, else this worktree's)

Each run starts in a new git folder with a new store. Runs live in $FOLLOW_UP_RUNS (default /tmp/celis-follow-up);
each run appends one line to results/<arm>-<model>.jsonl.
"""

import argparse
import json
import os
import shutil
import subprocess
import uuid
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
MAIN = Path(os.environ.get("FOLLOW_UP_MAIN", ROOT.parents[2] if ROOT.parent.name == "worktrees" else ROOT))
RUNS = Path(os.environ.get("FOLLOW_UP_RUNS", "/tmp/celis-follow-up"))
RESULTS = HERE / "results"
MODELS = {"sonnet": "claude-sonnet-5-5", "opus": "claude-opus-5-5"}
AGENT = ["--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions", "--setting-sources",
         "project", "--strict-mcp-config", "--max-budget-usd", "3"]
BINS = {"base": Path(shutil.which("celis") or "/usr/local/bin/celis").parent,
        "new": Path(os.environ.get("CELIS_BIN", ROOT / "crates" / "target" / "release"))}
PLUGINS = {"base": MAIN / "plugins" / "celis", "new": Path(os.environ.get("CELIS_PLUGIN", ROOT / "plugins" / "celis"))}
DONE = "Go on to the end without asking."
PROMPTS = [
    "Write `parse_port(text)` in `port.py`: it returns the port as an int and raises a ValueError for text that "
    "is not a port from 1 to 65535. Add a unittest test in `test_port.py`, run it with `python3 -m unittest`, "
    "and commit.",
    "Now test it at the edges: add tests for 0, 65536 and ' 80 ', run them, and commit.",
]


def sh(*args, cwd, env=None):
    return subprocess.run(args, cwd=cwd, env=env, check=True, capture_output=True, text=True).stdout.strip()


def steps(home):
    """The step records of the store at `home`."""
    found = []
    for path in home.rglob("*"):
        if not path.is_file() or "blobs" in path.parts:
            continue
        for line in path.read_text(errors="ignore").splitlines():
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                continue
            tags = record.get("tags") if isinstance(record, dict) else None
            if record.get("kind") == 1111 and tags and ["l", "step", "celis.step"] in tags:
                found.append(record)
    return {r["id"]: r for r in found}


def turn(arm, model, work, home, prompt, session, resume, log):
    """One prompt of the session `session` in `work`; its cost in dollars."""
    env = dict(os.environ, CELIS_HOME=str(home), PATH=f"{BINS[arm]}:{os.environ['PATH']}")
    env.pop("CLAUDE_CODE_SESSION_ID", None)
    flags = ["--settings", json.dumps({"autoMemoryEnabled": False}), "--plugin-dir", str(PLUGINS[arm])]
    flags += ["--resume", session] if resume else ["--session-id", session]
    with log.open("w") as out:
        try:
            subprocess.run(["claude", "-p", f"{prompt}\n\n{DONE}", "--model", MODELS[model], *AGENT, *flags],
                           cwd=work, env=env, stdout=out, stderr=subprocess.DEVNULL, timeout=1200)
        except subprocess.TimeoutExpired:
            pass
    lines = [json.loads(l) for l in log.read_text().splitlines() if l.startswith("{")]
    result = next((l for l in reversed(lines) if l.get("type") == "result"), {})
    return result.get("total_cost_usd", 0)


def run(arm, model, name, number):
    """Does one run and appends its line to the results."""
    root = RUNS / name / str(number)
    shutil.rmtree(root, ignore_errors=True)
    (root / "work").mkdir(parents=True)
    (root / "home").mkdir()
    sh("git", "init", "-q", cwd=root / "work")
    sh("git", "commit", "-q", "--allow-empty", "-m", "start", cwd=root / "work")
    logs = RESULTS / "transcripts" / name
    logs.mkdir(parents=True, exist_ok=True)
    session = str(uuid.uuid4())
    cost, written = 0, []
    for index, prompt in enumerate(PROMPTS):
        before = set(steps(root / "home"))
        cost += turn(arm, model, root / "work", root / "home", prompt, session, index > 0,
                     logs / f"{number}-{index + 1}.jsonl")
        written.append(len(set(steps(root / "home")) - before))
    asks = [r["id"][:8] for r in steps(root / "home").values() if not any(t[0] == "under" for t in r["tags"])]
    passed = len(asks) == 1 and all(written)
    line = {"arm": arm, "model": model, "run": number, "asks": asks, "steps": written, "passed": passed,
            "cost": round(cost, 2)}
    with (RESULTS / f"{name}.jsonl").open("a") as out:
        out.write(json.dumps(line) + "\n")
    print(f"{name} {number}: {'pass' if passed else 'fail'}, {len(asks)} trails, steps {written}", flush=True)
    return passed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("arm", choices=["base", "new"])
    parser.add_argument("--model", choices=MODELS, default="opus")
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--jobs", type=int, default=3)
    args = parser.parse_args()
    RESULTS.mkdir(exist_ok=True)
    name = f"{args.arm}-{args.model}"
    (RESULTS / f"{name}.jsonl").unlink(missing_ok=True)
    with ThreadPoolExecutor(args.jobs) as pool:
        passed = list(pool.map(lambda n: run(args.arm, args.model, name, n), range(1, args.runs + 1)))
    print(f"{name}: {sum(passed)}/{len(passed)} one trail", flush=True)


if __name__ == "__main__":
    main()
