#!/bin/sh
# Fails an eval case unless its agent runs the celis built from this repo: the first on PATH, inside the
# repo, where the eval sandbox lets the agent run it, and built from the crates at HEAD.
root="$(cd "$(dirname "$0")/.." && pwd)"
bin="$(command -v celis)" || { echo "eval: no celis on PATH; run cargo build --release in $root/crates" >&2; exit 1; }
case "$bin" in
  "$root"/*) ;;
  *) echo "eval: $bin is outside $root, so the agent cannot run it; put $root/crates/target/release first on PATH" >&2; exit 1 ;;
esac
built="$(celis --version | sed -n 's/.*(\([0-9a-f]*\)).*/\1/p')"
git -C "$root" diff --quiet "$built" HEAD -- crates 2>/dev/null || {
  echo "eval: $bin was built from ${built:-an unknown commit}, not from the crates at HEAD; run cargo build --release in $root/crates" >&2
  exit 1
}
