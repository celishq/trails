#!/usr/bin/env python3
"""The drift bench: an agent does small tasks on the `stock` repo while its memory holds N rules, 34 of them
checked; 4 of those replaced an older version that the memory may still hold.

  run.py <arm> <rules...> [--jobs N] [--model haiku|sonnet|opus] [--tasks id,id]

  arm    plain (no memory), oracle (a user CLAUDE.md of the rules and real prose, the replaced versions
         kept above their new ones), celis (the rules as `me` rules, each replaced version replaced with
         --replaces; the plugin from $CELIS_PLUGIN, the celis in $CELIS_BIN), placed (celis, with each
         target rule at its place in the project and the filler as project rules), jit (placed, with the
         filler spread over the places of seed/ and as many places it lacks, and jit.py as the hooks that
         give the rules of a file when the agent reads it), narrow (jit, each target at its NARROW place),
         ranked (jit, the rules that share the most words with the file first), scoped and scopedrank (jit
         and ranked, each rule in the scope of data/scopes.json, `other` rules in another project), picked
         (scoped, with the hook giving only place rules and data/past.md as the past agent, which reads the
         `me` rules in pages and picks those for the ask), pickers and pickers50 (picked, with a
         data/pick.md agent for each page of 100 or 50 `me` rules, run in parallel, and data/past-pick.md
         as the past agent), merged (pickers, with jit.py running the pickers and a merge to 20 rules
         from the prompt hook, so the agent gets only those), staged (merged with no merge: the hook keeps the
         whole pick, and each read gives the file the picked rules that act on it), pull (celis, with no plugin
         and no rules given: the agent searches them with celis grep), taught (celis, each rule with its trigger
         of data/triggers.json, with no plugin, no tool hook and no picker: the session start gives the card,
         and the agent asks celis rules and celis grep itself), delegated (taught, with the plugin of the
         celis arm: the card, the prompt hook, and the celis:past agent, which runs celis rules for the work),
         coordinated (delegated, with the card of the agent that searches the rules itself: the index of
         celis rules, celis grep, and celis rules before each change; the past agent brings past asks),
         reminded (coordinated, with step add and step start saying what to do at that moment), sized
         (reminded, with no past agent: a file not made yet gets only its path rules and a count of the
         rest, and step start names celis rules --diff for the check before done), tooled (reminded,
         with celis rules --diff named for the check before done, and long rule output that says its
         size and reads in pages with --page)

Each task starts from a new copy of seed/ and a new copy of the memory, so tasks are independent and run in
parallel. Each task appends one line to results/<arm>-<model>-r<N>.jsonl.
"""

import argparse
import json
import os
import random
import shutil
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import check

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RUNS = Path(os.environ.get("DRIFT_RUNS", "/tmp/celis-drift"))
RESULTS = HERE / "results"
MODELS = {"haiku": "claude-haiku-5-5", "sonnet": "claude-sonnet-5-5", "opus": "claude-opus-5-5"}
AGENT = ["--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions", "--setting-sources",
         "project", "--strict-mcp-config", "--max-budget-usd", "3", "--no-session-persistence"]
CELIS = dict(os.environ, PATH=f"{os.environ.get('CELIS_BIN', ROOT / 'crates' / 'target' / 'release')}:{os.environ['PATH']}")
CHUNK = 12
FAKE = ["web", "docs", "infra", "mobile", "scripts", "migrations", "deploy", "api", "ui", "bench", "ops",
        "admin", "billing", "auth", "search", "mail", "jobs", "assets", "i18n", "sdk", "proto", "charts"]
NARROW = {"stock": "stock/*.py", "stock/commands": "stock/commands/*.py", "stock/storage": "stock/storage/*.py",
          "tests": "tests/*.py"}
PICKS = ("picked", "pickers", "pickers50", "merged", "staged")
JITS = ("jit", "narrow", "ranked", "scoped", "scopedrank", *PICKS)
OWN = ("pull",)
TRIGGERED = ("taught", "delegated", "coordinated", "reminded", "sized", "tooled")
STORED = ("celis", "placed", *JITS, *OWN, *TRIGGERED)
FIXED = {"GIT_AUTHOR_DATE": "2026-01-01T00:00:00Z", "GIT_COMMITTER_DATE": "2026-01-01T00:00:00Z"}


