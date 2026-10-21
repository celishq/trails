#!/usr/bin/env python3
"""Draws results/claude-md-vs-celis-<model>.svg: the rules broken as the memory grows, a CLAUDE.md that holds
every rule against celis, with no memory as the line to beat.

  chart.py [model] [celis arm]    opus and final when none is named

Each point is the share of checked rules broken over every task of every run at that size; habits are left
out as report.py leaves them out.
"""

import json
import sys
from math import log
from pathlib import Path

import report

HERE = Path(__file__).resolve().parent
W, H, LEFT, RIGHT, TOP, BOTTOM = 640, 400, 70, 150, 50, 60
ARMS = {"oracle": ("CLAUDE.md", "#e76f51"), "celis": ("celis", "#2a9d8f")}


def points(arm, model, skip, replaced):
    """The share broken and the task count at each memory size of `arm`."""
    found = {}
    for path in (HERE / "results").glob(f"{arm}-{model}-r*.jsonl"):
        runs = report.lines(path.stem)
        found[int(path.stem.split("-r")[-1])] = (report.measures(runs, skip, replaced)["broken"], len(runs))
    return dict(sorted(found.items()))


def main():
    model = (sys.argv[1:] or ["opus"])[0]
    arm = (sys.argv[2:] or ["final"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    plain = report.lines(f"plain-{model}")
    skip = report.habits(plain)
    none = report.measures(plain, skip, replaced)["broken"]
    series = {"oracle": points("oracle", model, skip, replaced), "celis": points(arm, model, skip, replaced)}
    sizes = sorted({n for s in series.values() for n in s})
    x = lambda n: LEFT + (log(n) - log(sizes[0])) / (log(sizes[-1]) - log(sizes[0])) * (W - LEFT - RIGHT)
    y = lambda v: TOP + (1 - v) * (H - TOP - BOTTOM)
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" font-family="sans-serif" '
             f'font-size="12">', f'<rect width="{W}" height="{H}" fill="#fff"/>',
             f'<text x="{LEFT}" y="24" font-size="15" font-weight="bold">Rules broken as the memory grows '
             f'({model.capitalize()}, {len(sizes)} sizes, 8 tasks each)</text>']
    for v in (0, 0.25, 0.5, 0.75, 1):
        parts.append(f'<line x1="{LEFT}" y1="{y(v)}" x2="{W - RIGHT}" y2="{y(v)}" stroke="#eee"/>')
        parts.append(f'<text x="{LEFT - 8}" y="{y(v) + 4}" text-anchor="end" fill="#666">{v:.0%}</text>')
    for n in sizes:
        parts.append(f'<text x="{x(n)}" y="{H - BOTTOM + 18}" text-anchor="middle" fill="#666">{n}</text>')
    parts.append(f'<text x="{(LEFT + W - RIGHT) / 2}" y="{H - 16}" text-anchor="middle" fill="#666">'
                 f'rules in memory</text>')
    parts.append(f'<line x1="{LEFT}" y1="{y(none)}" x2="{W - RIGHT}" y2="{y(none)}" stroke="#999" '
                 f'stroke-dasharray="5 4"/>')
    parts.append(f'<text x="{W - RIGHT + 8}" y="{y(none) + 4}" fill="#666">no memory {none:.0%}</text>')
    for key, found in series.items():
        name, colour = ARMS[key]
        line = " ".join(f"{x(n):.1f},{y(v):.1f}" for n, (v, _) in found.items())
        parts.append(f'<polyline points="{line}" fill="none" stroke="{colour}" stroke-width="2.5"/>')
        for n, (v, tasks) in found.items():
            parts.append(f'<circle cx="{x(n):.1f}" cy="{y(v):.1f}" r="4" fill="{colour}"><title>{name}, {n} '
                         f'rules: {v:.0%} broken over {tasks} tasks</title></circle>')
        last_n, (last_v, _) = list(found.items())[-1]
        parts.append(f'<text x="{x(last_n) + 10}" y="{y(last_v) + 4}" fill="{colour}" font-weight="bold">'
                     f'{name} {last_v:.0%}</text>')
    parts.append("</svg>")
    out = HERE / "results" / f"claude-md-vs-celis-{model}.svg"
    out.write_text("\n".join(parts) + "\n")
    for key, found in series.items():
        print(ARMS[key][0], "  ".join(f"{n}: {v:.0%} ({t} tasks)" for n, (v, t) in found.items()))
    print(f"no memory: {none:.0%}\n{out}")


if __name__ == "__main__":
    main()
