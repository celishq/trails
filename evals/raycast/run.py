#!/usr/bin/env python3
"""The Raycast bench: an agent does ten extension tasks, each in its own folder; reviews after the first two give
rules, and the bench counts how often later tasks break them.

  run.py <arm> [runs] [--jobs N] [--tasks pr,pr] [--model sonnet|haiku]    5 runs, 1 at a time, every task, sonnet

  arm    plain (no memory), auto (Claude Code auto memory, one folder per extension), self (a user CLAUDE.md that
         the agent keeps), oracle (a user CLAUDE.md the bench writes each review rule into), celis (the plugin
         from $CELIS_PLUGIN, the celis in $CELIS_BIN), gold (no agent: the merged code)

Each task is a new git repo that holds only its extension at the PR's base, so only the arm's memory carries over.
Runs live in $RAYCAST_RUNS (default /tmp/celis-raycast); each run appends one line a task to results/<arm>.jsonl, or
results/<arm>-haiku.jsonl for haiku.
"""

import argparse
import json
import os
import shutil
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import check

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CLONE = HERE / "cache" / "extensions"
RUNS = Path(os.environ.get("RAYCAST_RUNS", "/tmp/celis-raycast"))
RESULTS = HERE / "results"
MODELS = {"sonnet": "claude-sonnet-5-5", "haiku": "claude-haiku-5-5"}
AGENT = ["--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions",
         "--setting-sources", "project", "--strict-mcp-config", "--max-budget-usd", "5", "--no-session-persistence"]
DONE = "Go on to the end without asking."
FOLLOW = 3
OFFLINE = dict(os.environ, GIT_NO_LAZY_FETCH="1")
ARMS = ("plain", "auto", "self", "oracle", "celis")
SELF = ("# My rules\n\nThis file is my memory across all my projects. When someone reviews or corrects my work, "
        "I add each point here as a short rule, and I follow these rules in every task.\n")


def sh(*args, cwd, stdin=None):
    return subprocess.run(args, cwd=cwd, input=stdin, check=True, capture_output=True)


def workdir(task, path, commit):
    """A new git repo at path that holds the task's extension as it is at commit, committed as one base commit."""
    shutil.rmtree(path, ignore_errors=True)
    path.mkdir(parents=True)
    sh("git", "init", "-q", cwd=path)
    tree = subprocess.run(["git", "archive", f"{commit}:extensions/{task['extension']}"], cwd=CLONE,
                          env=OFFLINE, capture_output=True).stdout
    if tree:
        sh("tar", "-x", cwd=path, stdin=tree)
    (path / ".gitignore").open("a").write("\nnode_modules/\ndist/\n")
    sh("git", "add", "-A", cwd=path)
    sh("git", "-c", "user.name=bench", "-c", "user.email=bench@local", "commit", "-q", "-m", "base", cwd=path)
    if (path / "package.json").exists():
        subprocess.run(["npm", "install", "--no-audit", "--no-fund"], cwd=path, capture_output=True)
    return sh("git", "rev-parse", "HEAD", cwd=path).stdout.decode().strip()


def brief(task):
    if task["new"]:
        return f"Build a new Raycast extension named `{task['extension']}` in this folder.\n\n{task['prompt']}\n\n{DONE}"
    return f"This folder is the `{task['extension']}` Raycast extension. Make this change:\n\n{task['prompt']}\n\n{DONE}"


def review(task, rules):
    said = " ".join(r["review"] for r in rules)
    return f"Review feedback on the `{task['extension']}` extension: {said} Fix it. {DONE}"


def agent(arm, model, work, home, extension, prompt, log):
    """One Claude Code session in the work folder; returns its numbers from the stream-json transcript."""
    env = dict(os.environ, CELIS_HOME=str(home / "celis"))
    memory = {"autoMemoryEnabled": arm == "auto", "autoMemoryDirectory": str(home / "memory" / extension)}
    flags = ["--settings", json.dumps(memory)]
    if arm in ("self", "oracle"):
        env["CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD"] = "1"
        flags += ["--add-dir", str(home / "user")]
    if arm == "celis":
        env["PATH"] = f"{os.environ.get('CELIS_BIN', ROOT / 'crates' / 'target' / 'release')}:{env['PATH']}"
        flags += ["--plugin-dir", os.environ.get("CELIS_PLUGIN", str(ROOT / "plugins" / "celis"))]
    start = time.time()
    with log.open("w") as out:
        try:
            subprocess.run(["claude", "-p", prompt, "--model", MODELS[model], *AGENT, *flags], cwd=work, env=env, stdout=out,
                           stderr=subprocess.DEVNULL, timeout=2400)
        except subprocess.TimeoutExpired:
            pass
    lines = [json.loads(l) for l in log.read_text().splitlines() if l.startswith("{")]
    result = next((l for l in reversed(lines) if l.get("type") == "result"), {})
    usage = result.get("usage", {})
    return {"turns": result.get("num_turns", 0), "cost": result.get("total_cost_usd", 0),
            "input": sum(usage.get(k, 0) for k in ("input_tokens", "cache_read_input_tokens",
                                                   "cache_creation_input_tokens")),
            "output": usage.get("output_tokens", 0), "seconds": round(time.time() - start)}


