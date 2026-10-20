"""Prints, for each rule count, how many of the checked `me` rules that apply to a task the past or pick agents
passed on to the agent, and how many of those the agent broke.

  recall.py <arm> [model]    haiku when no model is named
"""
import json, sys
ARM, MODEL = (sys.argv[1:] + ["haiku"])[:2]
from pathlib import Path

D = Path(__file__).resolve().parent
targets = json.loads((D / "data/targets.json").read_text())
scopes = json.loads((D / "data/scopes.json").read_text())
habit = {"absolute-imports", "broad-except", "builtin-generics", "command-names", "exit-codes", "flag-kebab",
         "module-docstring", "no-assert", "no-print", "own-errors", "readme-commands", "store-names"}
me = [t for t in targets if scopes.get(t["id"]) == "me" and t["id"] not in habit]
print("rules\tpicked\tbroken when picked\tbroken when not\tlines picked a task\tpast ran")
for n in (40, 100, 400, 1000, 1600):
    picked = [0, 0]; missed = [0, 0]; lines = 0; ran = 0; tasks = 0
    path = D / f"results/{ARM}-{MODEL}-r{n}.jsonl"
    if not path.exists():
        continue
    for row in map(json.loads, open(path)):
        tasks += 1
        agents, answer = set(), ""
        for l in open(D / f"results/transcripts/{ARM}-{MODEL}-r{n}/{row['task']}.jsonl"):
            d = json.loads(l)
            if d.get("parent_tool_use_id"):
                continue
            for c in (d.get("message") or {}).get("content") or []:
                if not isinstance(c, dict):
                    continue
                if c.get("type") == "tool_use" and c["name"] in ("Agent", "Task") and any(k in json.dumps(c["input"]) for k in ("past", "pick")):
                    agents.add(c["id"])
                if c.get("type") == "tool_result" and c.get("tool_use_id") in agents:
                    answer += json.dumps(c.get("content"))
        ran += bool(answer)
        lines += answer.count(" live {me} ")
        for t in me:
            s = row["states"].get(t["id"])
            if s is None:
                continue
            b = picked if json.dumps(t["text"])[1:-1][:60] in answer else missed
            b[0] += s != "kept"; b[1] += 1
    total = picked[1] + missed[1]
    print(f"{n}\t{picked[1]}/{total} ({picked[1] / max(1, total):.0%})\t{picked[0]}/{picked[1]}\t"
          f"{missed[0]}/{missed[1]}\t{lines / tasks:.0f}\t{ran}/{tasks}")
