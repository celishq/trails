#!/usr/bin/env python3
"""Prints the Raycast bench per arm: review rules broken on later tasks, follow-up rounds, builds and cost.

  report.py [arm...]    every arm in results/ when none is named

A rule counts on a task only when its review came before that task. Each run gives one share; the table shows the
mean and the range over runs.
"""

import json
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent


def measures(line):
    """The task's rule-tasks given and broken, its follow-ups, and what it cost to the end."""
    sessions = [line[k] for k in ("session", "review") if k in line] + [f["session"] for f in line["follow"]]
    final = line["follow"][-1] if line["follow"] else line
    return {"given": len(line["rules"]), "broken": sum(r in line["broken"] for r in line["rules"]),
            "rounds": len(line["follow"]), "left": sum(r in final["broken"] for r in line["rules"]),
            "build": final["build"]["ok"], "cost": sum(s["cost"] for s in sessions),
            "kin": sum(s["input"] for s in sessions) / 1000}


def spread(values, form):
    mean = sum(values) / len(values)
    return form.format(mean) + (f" ({form.format(min(values))}–{form.format(max(values))})" if len(values) > 1 else "")


def main():
    arms = sys.argv[1:] or sorted(p.stem for p in (HERE / "results").glob("*.jsonl"))
    print("arm\truns\tbroken\trounds\tleft\tbuilds\tcost $\tk input")
    for arm in arms:
        lines = [json.loads(l) for l in (HERE / "results" / f"{arm}.jsonl").read_text().splitlines() if l]
        runs = defaultdict(list)
        for line in lines:
            runs[line["run"]].append(measures(line))
        per = [{k: sum(m[k] for m in ms) for k in ms[0]} for ms in runs.values()]
        print(f"{arm}\t{len(per)}\t"
              + spread([p["broken"] / p["given"] for p in per], "{:.0%}") + f" of {per[0]['given']}\t"
              + "\t".join(spread([p[k] for p in per], "{:.1f}") for k in ("rounds", "left", "build", "cost", "kin")))
    print()
    for arm in arms:
        lines = [json.loads(l) for l in (HERE / "results" / f"{arm}.jsonl").read_text().splitlines() if l]
        by = defaultdict(lambda: [0, 0])
        for line in lines:
            for rule in line["rules"]:
                by[rule][0] += rule in line["broken"]
                by[rule][1] += 1
        print(f"{arm}\t" + "  ".join(f"{r} {n}/{t}" for r, (n, t) in by.items()))


if __name__ == "__main__":
    main()