def gold(task, work):
    """Puts the merged extension over the base, keeping the repo and its installed packages."""
    for item in work.iterdir():
        if item.name not in (".git", "node_modules", ".gitignore"):
            shutil.rmtree(item) if item.is_dir() else item.unlink()
    tree = subprocess.run(["git", "archive", f"{task['merge']}:extensions/{task['extension']}"], cwd=CLONE,
                          env=OFFLINE, check=True, capture_output=True).stdout
    sh("tar", "-x", cwd=work, stdin=tree)
    subprocess.run(["npm", "install", "--no-audit", "--no-fund"], cwd=work, capture_output=True)


def keep(work, label):
    """Commits the work as it stands, so the run can be scored again later."""
    sh("git", "add", "-A", cwd=work)
    sh("git", "-c", "user.name=bench", "-c", "user.email=bench@local", "commit", "-q", "--allow-empty", "-m", label,
       cwd=work)


def chain(arm, model, run, tasks, rules):
    name = arm if model == "sonnet" else f"{arm}-{model}"
    root = RUNS / f"{name}-{run}"
    logs = RESULTS / "transcripts" / f"{name}-{run}"
    shutil.rmtree(root, ignore_errors=True)
    (root / "home" / "user").mkdir(parents=True)
    logs.mkdir(parents=True, exist_ok=True)
    if arm in ("self", "oracle"):
        (root / "home" / "user" / "CLAUDE.md").write_text(SELF if arm == "self" else "# My rules\n\n")
    order = [t["pr"] for t in tasks]
    for position, task in enumerate(tasks, 1):
        work = root / "work" / task["extension"]
        base = workdir(task, work, task["base"])
        line = {"arm": arm, "model": model, "run": run, "position": position, "pr": task["pr"],
                "rules": [r["id"] for r in rules if order.index(r["from"]) < position - 1]}
        label = f"{position}-{task['pr']}"
        if arm == "gold":
            gold(task, work)
        else:
            line["session"] = agent(arm, model, work, root / "home", task["extension"], brief(task),
                                    logs / f"{label}.jsonl")
            given = [r for r in rules if r["from"] == task["pr"]]
            if given:
                line["review"] = agent(arm, model, work, root / "home", task["extension"], review(task, given),
                                       logs / f"{label}-review.jsonl")
                if arm == "oracle":
                    with (root / "home" / "user" / "CLAUDE.md").open("a") as memory:
                        memory.writelines(f"- {r['rule']}\n" for r in given)
        line.update(check.check(work, base))
        keep(work, "check")
        line["follow"] = []
        state = line
        while arm != "gold" and len(line["follow"]) < FOLLOW and (wrong := broken(line, state)):
            n = len(line["follow"]) + 1
            session = agent(arm, model, work, root / "home", task["extension"],
                            review(task, [r for r in rules if r["id"] in wrong]), logs / f"{label}-follow{n}.jsonl")
            state = check.check(work, base)
            keep(work, f"follow-up {n}")
            line["follow"].append({"session": session, "wrong": wrong, **state})
        with (RESULTS / f"{name}.jsonl").open("a") as out:
            out.write(json.dumps(line) + "\n")
        print(f"{name} run {run} task {position} #{task['pr']}: broken {broken(line, line) or '-'}, "
              f"build {'ok' if line['build']['ok'] else 'failed'}, follow-ups {len(line['follow'])}", flush=True)


def broken(line, state):
    """The rules given before the task that the work in `state` breaks."""
    return [r for r in line["rules"] if r in state["broken"]]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("arm", choices=[*ARMS, "gold"])
    parser.add_argument("runs", nargs="?", type=int, default=5)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--tasks", default="")
    parser.add_argument("--model", choices=MODELS, default="sonnet")
    args = parser.parse_args()
    tasks = json.loads((HERE / "data" / "tasks.json").read_text())
    if args.tasks:
        chosen = {int(n) for n in args.tasks.split(",")}
        tasks = [t for t in tasks if t["pr"] in chosen]
    rules = [r for r in json.loads((HERE / "data" / "rules.json").read_text()) if r["from"] in [t["pr"] for t in tasks]]
    RESULTS.mkdir(exist_ok=True)
    with ThreadPoolExecutor(args.jobs) as pool:
        for future in [pool.submit(chain, args.arm, args.model, run, tasks, rules) for run in range(1, args.runs + 1)]:
            future.result()


if __name__ == "__main__":
    main()
