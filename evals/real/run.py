#!/usr/bin/env python3
"""The real-rules bench: an agent does new tasks on celis-trails with a copy of the person's real store, and an
Opus judge checks each diff against every live rule of the project and every `me` rule.

  run.py <arm> [--model sonnet|opus] [--jobs N] [--tasks id,id]

  arm    base (the installed celis and the plugin of the main checkout), new (the celis in $CELIS_BIN and the
         plugin in $CELIS_PLUGIN, else this worktree's), plain (no plugin and no store)

Each task starts from a new clone of the main checkout's HEAD and a new copy of the store, so the real store is
never written. Runs live in $REAL_RUNS (default /tmp/celis-real); each task appends one line to
results/<arm>-<model>.jsonl. The results hold the real rules, so git ignores them.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
MAIN = Path(os.environ.get("REAL_MAIN", ROOT.parents[2] if ROOT.parent.name == "worktrees" else ROOT))
STORE = Path(os.environ.get("REAL_STORE", Path.home() / "Library" / "Application Support" / "celis"))
RUNS = Path(os.environ.get("REAL_RUNS", "/tmp/celis-real"))
RESULTS = HERE / "results"
MODELS = {"sonnet": "claude-sonnet-5-5", "opus": "claude-opus-5-5"}
AGENT = ["--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions", "--setting-sources",
         "project", "--strict-mcp-config", "--max-budget-usd", "6", "--no-session-persistence"]
BINS = {"base": Path(shutil.which("celis") or "/usr/local/bin/celis").parent,
        "new": Path(os.environ.get("CELIS_BIN", ROOT / "crates" / "target" / "release"))}
PLUGINS = {"base": MAIN / "plugins" / "celis", "new": Path(os.environ.get("CELIS_PLUGIN", ROOT / "plugins" / "celis"))}
DONE = "Go on to the end without asking. Do not commit."
JUDGE = """You judge whether a code change keeps the rules of its project. Below are the rules, one a line as
`<id8> {{<scope>}} <rule>`, then the change as a git diff.

