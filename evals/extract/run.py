#!/usr/bin/env python3
"""The extraction bench: the celis:extraction agent learns again from past asks of celis-trails, on a copy of
the person's real store, and celis grades each pattern it writes by the reasons of `celis patterns --tidy`.

  run.py <arm> [--model sonnet|opus] [--jobs N] [--count N] [--asks id,id]

  arm    base (the installed celis and the plugin of the main checkout), new (the celis in $CELIS_BIN and the
         plugin in $CELIS_PLUGIN, else this worktree's)

Both arms take the same asks: the newest celis-trails trails with a `changed` line, else those that --asks names.
The grade always uses the new celis, so both arms are judged by the same signs. Each ask starts from a new copy of
the store, so the real store is never written. Results, with the patterns each run wrote, go to
results/<arm>-<model>.jsonl, which git ignores.
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
RUNS = Path(os.environ.get("EXTRACT_RUNS", "/tmp/celis-extract"))
RESULTS = HERE / "results"
MODELS = {"sonnet": "claude-sonnet-5-5", "opus": "claude-opus-5-5"}
AGENT = ["--output-format", "stream-json", "--verbose", "--allowedTools", "Bash(celis *)", "Read", "Grep", "Glob",
         "--setting-sources", "project", "--strict-mcp-config", "--max-budget-usd", "2", "--no-session-persistence"]
BINS = {"base": Path(shutil.which("celis") or "/usr/local/bin/celis").parent,
        "new": Path(os.environ.get("CELIS_BIN", ROOT / "crates" / "target" / "debug"))}
PLUGINS = {"base": MAIN / "plugins" / "celis", "new": Path(os.environ.get("CELIS_PLUGIN", ROOT / "plugins" / "celis"))}
WEAK = ["nameless", "form", "topic", "tension", "fact", "many"]
SKIP = {"e31b9a52", "6a9d3f68", "1d796c62"}


def celis(arm, home, *args, cwd):
    """What the celis of `arm` prints for `args`, on the store at `home`."""
    env = dict(os.environ, CELIS_HOME=str(home), PATH=f"{BINS[arm]}:{os.environ['PATH']}")
    env.pop("CLAUDE_CODE_SESSION_ID", None)
    return subprocess.run(["celis", *args], cwd=cwd, env=env, capture_output=True, text=True).stdout


def asks(count):
    """The ids of the newest celis-trails trails with a `changed` line."""
    text = celis("new", STORE, "grep", " changed ", cwd=MAIN)
    heads = re.findall(r"^([0-9a-f]{8}) \[celis-trails\] .* · (\d{4}-\d{2}-\d{2})$", text, re.M)
    heads = [h for h in heads if h[0] not in SKIP]
    return [i for i, _ in sorted(heads, key=lambda h: h[1], reverse=True)[:count]]


def live(home, work):
    """The ids of the live patterns in scope in `work`."""
    return set(re.findall(r"^([0-9a-f]{8}) ", celis("new", home, "patterns", cwd=work), re.M))


def grade(home, work, before):
    """Each pattern written since `before`: its id, its weak reasons and its text."""
    written = live(home, work) - before
    tidy = celis("new", home, "patterns", "--tidy", cwd=work)
    reasons = {m[0]: m[1].split(",") for m in re.findall(r"^([0-9a-f]{8}) (\S+) ", tidy, re.M)}
    page = celis("new", home, "patterns", "--markdown", cwd=work)
    texts = dict(re.findall(r"^## .*?\n\n`([0-9a-f]{8})` .*?\n\n(.*?)(?=\n## |\Z)", page, re.M | re.S))
    return [{"id": i, "weak": [r for r in reasons.get(i, []) if r in WEAK], "text": texts.get(i, "").strip()}
            for i in sorted(written)]


def run(arm, model, name, ask):
    """Learns again from `ask` with a copy of the store, and appends its line to the results."""
    root = RUNS / name / ask
    shutil.rmtree(root, ignore_errors=True)
    root.mkdir(parents=True)
    work, home = root / "work", root / "home"
    subprocess.run(["git", "clone", "-q", "--shared", str(MAIN), str(work)], check=True)
    shutil.copytree(STORE, home, ignore=shutil.ignore_patterns("given", "asked", "cache"))
    before = live(home, work)
    brief = celis(arm, home, "extract", ask, cwd=work)
    env = dict(os.environ, CELIS_HOME=str(home), PATH=f"{BINS[arm]}:{os.environ['PATH']}")
    env.pop("CLAUDE_CODE_SESSION_ID", None)
    flags = ["--settings", json.dumps({"autoMemoryEnabled": False}), "--plugin-dir", str(PLUGINS[arm]),
             "--agent", "celis:extraction"]
    logs = RESULTS / "transcripts" / name
    logs.mkdir(parents=True, exist_ok=True)
    log, start = logs / f"{ask}.jsonl", time.time()
    with log.open("w") as out:
        try:
            subprocess.run(["claude", "-p", brief, "--model", MODELS[model], *AGENT, *flags], cwd=work, env=env,
                           stdout=out, stderr=subprocess.DEVNULL, timeout=1200)
        except subprocess.TimeoutExpired:
            pass
    lines = [json.loads(l) for l in log.read_text().splitlines() if l.startswith("{")]
    result = next((l for l in reversed(lines) if l.get("type") == "result"), {})
    written = grade(home, work, before)
    line = {"arm": arm, "model": model, "ask": ask, "cost": result.get("total_cost_usd", 0),
            "seconds": round(time.time() - start), "answer": result.get("result", ""), "written": written}
    with (RESULTS / f"{name}.jsonl").open("a") as out:
        out.write(json.dumps(line) + "\n")
    clean = sum(not w["weak"] for w in written)
    print(f"{name} {ask}: {len(written)} written, {clean} with no weak reason", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("arm", choices=list(BINS))
    parser.add_argument("--model", choices=MODELS, default="sonnet")
    parser.add_argument("--jobs", type=int, default=4)
    parser.add_argument("--count", type=int, default=10)
    parser.add_argument("--asks", default="")
    args = parser.parse_args()
    chosen = args.asks.split(",") if args.asks else asks(args.count)
    RESULTS.mkdir(exist_ok=True)
    name = f"{args.arm}-{args.model}"
    (RESULTS / f"{name}.jsonl").unlink(missing_ok=True)
    print(f"{len(chosen)} asks: {','.join(chosen)}", flush=True)
    with ThreadPoolExecutor(args.jobs) as pool:
        for future in [pool.submit(run, args.arm, args.model, name, a) for a in chosen]:
            future.result()


if __name__ == "__main__":
    main()
