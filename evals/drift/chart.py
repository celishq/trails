#!/usr/bin/env python3
"""Draws two benchmark cards in media/: the outdated rules the agent followed, and the rules it broke, at each
memory size, a CLAUDE.md that holds every rule against Celis, as pairs of columns.

  chart.py [model] [celis arm]    opus and final when none is named

Each column pools every task of every run at that size; habits are left out as report.py leaves them out. Marks
under the sizes show where the root agent file of some well-known repos falls. The cards embed Anton and
JetBrains Mono from data/fonts (SIL Open Font License), so they look the same anywhere.
"""

import base64
import json
import random
import sys
from math import log
from pathlib import Path

import report

HERE = Path(__file__).resolve().parent
MEDIA = HERE.parents[1] / "media"
FONTS = HERE / "data" / "fonts"
W, H, M, LEFT, TOP, BOTTOM, BAR, PAIR = 880, 700, 56, 96, 300, 190, 48, 6
BG, INK, CLAUDE, GREY, RULE = "#e8e7e1", "#1e1e1e", "#c15f3c", "#8a8a86", "#cfcdc6"
REPOS = {"VS Code": 79, "Next.js": 123, "kube-router": 282}
CARDS = (("old", "outdated-rules.svg", "Outdated rules followed",
          "When a rule changed, how often the agent followed its old version.", 1.1),
         ("broken", "rules-broken.svg", "Rules broken",
          "Of the rules each task checks, how many the code broke.", 0.8))
STYLE = """
  .head { font-family: Anton, Impact, 'Arial Narrow', sans-serif; text-transform: uppercase }
  .mono { font-family: 'JetBrains Mono', 'SF Mono', Menlo, monospace }
  .caps { text-transform: uppercase; letter-spacing: 0.12em; font-weight: 700 }
"""


def fonts():
    """The @font-face rules of the embedded fonts."""
    face = lambda name, file: (f"@font-face {{ font-family: '{name}'; src: url(data:font/woff2;base64,"
                               f"{base64.b64encode((FONTS / file).read_bytes()).decode()}) format('woff2'); }}")
    return face("Anton", "Anton-Regular.woff2") + "\n" + face("JetBrains Mono", "JetBrainsMono.woff2")


def shares(arm, model, key, skip, replaced):
    """The share `key` of `arm` at each memory size."""
    found = {}
    for path in (HERE / "results").glob(f"{arm}-{model}-r*.jsonl"):
        found[int(path.stem.split("-r")[-1])] = report.measures(report.lines(path.stem), skip, replaced)[key]
    return dict(sorted(found.items()))


def ornament(y):
    """The top rule of the card: crosses, a line, ticks, three squares and a barcode."""
    parts = [f'<path d="M{M} {y}h16M{M + 8} {y - 8}v16M{W - M - 16} {y}h16M{W - M - 8} {y - 8}v16" '
             f'stroke="{INK}" stroke-width="2.5"/>',
             f'<line x1="{M + 26}" y1="{y}" x2="{W - 380}" y2="{y}" stroke="{INK}" stroke-width="2.5"/>']
    parts += [f'<line x1="{x}" y1="{y - 7}" x2="{x}" y2="{y + 7}" stroke="{INK}" stroke-width="1.5"/>'
              for x in range(W - 365, W - 205, 9)]
    parts += [f'<rect x="{W - 192 + 15 * k}" y="{y - 6}" width="11" height="12" '
              f'fill="{CLAUDE if k == 2 else INK}"/>' for k in range(3)]
    pick, x = random.Random(7), W - 135
    while x < W - M - 30:
        width = pick.choice((1.5, 1.5, 3))
        parts.append(f'<rect x="{x}" y="{y - 7}" width="{width}" height="14" fill="{INK}"/>')
        x += width + pick.choice((2, 3))
    return parts


def marks(sizes, centre, slot, row):
    """The repos under the axis: a tick at each one's rule count, its label on the first row that is free."""
    parts = [f'<line x1="{centre(0)}" y1="{row}" x2="{centre(len(sizes) - 1)}" y2="{row}" stroke="{GREY}"/>',
             f'<text class="mono" x="{centre(0) - 10}" y="{row + 4}" font-size="11" fill="{GREY}" '
             f'text-anchor="end">AGENTS.md of</text>']
    placed = []
    for name, n in REPOS.items():
        i = max(k for k, size in enumerate(sizes[:-1]) if size <= n)
        x = centre(i) + (log(n) - log(sizes[i])) / (log(sizes[i + 1]) - log(sizes[i])) * slot
        half = len(f"{name} {n}") * 3.4 + 4
        free = lambda level: all(
            (r != level or abs(x - other) >= half + w) and (r >= level or abs(x - other) >= w)
            and (r <= level or abs(other - x) >= half) for other, w, r in placed)
        level = next((level for level in range(3) if free(level)), 2)
        placed.append((x, half, level))
        drop = row + 6 + 16 * level
        parts += [f'<line x1="{x:.1f}" y1="{row - 4}" x2="{x:.1f}" y2="{drop}" stroke="{GREY}"/>',
                  f'<text class="mono" x="{x:.1f}" y="{drop + 13}" font-size="11" fill="{GREY}" '
                  f'text-anchor="middle">{name} {n}</text>']
    return parts


