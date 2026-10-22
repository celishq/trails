#!/usr/bin/env python3
"""Draws media/claude-md-vs-celis.svg in the style of the celis app: as the memory grows, the rules broken and the
outdated rules followed, a CLAUDE.md that holds every rule against celis, with the gap between them marked.

  chart.py [model] [celis arm]    opus and final when none is named

Each point pools every task of every run at that size; habits are left out as report.py leaves them out. The
ticks under each card mark the rules in the root agent file of some well-known repos.
"""

import json
import sys
from math import log
from pathlib import Path

import report

HERE = Path(__file__).resolve().parent
OUT = HERE.parents[1] / "media" / "claude-md-vs-celis.svg"
CARD, H, GAP, PAD, LEFT, TOP, BOTTOM = 440, 460, 24, 24, 44, 150, 96
REPOS = {"VS Code": 79, "Next.js": 123, "PostHog": 160, "Kiali": 229, "kube-router": 282}
STYLE = """
  .board { fill: #fafcff } .card { fill: #fff; stroke: rgb(81 116 153 / 0.1125) }
  .ink { fill: #23262e } .secondary { fill: rgb(35 38 46 / 0.65) } .accent { fill: #517499 }
  .danger { fill: #944842 } .grid { stroke: rgb(35 38 46 / 0.06) } .rule { stroke: rgb(35 38 46 / 0.225) }
  .md { stroke: rgb(35 38 46 / 0.65) } .md-dot { fill: rgb(35 38 46 / 0.65) }
  .celis { stroke: #517499 } .celis-dot { fill: #517499 } .wash { fill: rgb(81 116 153 / 0.10) }
  .loss { fill: rgb(148 72 66 / 0.12) }
  text { font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Helvetica, Arial, sans-serif;
         font-variant-numeric: tabular-nums; letter-spacing: 0.025em }
  .group { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em }
  .headline { font-size: 42px; font-weight: 500; letter-spacing: -0.02em }
  .body { font-size: 13px } .caption { font-size: 11px } .mark { font-size: 9px; font-weight: 500 }
  .delta { font-size: 12px; font-weight: 600 }
  @media (prefers-color-scheme: dark) {
    .board { fill: #1c1f23 } .card { fill: #1c1d1f; stroke: rgb(100 143 189 / 0.18) }
    .ink { fill: #e6edf5 } .secondary { fill: rgb(230 237 245 / 0.65) } .accent { fill: #648fbd }
    .danger { fill: #cc6d66 } .grid { stroke: rgb(230 237 245 / 0.07) } .rule { stroke: rgb(230 237 245 / 0.225) }
    .md { stroke: rgb(230 237 245 / 0.65) } .md-dot { fill: rgb(230 237 245 / 0.65) }
    .celis { stroke: #648fbd } .celis-dot { fill: #648fbd } .wash { fill: rgb(100 143 189 / 0.14) }
    .loss { fill: rgb(204 109 102 / 0.16) }
  }
"""


def points(arm, model, key, skip, replaced):
    """The share `key` of `arm` at each memory size."""
    found = {}
    for path in (HERE / "results").glob(f"{arm}-{model}-r*.jsonl"):
        found[int(path.stem.split("-r")[-1])] = report.measures(report.lines(path.stem), skip, replaced)[key]
    return dict(sorted(found.items()))


