#!/usr/bin/env python3
"""Draws media/claude-md-vs-celis.svg: as the memory grows, the rules broken and the outdated rules followed, a
CLAUDE.md that holds every rule against celis, with the gap between them marked.

  chart.py [model] [celis arm]    opus and final when none is named

Each point pools every task of every run at that size; habits are left out as report.py leaves them out. The
ticks under each panel mark the rules in the root agent file of some well-known repos.
"""

import json
import sys
from math import log
from pathlib import Path

import report

HERE = Path(__file__).resolve().parent
OUT = HERE.parents[1] / "media" / "claude-md-vs-celis.svg"
PANEL, H, GAP, LEFT, TOP, BOTTOM = 420, 440, 60, 56, 112, 92
MD, CELIS, BETTER, WORSE, INK, SOFT = "#e76f51", "#2a9d8f", "#2a9d8f", "#e76f51", "#222", "#888"
REPOS = {"vscode": 79, "next.js": 123, "PostHog": 160, "Kiali": 229, "kube-router": 282}


def points(arm, model, key, skip, replaced):
    """The share `key` of `arm` at each memory size."""
    found = {}
    for path in (HERE / "results").glob(f"{arm}-{model}-r*.jsonl"):
        found[int(path.stem.split("-r")[-1])] = report.measures(report.lines(path.stem), skip, replaced)[key]
    return dict(sorted(found.items()))


def panel(left, title, md, celis, none):
    """One panel: CLAUDE.md and celis by memory size, the gap shaded and labelled in points."""
    sizes = list(md)
    right = left + PANEL
    x = lambda n: left + LEFT + (log(n) - log(sizes[0])) / (log(sizes[-1]) - log(sizes[0])) * (PANEL - LEFT - 20)
    y = lambda v: TOP + (1 - v) * (H - TOP - BOTTOM)
    parts = [f'<text x="{left + LEFT}" y="{TOP - 30}" font-size="17" font-weight="700" fill="{INK}">{title}</text>']
    for v in (0, 0.25, 0.5, 0.75, 1):
        parts.append(f'<line x1="{left + LEFT}" y1="{y(v)}" x2="{right - 20}" y2="{y(v)}" stroke="#eee"/>')
        parts.append(f'<text x="{left + LEFT - 8}" y="{y(v) + 4}" text-anchor="end" fill="{SOFT}">{v:.0%}</text>')
    if none is not None:
        parts.append(f'<line x1="{left + LEFT}" y1="{y(none)}" x2="{right - 20}" y2="{y(none)}" stroke="#bbb" '
                     f'stroke-dasharray="5 4"/>')
        parts.append(f'<text x="{right - 20}" y="{y(none) - 6}" text-anchor="end" fill="{SOFT}">no memory '
                     f'{none:.0%}</text>')
    for a, b in zip(sizes, sizes[1:]):
        better = md[a] + md[b] >= celis[a] + celis[b]
        shape = f"{x(a)},{y(md[a])} {x(b)},{y(md[b])} {x(b)},{y(celis[b])} {x(a)},{y(celis[a])}"
        parts.append(f'<polygon points="{shape}" fill="{BETTER if better else WORSE}" opacity="0.14"/>')
    for colour, series in ((MD, md), (CELIS, celis)):
        line = " ".join(f"{x(n):.1f},{y(v):.1f}" for n, v in series.items())
        parts.append(f'<polyline points="{line}" fill="none" stroke="{colour}" stroke-width="3"/>')
        parts += [f'<circle cx="{x(n):.1f}" cy="{y(v):.1f}" r="4.5" fill="{colour}"/>' for n, v in series.items()]
    for n in sizes:
        delta = round((celis[n] - md[n]) * 100)
        colour = BETTER if delta <= 0 else WORSE
        top = min(y(md[n]), y(celis[n])) - 12
        parts.append(f'<text x="{x(n):.1f}" y="{top:.1f}" text-anchor="middle" font-size="14" font-weight="700" '
                     f'fill="{colour}">{"−" if delta < 0 else "+" if delta > 0 else "±"}{abs(delta)}</text>')
        parts.append(f'<text x="{x(n):.1f}" y="{H - BOTTOM + 20}" text-anchor="middle" fill="{SOFT}">{n}</text>')
    parts.append(f'<text x="{left + LEFT + (PANEL - LEFT - 20) / 2}" y="{H - BOTTOM + 40}" text-anchor="middle" '
                 f'fill="{SOFT}">rules in memory</text>')
    parts.append(f'<text x="{left + LEFT - 8}" y="{H - BOTTOM + 58}" text-anchor="end" font-size="10" '
                 f'fill="{SOFT}">agent file of</text>')
    for name, n in REPOS.items():
        parts.append(f'<line x1="{x(n):.1f}" y1="{H - BOTTOM + 50}" x2="{x(n):.1f}" y2="{H - BOTTOM + 58}" '
                     f'stroke="{SOFT}"/>')
        parts.append(f'<text x="{x(n):.1f}" y="{H - BOTTOM + 70}" text-anchor="middle" font-size="10" '
                     f'fill="{SOFT}" transform="rotate(-25 {x(n):.1f} {H - BOTTOM + 70})">{name}</text>')
    return parts


def main():
    model = (sys.argv[1:] or ["opus"])[0]
    arm = (sys.argv[2:] or ["final"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    plain = report.lines(f"plain-{model}")
    skip = report.habits(plain)
    none = report.measures(plain, skip, replaced)["broken"]
    width = 2 * PANEL + GAP
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {H}" width="{width}" height="{H}" '
             f'font-family="-apple-system, Segoe UI, Helvetica, Arial, sans-serif" font-size="12">',
             f'<rect width="{width}" height="{H}" fill="#fff"/>',
             f'<text x="{LEFT}" y="24" font-size="13" fill="{SOFT}">Claude Code, {model.capitalize()}. Lower is '
             f'better. Labels: celis minus CLAUDE.md, in points.</text>',
             f'<circle cx="{LEFT + 6}" cy="44" r="5" fill="{MD}"/><text x="{LEFT + 16}" y="48" font-weight="700" '
             f'fill="{MD}">CLAUDE.md with every rule</text>',
             f'<circle cx="{LEFT + 206}" cy="44" r="5" fill="{CELIS}"/><text x="{LEFT + 216}" y="48" '
             f'font-weight="700" fill="{CELIS}">celis</text>']
    for i, (key, title, base) in enumerate((("broken", "Rules broken", none), ("old", "Outdated rules followed",
                                                                               None))):
        md = points("oracle", model, key, skip, replaced)
        celis = points(arm, model, key, skip, replaced)
        parts += panel(i * (PANEL + GAP), title, md, celis, base)
        print(title, "  ".join(f"{n}: {md[n]:.0%} → {celis[n]:.0%}" for n in md))
    parts.append("</svg>")
    OUT.write_text("\n".join(parts) + "\n")
    print(OUT)


if __name__ == "__main__":
    main()