def card(model, title, note, top, md, celis):
    """One card: the headline, the legend, a pair of columns at each memory size, the repos, and the footer."""
    sizes = list(md)
    last = sizes[-1]
    gap = round((md[last] - celis[last]) * 100)
    y = lambda v: TOP + (1 - v / top) * (H - TOP - BOTTOM)
    slot = (W - LEFT - M) / len(sizes)
    centre = lambda i: LEFT + slot * (i + 0.5)
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
             f"<style>{fonts()}{STYLE}</style>", f'<rect width="{W}" height="{H}" fill="{BG}"/>']
    parts += ornament(64)
    parts += [f'<text class="head" x="{M}" y="156" font-size="54" fill="{INK}">{gap} pts <tspan fill="{GREY}">'
              f'fewer</tspan></text>',
              f'<text class="head" x="{M}" y="222" font-size="54" fill="{INK}">{title}</text>',
              f'<text class="mono" x="{M}" y="254" font-size="13" fill="{GREY}">{note}</text>',
              f'<text class="mono" x="{M}" y="272" font-size="13" fill="{GREY}">Claude Code with '
              f'{model.capitalize()}, 8 tasks at each size. The headline gap is at {last:,} rules.</text>']
    for k, (name, colour) in enumerate((("CLAUDE.md", CLAUDE), ("Celis", INK))):
        parts += [f'<text class="mono caps" x="{W - M - 26}" y="{132 + 30 * k}" font-size="14" fill="{INK}" '
                  f'text-anchor="end">{name}</text>',
                  f'<rect x="{W - M - 16}" y="{120 + 30 * k}" width="16" height="14" fill="{colour}"/>']
    parts.append(f'<text class="mono caps" x="{W - M}" y="198" font-size="14" fill="{INK}" '
                 f'text-anchor="end">↓ Lower is better</text>')
    for step in range(5):
        v = min(top, 1.0) * step / 4
        parts += [f'<line x1="{LEFT}" y1="{y(v):.1f}" x2="{W - M}" y2="{y(v):.1f}" stroke="{RULE}"/>',
                  f'<text class="mono" x="{LEFT - 12}" y="{y(v) + 4:.1f}" font-size="11" fill="{GREY}" '
                  f'text-anchor="end">{v:.0%}</text>']
    parts.append(f'<rect x="{LEFT}" y="{y(0):.1f}" width="{W - M - LEFT}" height="2.5" fill="{INK}"/>')
    for i, n in enumerate(sizes):
        for colour, value, dx in ((CLAUDE, md[n], -BAR - PAIR / 2), (INK, celis[n], PAIR / 2)):
            x = centre(i) + dx
            parts += [f'<rect x="{x:.1f}" y="{y(value):.1f}" width="{BAR}" height="{max(y(0) - y(value), 1.5):.1f}" '
                      f'fill="{colour}"/>',
                      f'<text class="head" x="{x + BAR / 2:.1f}" y="{y(value) - 8:.1f}" font-size="22" '
                      f'fill="{colour}" text-anchor="middle">{value:.0%}</text>']
        delta = round((celis[n] - md[n]) * 100)
        text = f"{'−' if delta < 0 else '+'}{abs(delta)} pts" if delta else "even"
        parts += [f'<text class="mono caps" x="{centre(i):.1f}" y="{y(0) + 26:.1f}" font-size="13" fill="{INK}" '
                  f'text-anchor="middle">{n:,} rules</text>',
                  f'<text class="mono" x="{centre(i):.1f}" y="{y(0) + 46:.1f}" font-size="13" font-weight="700" '
                  f'fill="{INK if delta <= 0 else CLAUDE}" text-anchor="middle">{text}</text>']
    parts += marks(sizes, centre, slot, y(0) + 74)
    parts += [f'<text class="mono caps" x="{M}" y="{H - 30}" font-size="15" fill="{GREY}">Benchmark</text>',
              f'<text class="mono caps" x="{W - M}" y="{H - 30}" font-size="15" fill="{GREY}" '
              f'text-anchor="end">celis.ai</text>', "</svg>"]
    return "\n".join(parts) + "\n"


def main():
    model = (sys.argv[1:] or ["opus"])[0]
    arm = (sys.argv[2:] or ["final"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    skip = report.habits(report.lines(f"plain-{model}"))
    for key, name, title, note, top in CARDS:
        md = shares("oracle", model, key, skip, replaced)
        celis = shares(arm, model, key, skip, replaced)
        (MEDIA / name).write_text(card(model, title, note, top, md, celis))
        print(name, "  ".join(f"{n}: {md[n]:.0%} vs {celis[n]:.0%}" for n in md))


if __name__ == "__main__":
    main()