def card(left, title, md, celis, none):
    """One card: a group label, the gap at the largest size as its headline, then CLAUDE.md and celis by size."""
    sizes = list(md)
    plot = (left + PAD + LEFT, left + CARD - PAD)
    x = lambda n: plot[0] + (log(n) - log(sizes[0])) / (log(sizes[-1]) - log(sizes[0])) * (plot[1] - plot[0])
    y = lambda v: TOP + (1 - v) * (H - TOP - BOTTOM)
    last = sizes[-1]
    gap = round((md[last] - celis[last]) * 100)
    parts = [f'<rect class="card" x="{left}" y="0.5" width="{CARD}" height="{H - 1}" rx="12" stroke-width="1"/>',
             f'<text class="group secondary" x="{left + PAD}" y="{PAD + 14}">{title}</text>',
             f'<text class="headline accent" x="{left + PAD}" y="{PAD + 64}">{gap} pts</text>',
             f'<text class="body secondary" x="{left + PAD}" y="{PAD + 86}">fewer with celis at {last:,} rules: '
             f'{celis[last]:.0%} against {md[last]:.0%}</text>']
    for v in (0, 0.25, 0.5, 0.75, 1):
        parts.append(f'<line class="grid" x1="{plot[0]}" y1="{y(v)}" x2="{plot[1]}" y2="{y(v)}"/>')
        parts.append(f'<text class="mark secondary" x="{plot[0] - 8}" y="{y(v) + 3}" text-anchor="end">'
                     f'{v:.0%}</text>')
    if none is not None:
        parts.append(f'<line class="rule" x1="{plot[0]}" y1="{y(none)}" x2="{plot[1]}" y2="{y(none)}" '
                     f'stroke-dasharray="2 2"/>')
        parts.append(f'<text class="caption secondary" x="{plot[1]}" y="{y(none) - 6}" text-anchor="end">'
                     f'no memory {none:.0%}</text>')
    for a, b in zip(sizes, sizes[1:]):
        fill = "wash" if md[a] + md[b] >= celis[a] + celis[b] else "loss"
        shape = f"{x(a):.1f},{y(md[a]):.1f} {x(b):.1f},{y(md[b]):.1f} {x(b):.1f},{y(celis[b]):.1f} " \
                f"{x(a):.1f},{y(celis[a]):.1f}"
        parts.append(f'<polygon class="{fill}" points="{shape}"/>')
    for cls, series in (("md", md), ("celis", celis)):
        line = " ".join(f"{x(n):.1f},{y(v):.1f}" for n, v in series.items())
        parts.append(f'<polyline class="{cls}" points="{line}" fill="none" stroke-width="2" '
                     f'stroke-linecap="round" stroke-linejoin="round"/>')
        parts += [f'<circle class="{cls}-dot" cx="{x(n):.1f}" cy="{y(v):.1f}" r="3.5"/>' for n, v in series.items()]
    for n in sizes:
        delta = round((celis[n] - md[n]) * 100)
        sign = "−" if delta < 0 else "+" if delta > 0 else "±"
        top = min(y(md[n]), y(celis[n])) - 10
        parts.append(f'<text class="delta {"accent" if delta <= 0 else "danger"}" x="{x(n):.1f}" y="{top:.1f}" '
                     f'text-anchor="middle">{sign}{abs(delta)}</text>')
        parts.append(f'<text class="mark secondary" x="{x(n):.1f}" y="{H - BOTTOM + 16}" text-anchor="middle">'
                     f'{n:,}</text>')
    parts.append(f'<text class="caption secondary" x="{(plot[0] + plot[1]) / 2}" y="{H - BOTTOM + 34}" '
                 f'text-anchor="middle">rules in memory</text>')
    parts.append(f'<text class="mark secondary" x="{plot[0] - 8}" y="{H - BOTTOM + 56}" text-anchor="end">'
                 f'AGENTS.md</text>')
    for name, n in REPOS.items():
        parts.append(f'<line class="rule" x1="{x(n):.1f}" y1="{H - BOTTOM + 46}" x2="{x(n):.1f}" '
                     f'y2="{H - BOTTOM + 54}"/>')
        parts.append(f'<text class="mark secondary" x="{x(n):.1f}" y="{H - BOTTOM + 66}" text-anchor="end" '
                     f'transform="rotate(-30 {x(n):.1f} {H - BOTTOM + 66})">{name}</text>')
    return parts


def main():
    model = (sys.argv[1:] or ["opus"])[0]
    arm = (sys.argv[2:] or ["final"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    plain = report.lines(f"plain-{model}")
    skip = report.habits(plain)
    none = report.measures(plain, skip, replaced)["broken"]
    width, height = 2 * CARD + GAP + 2 * PAD, H + 2 * PAD + 34
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" '
             f'height="{height}">', f"<style>{STYLE}</style>",
             f'<rect class="board" width="{width}" height="{height}" rx="18"/>',
             f'<circle class="md-dot" cx="{PAD + 6}" cy="{PAD + 10}" r="4"/>',
             f'<text class="body ink" x="{PAD + 16}" y="{PAD + 14}">CLAUDE.md with every rule</text>',
             f'<circle class="celis-dot" cx="{PAD + 206}" cy="{PAD + 10}" r="4"/>',
             f'<text class="body ink" x="{PAD + 216}" y="{PAD + 14}">celis</text>',
             f'<text class="caption secondary" x="{width - PAD}" y="{PAD + 14}" text-anchor="end">Claude Code, '
             f'{model.capitalize()}. Lower is better. Labels: celis minus CLAUDE.md, in points.</text>',
             f'<g transform="translate({PAD} {PAD + 34})">']
    for i, (key, title, base) in enumerate((("broken", "Rules broken", none),
                                            ("old", "Outdated rules followed", None))):
        md = points("oracle", model, key, skip, replaced)
        celis = points(arm, model, key, skip, replaced)
        parts += card(i * (CARD + GAP), title, md, celis, base)
        print(title, "  ".join(f"{n}: {md[n]:.0%} → {celis[n]:.0%}" for n in md))
    parts += ["</g>", "</svg>"]
    OUT.write_text("\n".join(parts) + "\n")
    print(OUT)


if __name__ == "__main__":
    main()