For each rule that applies to this change (the change does the work the rule's "When" names), say whether
the change keeps it or breaks it. Leave out each rule that does not apply. Answer with JSON only:
{{"rules": [{{"id": "<id8>", "verdict": "kept" | "broken", "why": "<one short sentence>"}}]}}

Rules:
{rules}

Change:
{diff}
"""


def sh(*args, cwd, env=None, check=True):
    return subprocess.run(args, cwd=cwd, env=env, check=check, capture_output=True, text=True).stdout.strip()


def rules(home):
    """The live patterns of celis-trails and the `me` patterns of the store at `home`, as `<id8> {<scope>} <text>`."""
    env = dict(os.environ, CELIS_HOME=str(home), PATH=f"{BINS['new']}:{os.environ['PATH']}")
    text = sh("celis", "grep", "-E", " pattern [0-9a-f]{8} live ", cwd=MAIN, env=env, check=False)
    lines, project = [], None
    for line in text.splitlines():
        if not line.startswith(" "):
            found = re.search(r"\[([^]]+)\]", line)
            project = found and found.group(1)
            continue
        line = line.strip()
        own = re.match(r"\[([^]]+)\] (.*)", line)
        where, line = (own.group(1), own.group(2)) if own else (project, line)
        got = re.match(r"pattern ([0-9a-f]{8}) live (\{[^}]*\})(?: trigger| old-rule| uses:\S+| misled:\S+| used:\S+)* (.*)", line)
        if got and (where == "celis-trails" or got.group(2) == "{me}"):
            lines.append(f"{got.group(1)} {got.group(2)} {got.group(3)}")
    return lines


def agent(arm, model, work, home, prompt, log):
    """One Claude Code session in `work`; its numbers from the stream-json transcript."""
    env = dict(os.environ, CELIS_HOME=str(home), CARGO_TARGET_DIR=str(work.parent / "target"))
    env.pop("CLAUDE_CODE_SESSION_ID", None)
    flags = ["--settings", json.dumps({"autoMemoryEnabled": False})]
    if arm in BINS:
        env["PATH"] = f"{BINS[arm]}:{env['PATH']}"
        flags += ["--plugin-dir", str(PLUGINS[arm])]
    start = time.time()
    with log.open("w") as out:
        try:
            subprocess.run(["claude", "-p", prompt, "--model", MODELS[model], *AGENT, *flags], cwd=work, env=env,
                           stdout=out, stderr=subprocess.DEVNULL, timeout=2400)
        except subprocess.TimeoutExpired:
            pass
    lines = [json.loads(l) for l in log.read_text().splitlines() if l.startswith("{")]
    result = next((l for l in reversed(lines) if l.get("type") == "result"), {})
    return {"turns": result.get("num_turns", 0), "cost": result.get("total_cost_usd", 0),
            "seconds": round(time.time() - start)}


def judge(listed, diff):
    """The Opus judge's verdicts on `diff` against the rules `listed`."""
    prompt = JUDGE.format(rules="\n".join(listed), diff=diff[:150000])
    out = subprocess.run(["claude", "-p", "--model", MODELS["opus"], "--output-format", "json",
                          "--setting-sources", "project", "--strict-mcp-config", "--no-session-persistence"],
                         input=prompt, capture_output=True, text=True, timeout=1200).stdout
    text = json.loads(out).get("result", "") if out.startswith("{") else out
    found = re.search(r"\{.*\}", text, re.S)
    try:
        return json.loads(found.group(0))["rules"] if found else []
    except (json.JSONDecodeError, KeyError):
        return []


def task(arm, model, name, task, listed):
    """Does one task with a copy of the store, judges its diff, and appends its line to the results."""
    root = RUNS / name / task["id"]
    shutil.rmtree(root, ignore_errors=True)
    root.mkdir(parents=True)
    sh("git", "clone", "-q", "--no-hardlinks", str(MAIN), str(root / "work"), cwd=root)
    if arm in BINS:
        shutil.copytree(STORE, root / "home", ignore=shutil.ignore_patterns("given", "asked", "cache"))
    logs = RESULTS / "transcripts" / name
    logs.mkdir(parents=True, exist_ok=True)
    session = agent(arm, model, root / "work", root / "home", f"{task['prompt']}\n\n{DONE}", logs / f"{task['id']}.jsonl")
    sh("git", "add", "-A", cwd=root / "work")
    diff = sh("git", "diff", "--cached", "HEAD", cwd=root / "work")
    verdicts = judge(listed, diff) if diff else []
    broken = sum(v.get("verdict") == "broken" for v in verdicts)
    line = {"arm": arm, "model": model, "task": task["id"], "session": session, "diff": len(diff),
            "applied": len(verdicts), "broken": broken, "verdicts": verdicts}
    with (RESULTS / f"{name}.jsonl").open("a") as out:
        out.write(json.dumps(line) + "\n")
    print(f"{name} {task['id']}: {broken} broken of {len(verdicts)} that apply, diff {len(diff)}", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("arm", choices=["base", "new", "plain"])
    parser.add_argument("--model", choices=MODELS, default="opus")
    parser.add_argument("--jobs", type=int, default=5)
    parser.add_argument("--tasks", default="")
    args = parser.parse_args()
    tasks = json.loads((HERE / "tasks.json").read_text())
    if args.tasks:
        tasks = [t for t in tasks if t["id"] in args.tasks.split(",")]
    RESULTS.mkdir(exist_ok=True)
    listed = rules(STORE)
    name = f"{args.arm}-{args.model}"
    (RESULTS / f"{name}.jsonl").unlink(missing_ok=True)
    print(f"{len(listed)} rules for the judge", flush=True)
    with ThreadPoolExecutor(args.jobs) as pool:
        for future in [pool.submit(task, args.arm, args.model, name, t, listed) for t in tasks]:
            future.result()


if __name__ == "__main__":
    main()
