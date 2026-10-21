#!/usr/bin/env python3
"""Prints the drift bench by arm and rule count, and draws results/curve.svg.

  report.py [model]    haiku when none is named

A rule counts only where the task gave it something to act on. Rules that plain keeps on 70% or more of the
tasks they apply to are habits, not memory: the table leaves them out and names them.
"""

import json
import re
import sys
from collections import defaultdict
from math import log
from pathlib import Path

HERE = Path(__file__).resolve().parent
COLORS = {"oracle": "#e76f51", "celis": "#2a9d8f", "placed": "#264653", "jit": "#8338ec", "narrow": "#f4a261", "ranked": "#e9c46a", "scoped": "#6a4c93", "scopedrank": "#1982c4", "picked": "#ff595e", "pickers": "#8ac926", "pickers50": "#52b788", "merged": "#3a0ca3", "staged": "#f72585", "triggered": "#06d6a0", "pull": "#9d4edd", "taught": "#ef476f", "taught1": "#f78c6b", "delegated": "#118ab2", "coordinated": "#073b4c", "reminded": "#ffd166", "sized": "#ef8354", "tooled": "#2ec4b6", "full": "#9b5de5", "checked": "#00bbf9", "words": "#f15bb5", "quiet": "#fee440", "nodiff": "#00f5d4", "paths": "#9b2226"}
W, H, PAD = 440, 300, 50


def lines(name):
    path = HERE / "results" / f"{name}.jsonl"
    return [json.loads(l) for l in path.read_text().splitlines() if l] if path.exists() else []


def habits(plain):
    """The rules plain keeps on 70% or more of the tasks they apply to."""
    seen = defaultdict(list)
    for line in plain:
        for rule, state in line["states"].items():
            if state is not None:
                seen[rule].append(state == "kept")
    return {r for r, kept in seen.items() if sum(kept) / len(kept) >= 0.7}


def measures(runs, skip, replaced):
    """The share of rules broken, the share of replaced rules kept in their old version, input tokens and cost
    per task, and the share of tasks whose tests pass."""
    states = [(r, s) for l in runs for r, s in l["states"].items() if s is not None and r not in skip]
    old = [s for l in runs for r, s in l["states"].items() if r in replaced and s is not None]
    return {"broken": sum(s != "kept" for _, s in states) / max(1, len(states)),
            "old": sum(s == "old" for s in old) / max(1, len(old)),
            "kin": sum(l["session"]["input"] for l in runs) / len(runs) / 1000,
            "cost": sum(l["session"]["cost"] for l in runs) / len(runs),
            "tests": sum(l["tests"] for l in runs) / len(runs)}


def panel(series, key, title, left, form):
    xs = sorted({n for s in series.values() for n in s})
    peak = max([v[key] for s in series.values() for v in s.values()] + [1e-9])
    x = lambda n: left + PAD + (log(n) - log(xs[0])) / ((log(xs[-1]) - log(xs[0])) or 1) * (W - 2 * PAD)
    y = lambda v: H - PAD - v / peak * (H - 2 * PAD)
    parts = [f'<text x="{left + W / 2}" y="20" text-anchor="middle" font-weight="bold">{title}</text>',
             f'<line x1="{left + PAD}" y1="{H - PAD}" x2="{left + W - PAD}" y2="{H - PAD}" stroke="#999"/>',
             f'<line x1="{left + PAD}" y1="{PAD}" x2="{left + PAD}" y2="{H - PAD}" stroke="#999"/>',
             f'<text x="{left + PAD - 6}" y="{PAD + 4}" text-anchor="end">{form.format(peak)}</text>']
    parts += [f'<text x="{x(n)}" y="{H - PAD + 16}" text-anchor="middle">{n}</text>' for n in xs]
    for arm, s in sorted(series.items()):
        points = " ".join(f"{x(n):.1f},{y(v[key]):.1f}" for n, v in sorted(s.items()))
        parts.append(f'<polyline points="{points}" fill="none" stroke="{COLORS[arm]}" stroke-width="2"/>')
    return parts


def main():
    model = (sys.argv[1:] or ["haiku"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    plain = lines(f"plain-{model}")
    skip = habits(plain)
    print(f"habits left out ({len(skip)}): {', '.join(sorted(skip)) or '-'}")
    print("arm\trules\ttasks\tbroken\told\tk input\tcost\ttests")
    if plain:
        m = measures(plain, skip, replaced)
        print(f"plain\t-\t{len(plain)}\t{m['broken']:.0%}\t{m['old']:.0%}\t{m['kin']:.0f}\t${m['cost']:.2f}\t{m['tests']:.0%}")
    series = defaultdict(dict)
    for path in sorted((HERE / "results").glob(f"*-{model}-r*.jsonl"), key=lambda p: int(p.stem.split("-r")[-1])):
        arm, n = re.match(rf"(\w+)-{model}-r(\d+)$", path.stem).groups()
        runs = lines(path.stem)
        m = series[arm][int(n)] = measures(runs, skip, replaced)
        print(f"{arm}\t{n}\t{len(runs)}\t{m['broken']:.0%}\t{m['old']:.0%}\t{m['kin']:.0f}\t${m['cost']:.2f}\t{m['tests']:.0%}")
    if not series:
        return
    legend = [f'<text x="{PAD + 160 * i}" y="{H + 10}" fill="{COLORS[a]}">{"CLAUDE.md" if a == "oracle" else a}</text>'
              for i, a in enumerate(sorted(series))]
    svg = ([f'<svg xmlns="http://www.w3.org/2000/svg" width="{3 * W}" height="{H + 30}" font-family="sans-serif" '
            'font-size="12"><rect width="100%" height="100%" fill="white"/>']
           + panel(series, "broken", "Rules broken", 0, "{:.0%}")
           + panel(series, "old", "Replaced rules kept in the old version", W, "{:.0%}")
           + panel(series, "kin", "Input tokens a task (k)", 2 * W, "{:.0f}k") + legend + ["</svg>"])
    (HERE / "results" / f"curve-{model}.svg").write_text("\n".join(svg) + "\n")


if __name__ == "__main__":
    main()
