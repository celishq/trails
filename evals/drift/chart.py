#!/usr/bin/env python3
"""Draws two charts in media/: the outdated rules the agent followed, and the rules it broke, at each memory size,
a CLAUDE.md that holds every rule against celis, as pairs of bars with the gap under each pair.

  chart.py [model] [celis arm]    opus and final when none is named

Each bar pools every task of every run at that size; habits are left out as report.py leaves them out. Marks
under the sizes show where the root agent file of some well-known repos falls.
"""

import json
import sys
from math import log
from pathlib import Path

import report

HERE = Path(__file__).resolve().parent
MEDIA = HERE.parents[1] / "media"
W, H, LEFT, RIGHT, TOP, BOTTOM, BAR, PAIR = 880, 466, 64, 32, 104, 128, 44, 6
REPOS = {"VS Code": 79, "Next.js": 123, "PostHog": 160, "kube-router": 282}
CHARTS = (("old", "outdated-rules.svg", "Outdated rules the agent followed",
           "When a rule changed, how often the agent followed its old version. Lower is better.", 1.0),
          ("broken", "rules-broken.svg", "Rules the agent broke, as its memory grows",
           "Of the rules each task checks, how many the code broke. Lower is better.", 0.8))
STYLE = """
  .board { fill: #fff } .ink { fill: #23262e } .secondary { fill: rgb(35 38 46 / 0.6) }
  .grid { stroke: rgb(35 38 46 / 0.08) } .rule { stroke: rgb(35 38 46 / 0.3) }
  .claude { fill: #d97757 } .celis { fill: #517499 } .worse { fill: #b4492f }
  text { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
         font-variant-numeric: tabular-nums }
  .title { font-size: 20px; font-weight: 600 } .body { font-size: 13px } .value { font-size: 13px; font-weight: 600 }
  .size { font-size: 14px; font-weight: 600 } .delta { font-size: 13px; font-weight: 700 }
  .mark { font-size: 11px }
  @media (prefers-color-scheme: dark) {
    .board { fill: #0d1117 } .ink { fill: #e6edf5 } .secondary { fill: rgb(230 237 245 / 0.6) }
    .grid { stroke: rgb(230 237 245 / 0.1) } .rule { stroke: rgb(230 237 245 / 0.35) }
    .celis { fill: #6b9bd1 } .worse { fill: #e07a62 }
  }
"""


def shares(arm, model, key, skip, replaced):
    """The share `key` of `arm` at each memory size."""
    found = {}
    for path in (HERE / "results").glob(f"{arm}-{model}-r*.jsonl"):
        found[int(path.stem.split("-r")[-1])] = report.measures(report.lines(path.stem), skip, replaced)[key]
    return dict(sorted(found.items()))


def draw(model, title, note, top, md, celis):
    """One chart: a pair of bars at each memory size, the gap under it, and the marks of the repos."""
    sizes = list(md)
    y = lambda v: TOP + (1 - v / top) * (H - TOP - BOTTOM)
    slot = (W - LEFT - RIGHT) / len(sizes)
    centre = lambda i: LEFT + slot * (i + 0.5)
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
             f"<style>{STYLE}</style>", f'<rect class="board" width="{W}" height="{H}" rx="12"/>',
             f'<text class="title ink" x="{LEFT}" y="40">{title}</text>',
             f'<text class="body secondary" x="{LEFT}" y="62">{note} Claude Code with {model.capitalize()}, 8 '
             f'tasks at each size.</text>',
             f'<rect class="claude" x="{LEFT}" y="76" width="12" height="12" rx="3"/>',
             f'<text class="body ink" x="{LEFT + 18}" y="86">CLAUDE.md with every rule</text>',
             f'<rect class="celis" x="{LEFT + 210}" y="76" width="12" height="12" rx="3"/>',
             f'<text class="body ink" x="{LEFT + 228}" y="86">Celis</text>']
    for step in range(5):
        v = top * step / 4
        parts.append(f'<line class="grid" x1="{LEFT}" y1="{y(v):.1f}" x2="{W - RIGHT}" y2="{y(v):.1f}"/>')
        parts.append(f'<text class="mark secondary" x="{LEFT - 10}" y="{y(v) + 4:.1f}" text-anchor="end">'
                     f'{v:.0%}</text>')
    for i, n in enumerate(sizes):
        for cls, value, dx in (("claude", md[n], -BAR - PAIR / 2), ("celis", celis[n], PAIR / 2)):
            x = centre(i) + dx
            parts.append(f'<rect class="{cls}" x="{x:.1f}" y="{y(value):.1f}" width="{BAR}" '
                         f'height="{max(y(0) - y(value), 1.5):.1f}" rx="4"/>')
            parts.append(f'<text class="value ink" x="{x + BAR / 2:.1f}" y="{y(value) - 8:.1f}" '
                         f'text-anchor="middle">{value:.0%}</text>')
        delta = round((celis[n] - md[n]) * 100)
        text = f"{'−' if delta < 0 else '+'}{abs(delta)} pts" if delta else "even"
        parts.append(f'<text class="size ink" x="{centre(i):.1f}" y="{y(0) + 24:.1f}" text-anchor="middle">'
                     f'{n:,} rules</text>')
        parts.append(f'<text class="delta {"celis" if delta <= 0 else "worse"}" x="{centre(i):.1f}" '
                     f'y="{y(0) + 44:.1f}" text-anchor="middle">{text}</text>')
    row = y(0) + 66
    parts.append(f'<line class="rule" x1="{centre(0)}" y1="{row}" x2="{centre(len(sizes) - 1)}" y2="{row}"/>')
    parts.append(f'<text class="mark secondary" x="{centre(0) - 10}" y="{row + 4}" text-anchor="end">'
                 f'AGENTS.md of</text>')
    marks = []
    for name, n in REPOS.items():
        i = max(k for k, size in enumerate(sizes[:-1]) if size <= n)
        marks.append((centre(i) + (log(n) - log(sizes[i])) / (log(sizes[i + 1]) - log(sizes[i])) * slot,
                      f"{name} {n}"))
    widths = [len(label) * 6.2 for _, label in marks]
    spots = [x for x, _ in marks]
    for k in range(1, len(spots)):
        spots[k] = max(spots[k], spots[k - 1] + (widths[k - 1] + widths[k]) / 2 + 14)
    shift = sum(x - spot for (x, _), spot in zip(marks, spots)) / len(spots)
    spots = [spot + shift for spot in spots]
    for (x, label), spot in zip(marks, spots):
        parts.append(f'<polyline class="rule" points="{x:.1f},{row - 4} {x:.1f},{row + 6} {spot:.1f},{row + 16}" '
                     f'fill="none"/>')
        parts.append(f'<text class="mark secondary" x="{spot:.1f}" y="{row + 30}" text-anchor="middle">'
                     f'{label}</text>')
    parts.append("</svg>")
    return "\n".join(parts) + "\n"


def main():
    model = (sys.argv[1:] or ["opus"])[0]
    arm = (sys.argv[2:] or ["final"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    skip = report.habits(report.lines(f"plain-{model}"))
    for key, name, title, note, top in CHARTS:
        md = shares("oracle", model, key, skip, replaced)
        celis = shares(arm, model, key, skip, replaced)
        (MEDIA / name).write_text(draw(model, title, note, top, md, celis))
        print(name, "  ".join(f"{n}: {md[n]:.0%} vs {celis[n]:.0%}" for n in md))


if __name__ == "__main__":
    main()
