#!/usr/bin/env python3
"""Draws two benchmark cards in media/: the outdated rules the agent followed, and the rules it broke, at each
memory size, a CLAUDE.md that holds every rule against Celis.

  chart.py [model] [celis arm]    opus and final when none is named

Each bar pools every task of every run at that size; habits are left out as report.py leaves them out. The
cards embed Anton and JetBrains Mono from data/fonts (SIL Open Font License), so they look the same anywhere.
"""

import base64
import json
import random
import sys
from pathlib import Path

import report

HERE = Path(__file__).resolve().parent
MEDIA = HERE.parents[1] / "media"
FONTS = HERE / "data" / "fonts"
W, M, BAR_X, BAR_W, VALUE_X, ROW, HEAD = 880, 56, 210, 470, 824, 50, 44
BG, INK, ORANGE, GREY = "#e8e7e1", "#1e1e1e", "#e85d2f", "#8a8a86"
REPOS = {"VS Code": 79, "Next.js": 123, "PostHog": 160, "kube-router": 282}
CARDS = (("old", "outdated-rules.svg", "Outdated rules followed",
          "When a rule changed, how often the agent followed its old version."),
         ("broken", "rules-broken.svg", "Rules broken",
          "Of the rules each task checks, how many the code broke."))
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
              f'fill="{ORANGE if k == 2 else INK}"/>' for k in range(3)]
    pick, x = random.Random(7), W - 135
    while x < W - M - 30:
        width = pick.choice((1.5, 1.5, 3))
        parts.append(f'<rect x="{x}" y="{y - 7}" width="{width}" height="14" fill="{INK}"/>')
        x += width + pick.choice((2, 3))
    return parts


def card(model, title, note, md, celis):
    """One card: the headline, the legend, a pair of bars at each memory size, the repos, and the footer."""
    sizes = list(md)
    last = sizes[-1]
    gap = round((md[last] - celis[last]) * 100)
    top = 310
    height = top + len(sizes) * (HEAD + 2 * ROW) + 120
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {height}" width="{W}" height="{height}">',
             f"<style>{fonts()}{STYLE}</style>", f'<rect width="{W}" height="{height}" fill="{BG}"/>']
    parts += ornament(64)
    parts += [f'<text class="head" x="{M}" y="160" font-size="66" fill="{INK}"><tspan fill="{ORANGE}">{gap} pts'
              f'</tspan> fewer</text>',
              f'<text class="head" x="{M}" y="230" font-size="66" fill="{INK}">{title}</text>',
              f'<text class="mono" x="{M}" y="262" font-size="13" fill="{GREY}">{note}</text>',
              f'<text class="mono" x="{M}" y="280" font-size="13" fill="{GREY}">Claude Code with '
              f'{model.capitalize()}, 8 tasks at each size. The headline gap is at {last:,} rules.</text>']
    for k, (name, colour) in enumerate((("CLAUDE.md", INK), ("Celis", ORANGE))):
        parts += [f'<text class="mono caps" x="{W - M - 26}" y="{132 + 30 * k}" font-size="14" fill="{INK}" '
                  f'text-anchor="end">{name}</text>',
                  f'<rect x="{W - M - 16}" y="{120 + 30 * k}" width="16" height="14" fill="{colour}"/>']
    parts.append(f'<text class="mono caps" x="{W - M}" y="{top + 22}" font-size="14" fill="{ORANGE}" '
                 f'text-anchor="end">↓ Lower is better</text>')
    y = top
    for n in sizes:
        parts += [f'<text class="mono caps" x="{M}" y="{y + 22}" font-size="14" fill="{INK}">{n:,} rules in memory'
                  f'</text>',
                  f'<rect x="{M}" y="{y + HEAD - 6}" width="{W - 2 * M}" height="3.5" fill="{INK}"/>']
        y += HEAD
        delta = round((celis[n] - md[n]) * 100)
        gain = f"{'−' if delta < 0 else '+'}{abs(delta)} pts" if delta else "even"
        for name, value, colour, under in (("CLAUDE.md", md[n], INK, ""), ("Celis", celis[n], ORANGE, gain)):
            parts += [f'<text class="mono" x="{M}" y="{y + ROW / 2 + 5}" font-size="15" font-weight="500" '
                      f'fill="{INK}">{name}</text>',
                      f'<rect x="{BAR_X}" y="{y + 12}" width="{max(value * BAR_W, 2):.1f}" height="{ROW - 24}" '
                      f'fill="{colour}"/>',
                      f'<text class="head" x="{VALUE_X}" y="{y + ROW / 2 + 11}" font-size="30" fill="{colour}" '
                      f'text-anchor="end">{value:.0%}</text>']
            if under:
                parts.append(f'<text class="mono" x="{VALUE_X - 70}" y="{y + ROW / 2 + 5}" font-size="12" '
                             f'font-weight="700" fill="{ORANGE if delta <= 0 else INK}" text-anchor="end">{under}'
                             f'</text>')
            y += ROW
            parts.append(f'<line x1="{M}" y1="{y}" x2="{W - M}" y2="{y}" stroke="{INK}" stroke-width="1"/>')
    repos = " · ".join(f"{name} {n}" for name, n in REPOS.items())
    parts += [f'<text class="mono" x="{M}" y="{y + 40}" font-size="12" fill="{GREY}">Rules in the root agent file '
              f'of real repos: {repos}</text>',
              f'<text class="mono caps" x="{M}" y="{height - 34}" font-size="15" fill="{GREY}">Benchmark</text>',
              f'<text class="mono caps" x="{W - M}" y="{height - 34}" font-size="15" fill="{GREY}" '
              f'text-anchor="end">celis.ai</text>', "</svg>"]
    return "\n".join(parts) + "\n"


def main():
    model = (sys.argv[1:] or ["opus"])[0]
    arm = (sys.argv[2:] or ["final"])[0]
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    replaced = {t["id"] for t in targets if "old" in t}
    skip = report.habits(report.lines(f"plain-{model}"))
    for key, name, title, note in CARDS:
        md = shares("oracle", model, key, skip, replaced)
        celis = shares(arm, model, key, skip, replaced)
        (MEDIA / name).write_text(card(model, title, note, md, celis))
        print(name, "  ".join(f"{n}: {md[n]:.0%} vs {celis[n]:.0%}" for n in md))


if __name__ == "__main__":
    main()
