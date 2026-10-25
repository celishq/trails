#!/bin/sh
# Runs the given eval cases twice, with the card's Recall move as it is (form A) and as evals/wording/recall-b.md words it (form B), on the given model.
set -e
model="$1"; shift
root=$(cd "$(dirname "$0")/.." && pwd)
for form in a b; do
  copy=$(mktemp -d)
  mkdir -p "$copy/plugins"
  cp -R "$root/plugins/celis" "$copy/plugins/celis"
  cp -R "$root/evals" "$copy/evals"
  rm -rf "$copy/evals/results"
  if [ "$form" = b ]; then
    python3 -I - "$copy/plugins/celis/card.md" "$root/evals/wording/recall-b.md" <<'PY'
import sys
card, move = sys.argv[1], open(sys.argv[2]).read()
text = open(card).read()
start, end = text.index("1. **Recall.**"), text.index("2. **Work.**")
open(card, "w").write(text[:start] + move + text[end:])
PY
  fi
  for case in "$@"; do
    claude plugin eval "$copy/plugins/celis" --eval-dir ../../evals --case "$case" --model "$model" \
      --ablation none --scaffold --output-dir "$root/evals/results/wording-$form-$model"
  done
done