def sh(*args, cwd, env=None):
    return subprocess.run(args, cwd=cwd, env=env, check=True, capture_output=True, text=True).stdout.strip()


def memory(n, targets, filler):
    """The rule lines and prose blocks of a memory of `n` rules, in file order: each replaced version in the
    first half, its new version in the second."""
    olds = [t for t in targets if "old" in t]
    rules = [t["text"] for t in targets if "old" not in t] + [r["text"] for r in filler["rules"][:n - len(targets) - len(olds)]]
    random.Random(n).shuffle(rules)
    half = len(rules) // 2
    pick = random.Random(n + 1)
    for t in olds:
        rules.insert(pick.randint(0, half), t["old"])
    for t in olds:
        rules.insert(pick.randint(len(rules) // 2 + 1, len(rules)), t["text"])
    share = (n - len(targets) - len(olds)) / max(1, len(filler["rules"]))
    prose = filler["prose"][:round(len(filler["prose"]) * share)]
    return rules, prose


def claude_md(rules, prose):
    """A user CLAUDE.md: rule sections of CHUNK bullets with the prose blocks spread between them."""
    chunks = [rules[i:i + CHUNK] for i in range(0, len(rules), CHUNK)]
    every = max(1, len(chunks) // max(1, len(prose))) if prose else 0
    parts, blocks = ["# Engineering guide for stock\n"], list(prose)
    for i, chunk in enumerate(chunks):
        parts.append(f"## Rules, part {i + 1}\n\n" + "".join(f"- {r}\n" for r in chunk))
        if blocks and every and i % every == every - 1:
            b = blocks.pop(0)
            parts.append(f"## {b['heading']}\n\n{b['text']}\n")
    parts += [f"## {b['heading']}\n\n{b['text']}\n" for b in blocks]
    return "\n".join(parts)


def seed(arm, home, rules, targets):
    """Puts the memory in `home` once; tasks then start from copies of it."""
    (home / "user").mkdir(parents=True)
    if arm == "oracle":
        (home / "user" / "CLAUDE.md").write_text(claude_md(*rules))
    if arm in STORED:
        place = home / "seed"
        workdir(place)
        env = dict(CELIS, CELIS_HOME=str(home / "celis"))
        ids, spread = {}, random.Random(len(rules[0]))
        scopes = json.loads((HERE / "data" / "scopes.json").read_text())
        by_text = {r["text"]: r["id"] for r in json.loads((HERE / "data" / "filler.json").read_text())["rules"]}
        triggers = json.loads((HERE / "data" / "triggers.json").read_text()) if arm in TRIGGERED else {}
        other = home / "other"
        other.mkdir()
        sh("git", "init", "-q", cwd=other)
        sh("git", "-c", "user.name=bench", "-c", "user.email=bench@local", "commit", "-q", "--allow-empty", "-m",
           "other", cwd=other, env=dict(os.environ, **FIXED))
        places = sorted(str(p.relative_to(HERE / "seed")) for p in (HERE / "seed").rglob("*")
                        if "__pycache__" not in p.parts) + FAKE
        for line in rules[0]:
            old = next((t for t in targets if t.get("old") == line), None)
            new = next((t for t in targets if "old" in t and t["text"] == line), None)
            target = old or new or next((t for t in targets if t["text"] == line), None)
            at = NARROW.get(target["place"], target["place"]) if target and arm == "narrow" else target and target["place"]
            scope = ("me" if arm in ("celis", *OWN) else f"at:{at}" if target else "project" if arm == "placed"
                     else f"at:{spread.choice(places)}")
            if arm.startswith("scoped") or arm in PICKS:
                got = scopes.get(target["id"] if target else by_text[line], "other")
                scope = "me" if got == "me" else "project" if got == "other" else f"at:{got}"
            extra = ["--replaces", ids[new["id"]]] if new and new["id"] in ids else []
            if arm in TRIGGERED:
                scope = "me"
            if not old:
                extra += trigger_args(triggers.get(target["id"] if target else by_text[line]))
            where = other if scope == "project" and arm != "placed" else place
            out = sh("celis", "decide", "--scope", scope, *extra, "--", line, cwd=where, env=env)
            if old:
                ids[old["id"]] = out.split()[-1][:8]


def trigger_args(trigger):
    """The `celis decide` flags of `trigger`, a trigger of data/triggers.json or None."""
    if not trigger:
        return []
    return ([f for path in trigger["paths"] for f in ("--path", path)]
            + (["--content", trigger["content"]] if trigger["content"] else [])
            + (["--command", trigger["command"]] if trigger["command"] else [])
            + (["--situation"] if trigger["situation"] else []))


def workdir(path):
    """A new git repo at `path` holding seed/ as one base commit, the same commit in every copy, so celis
    counts every copy as one project; returns the commit."""
    shutil.copytree(HERE / "seed", path)
    sh("git", "init", "-q", cwd=path)
    sh("git", "add", "-A", cwd=path)
    sh("git", "-c", "user.name=bench", "-c", "user.email=bench@local", "commit", "-q", "-m", "base", cwd=path,
       env=dict(os.environ, **FIXED))
    return sh("git", "rev-parse", "HEAD", cwd=path)




def plugin(home, arm):
    """A copy of the plugin in `home` with data/past.md as its past agent, or for the pickers arms
    data/past-pick.md and the data/pick.md agent."""
    copy = home / "plugin"
    if not copy.exists():
        shutil.copytree(os.environ.get("CELIS_PLUGIN", ROOT / "plugins" / "celis"), copy)
        picks = arm.startswith("pickers") or arm in ("merged", "staged")
        (copy / "agents").mkdir(exist_ok=True)
        shutil.copy(HERE / "data" / ("past-pick.md" if picks else "past.md"), copy / "agents" / "past.md")
        if arm.startswith("pickers"):
            shutil.copy(HERE / "data" / "pick.md", copy / "agents" / "pick.md")
    return copy


def agent(arm, model, work, home, prompt, log):
    """One Claude Code session in `work`; its numbers from the stream-json transcript."""
    env = dict(CELIS if arm in STORED else os.environ, CELIS_HOME=str(home / "celis"))
    settings = {"autoMemoryEnabled": False}
    if arm in ("ranked", "scopedrank"):
        env["JIT_RANK"] = "words"
    if arm in PICKS:
        env["JIT_ME"] = "off"
        env["JIT_PAGE"] = "50" if arm == "pickers50" else "100"
    if arm in JITS:
        hook = lambda mode: [{"type": "command", "command": f"python3 {HERE / 'jit.py'} {mode}"}]
        settings["hooks"] = {"PostToolUse": [{"matcher": "Read|Write|Edit|MultiEdit", "hooks": hook("read")},
                                             {"matcher": "Bash", "hooks": hook("done")}]}
        if arm.startswith("pickers"):
            settings["hooks"]["UserPromptSubmit"] = [{"hooks": hook("ask")}]
        if arm in ("merged", "staged"):
            settings["hooks"]["UserPromptSubmit"] = [{"hooks": [dict(hook("pick")[0], timeout=600)]}]
        if arm == "staged":
            env["JIT_STAGE"] = "file"
            settings["hooks"]["PostToolUse"][0]["matcher"] = "Read|Write"
            settings["hooks"]["PostToolUse"][0]["hooks"] = [dict(hook("read")[0], timeout=120)]
    if arm in OWN:
        hook = lambda mode, timeout=60: [{"type": "command", "command": f"python3 {HERE / 'trigger.py'} {mode}",
                                          "timeout": timeout}]
        settings["hooks"] = {"UserPromptSubmit": [{"hooks": hook("pull", 600)}]}
    if arm == "taught":
        script = Path(os.environ.get("CELIS_PLUGIN", ROOT / "plugins" / "celis")) / "hooks" / "start.sh"
        settings["hooks"] = {"SessionStart": [{"hooks": [{"type": "command", "command": f"sh {script}"}]}]}
    flags = ["--settings", json.dumps(settings)]
    if arm == "oracle":
        env["CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD"] = "1"
        flags += ["--add-dir", str(home / "user")]
    if arm in ("celis", "placed", *JITS, "delegated", "coordinated", "reminded", "sized", "tooled"):
        flags += ["--plugin-dir", str(plugin(home, arm)) if arm in PICKS
                  else os.environ.get("CELIS_PLUGIN", str(ROOT / "plugins" / "celis"))]
    start = time.time()
    with log.open("w") as out:
        try:
            subprocess.run(["claude", "-p", prompt, "--model", MODELS[model], *AGENT, *flags], cwd=work, env=env,
                           stdout=out, stderr=subprocess.DEVNULL, timeout=1200)
        except subprocess.TimeoutExpired:
            pass
    lines = [json.loads(l) for l in log.read_text().splitlines() if l.startswith("{")]
    result = next((l for l in reversed(lines) if l.get("type") == "result"), {})
    usage = result.get("usage", {})
    return {"turns": result.get("num_turns", 0), "cost": result.get("total_cost_usd", 0),
            "input": sum(usage.get(k, 0) for k in ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens")),
            "output": usage.get("output_tokens", 0), "seconds": round(time.time() - start)}


def task(arm, model, n, name, home, task, targets):
    """Does one task with a copy of the memory, checks it, and appends its line to the results."""
    root = RUNS / name / task["id"]
    shutil.rmtree(root, ignore_errors=True)
    shutil.copytree(home, root / "home")
    base = workdir(root / "work")
    logs = RESULTS / "transcripts" / name
    logs.mkdir(parents=True, exist_ok=True)
    session = agent(arm, model, root / "work", root / "home", task["prompt"], logs / f"{task['id']}.jsonl")
    line = {"arm": arm, "model": model, "rules": n, "task": task["id"], "session": session,
            **check.check(root / "work", base, targets)}
    sh("git", "add", "-A", cwd=root / "work")
    sh("git", "-c", "user.name=bench", "-c", "user.email=bench@local", "commit", "-q", "--allow-empty", "-m", "work",
       cwd=root / "work")
    with (RESULTS / f"{name}.jsonl").open("a") as out:
        out.write(json.dumps(line) + "\n")
    states = list(line["states"].values())
    print(f"{name} {task['id']}: broken {states.count('broken')}, old {states.count('old')}, "
          f"kept {states.count('kept')}, tests {'ok' if line['tests'] else 'failed'}", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("arm", choices=["plain", "oracle", "celis", "placed", *JITS, *OWN, *TRIGGERED])
    parser.add_argument("rules", nargs="+", type=int)
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--model", choices=MODELS, default="haiku")
    parser.add_argument("--tasks", default="")
    args = parser.parse_args()
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    filler = json.loads((HERE / "data" / "filler.json").read_text()) if args.arm != "plain" else {}
    tasks = json.loads((HERE / "data" / "tasks.json").read_text())
    if args.tasks:
        tasks = [t for t in tasks if t["id"] in args.tasks.split(",")]
    RESULTS.mkdir(exist_ok=True)
    jobs = []
    for n in args.rules:
        name = f"{args.arm}-{args.model}-r{n}" if args.arm != "plain" else f"plain-{args.model}"
        home = RUNS / name / "home"
        shutil.rmtree(RUNS / name, ignore_errors=True)
        seed(args.arm, home, memory(n, targets, filler) if filler else ([], []), targets)
        (RESULTS / f"{name}.jsonl").unlink(missing_ok=True)
        jobs += [(args.arm, args.model, n, name, home, t, targets) for t in tasks]
    with ThreadPoolExecutor(args.jobs) as pool:
        for future in [pool.submit(task, *job) for job in jobs]:
            future.result()


if __name__ == "__main__":
    main()
