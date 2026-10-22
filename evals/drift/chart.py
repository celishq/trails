#!/usr/bin/env python3
"""Draws media/claude-md-vs-celis.svg: the rules broken at each memory size, a CLAUDE.md that holds every rule
against celis, as pairs of bars with the gap under each pair.

  chart.py [model] [celis arm]    opus and final when none is named

Each bar pools every task of every run at that size; habits are left out as report.py leaves them out.
"""

import json
import sys
from pathlib import Path

import report

HERE = Path(__file__).resolve().parent
OUT = HERE.parents[1] / "media" / "claude-md-vs-celis.svg"
W, H, LEFT, RIGHT, TOP, BOTTOM, BAR, PAIR = 880, 410, 64, 32, 104, 72, 44, 6
STYLE = """
  .board { fill: #fff } .ink { fill: #23262e } .secondary { fill: rgb(35 38 46 / 0.6) }
  .grid { stroke: rgb(35 38 46 / 0.08) } .claude { fill: #d97757 } .celis { fill: #517499 }
  .worse { fill: #b4492f }
  text { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
         font-variant-numeric: tabular-nums }
  .title { font-size: 20px; font-weight: 600 } .body { font-size: 13px } .value { font-size: 13px; font-weight: 600 }
  .size { font-size: 14px; font-weight: 600 } .delta { font-size: 13px; font-weight: 700 }
  .mark { font-size: 11px }
  @media (prefers-color-scheme: dark) {
    .board { fill: #0d1117 } .ink { fill: #e6edf5 } .secondary { fill: rgb(230 237 245 / 0.6) }
    .grid { stroke: rgb(230 237 245 / 0.1) } .celis { fill: #6b9bd1 } .worse { fill: #e07a62 }
  }
"""


def broken(arm, model, skip, replaced):
    """The share of rules `arm` broke at each memory size."""
    found = {}
    for path in (HERE / "results").glob(f"{arm}-{model}-r*.jsonl"):
        found[int(path.stem.split("-r")[-1])] = report.measures(report.lines(path.stem), skip, replaced)["broken"]
    return dict(sorted(found.items()))


def main():
    model = (sys.argv[1:] or ["opus"])[0]
    arm = (sys.argv[2:] or ["final"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    skip = report.habits(report.lines(f"plain-{model}"))
    md = broken("oracle", model, skip, replaced)
    celis = broken(arm, model, skip, replaced)
    sizes = list(md)
    top = 0.8
    y = lambda v: TOP + (1 - v / top) * (H - TOP - BOTTOM)
    slot = (W - LEFT - RIGHT) / len(sizes)
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
             f"<style>{STYLE}</style>", f'<rect class="board" width="{W}" height="{H}" rx="12"/>',
             f'<text class="title ink" x="{LEFT}" y="40">Rules the agent broke, as its memory grows</text>',
             f'<text class="body secondary" x="{LEFT}" y="62">Claude Code with {model.capitalize()}, 8 tasks at '
             f'each size. Lower is better.</text>',
             f'<rect class="claude" x="{LEFT}" y="76" width="12" height="12" rx="3"/>',
             f'<text class="body ink" x="{LEFT + 18}" y="86">CLAUDE.md with every rule</text>',
             f'<rect class="celis" x="{LEFT + 210}" y="76" width="12" height="12" rx="3"/>',
             f'<text class="body ink" x="{LEFT + 228}" y="86">celis</text>']
    for v in (0, 0.2, 0.4, 0.6, 0.8):
        parts.append(f'<line class="grid" x1="{LEFT}" y1="{y(v):.1f}" x2="{W - RIGHT}" y2="{y(v):.1f}"/>')
        parts.append(f'<text class="mark secondary" x="{LEFT - 10}" y="{y(v) + 4:.1f}" text-anchor="end">'
                     f'{v:.0%}</text>')
    for i, n in enumerate(sizes):
        centre = LEFT + slot * (i + 0.5)
        for cls, value, dx in (("claude", md[n], -BAR - PAIR / 2), ("celis", celis[n], PAIR / 2)):
            x = centre + dx
            parts.append(f'<rect class="{cls}" x="{x:.1f}" y="{y(value):.1f}" width="{BAR}" '
                         f'height="{y(0) - y(value):.1f}" rx="4"/>')
            parts.append(f'<text class="value ink" x="{x + BAR / 2:.1f}" y="{y(value) - 8:.1f}" '
                         f'text-anchor="middle">{value:.0%}</text>')
        delta = round((celis[n] - md[n]) * 100)
        text = f"{'−' if delta < 0 else '+'}{abs(delta)} pts" if delta else "even"
        parts.append(f'<text class="size ink" x="{centre:.1f}" y="{y(0) + 24:.1f}" text-anchor="middle">'
                     f'{n:,} rules</text>')
        parts.append(f'<text class="delta {"celis" if delta <= 0 else "worse"}" x="{centre:.1f}" '
                     f'y="{y(0) + 46:.1f}" text-anchor="middle">{text}</text>')
    parts.append("</svg>")
    OUT.write_text("\n".join(parts) + "\n")
    print("  ".join(f"{n}: {md[n]:.0%} vs {celis[n]:.0%}" for n in sizes))
    print(OUT)


if __name__ == "__main__":
    main()
