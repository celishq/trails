#!/usr/bin/env python3
"""Prints and draws the rule-count curve: rules broken and input tokens per session against the rules held.

  curve.py    reads results/<arm>[-<model>]-r<N>.jsonl, writes results/curve.svg
"""

import json
import re
from collections import defaultdict
from math import log
from pathlib import Path

HERE = Path(__file__).resolve().parent
NAME = re.compile(r"^(oracle|celis)(?:-(haiku|opus|team))?-r(\d+)$")
COLORS = {"haiku": "#2a9d8f", "sonnet": "#e76f51", "opus": "#6a4c93", "team": "#264653"}
W, H, PAD = 440, 300, 50


def points():
    """Per model and arm, the rules held mapped to the share broken and the mean input tokens a session."""
    out = defaultdict(dict)
    for path in sorted((HERE / "results").glob("*-r*.jsonl")):
        if not (m := NAME.match(path.stem)):
            continue
        lines = [json.loads(l) for l in path.read_text().splitlines() if l]
        given = sum(len(l["rules"]) for l in lines)
        out[(m[2] or "sonnet", m[1])][int(m[3])] = (sum(r in l["broken"] for l in lines for r in l["rules"]) / given,
                                                   sum(l["session"]["input"] for l in lines) / len(lines) / 1000)
    return out


def panel(series, index, title, top, left):
    """One chart: x is the rules held on a log scale, y is the value at `index`."""
    xs = sorted({n for s in series.values() for n in s})
    peak = max(v[index] for s in series.values() for v in s.values()) or 1
    x = lambda n: left + PAD + (log(n) - log(xs[0])) / ((log(xs[-1]) - log(xs[0])) or 1) * (W - 2 * PAD)
    y = lambda v: top + H - PAD - v / peak * (H - 2 * PAD)
    parts = [f'<text x="{left + W / 2}" y="{top + 20}" text-anchor="middle" font-weight="bold">{title}</text>',
             f'<line x1="{left + PAD}" y1="{top + H - PAD}" x2="{left + W - PAD}" y2="{top + H - PAD}" stroke="#999"/>',
             f'<line x1="{left + PAD}" y1="{top + PAD}" x2="{left + PAD}" y2="{top + H - PAD}" stroke="#999"/>',
             f'<text x="{left + PAD - 6}" y="{top + PAD + 4}" text-anchor="end">'
             + (f"{peak:.0%}" if index == 0 else f"{peak:.0f}k") + "</text>"]
    parts += [f'<text x="{x(n)}" y="{top + H - PAD + 16}" text-anchor="middle">{n}</text>' for n in xs]
    for (model, arm), s in sorted(series.items()):
        path = " ".join(f"{x(n):.1f},{y(v[index]):.1f}" for n, v in sorted(s.items()))
        dash = ' stroke-dasharray="6 4"' if arm == "oracle" else ""
        parts.append(f'<polyline points="{path}" fill="none" stroke="{COLORS[model]}" stroke-width="2"{dash}/>')
    return parts


def main():
    series = points()
    print("model\tarm\t" + "\t".join(str(n) for n in sorted({n for s in series.values() for n in s})))
    for (model, arm), s in sorted(series.items()):
        print(f"{model}\t{arm}\t" + "\t".join(f"{b:.0%} {k:.0f}k" for _, (b, k) in sorted(s.items())))
    legend = [f'<text x="{PAD + 200 * (i % 4)}" y="{H + 10 + 18 * (i // 4)}" fill="{COLORS[m]}">{m} '
              + ("CLAUDE.md (dashed)" if a == "oracle" else "celis") + "</text>"
              for i, (m, a) in enumerate(sorted(series))]
    svg = ([f'<svg xmlns="http://www.w3.org/2000/svg" width="{2 * W}" height="{H + 70}" font-family="sans-serif" '
            f'font-size="12"><rect width="100%" height="100%" fill="white"/>']
           + panel(series, 0, "Review rules broken, by rules held", 0, 0)
           + panel(series, 1, "Input tokens a session, by rules held", 0, W)
           + legend + ["</svg>"])
    (HERE / "results" / "curve.svg").write_text("\n".join(svg) + "\n")


if __name__ == "__main__":
    main()
