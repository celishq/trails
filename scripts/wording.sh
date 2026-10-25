#!/bin/sh
# Runs eval cases on a model with the card's Recall move as a file words it, or as the card has it when the file is "-"; usage: sh scripts/wording.sh <model> <move file | -> <case>...
set -e
model="$1"; move="$2"; shift 2
root=$(cd "$(dirname "$0")/.." && pwd)
name=$(basename "$move" .md)
[ "$move" = - ] && name=card
copy=$(mktemp -d)
mkdir -p "$copy/plugins"
cp -R "$root/plugins/celis" "$copy/plugins/celis"
cp -R "$root/evals" "$copy/evals"
cp -R "$root/.claude-plugin" "$copy/.claude-plugin"
rm -rf "$copy/evals/results"
if [ "$move" != - ]; then
  python3 -I - "$copy/plugins/celis/card.md" "$root/$move" <<'PY'
import sys
card, move = sys.argv[1], open(sys.argv[2]).read()
text = open(card).read()
start, end = text.index("1. **Recall.**"), text.index("2. **Work.**")
open(card, "w").write(text[:start] + move + text[end:])
PY
fi
cd "$copy"
for case in "$@"; do
  PATH="$root/crates/target/debug:$PATH" claude plugin eval . --case "$case" --model "$model" \
    --ablation none --scaffold --trust-plugin --allow-tools Write Edit Bash -j 4 --judge-model sonnet --output-dir "$root/evals/results/$name-$model-$(echo "$case" | tr -d "*")"
done
