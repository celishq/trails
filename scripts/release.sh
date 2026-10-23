#!/bin/sh
# Opens the release PR of a version:
#
#   make release V=1.2.3
#
# It commits the version in each plugin manifest on an up-to-date dev, pushes dev, and opens the PR from dev into
# main. Merge the PR with a merge commit once its checks pass: the merge tags the commit `v1.2.3` and publishes
# the release. It stops before it changes anything when the run is wrong, and says what to do.
set -eu

fail() {
	echo "release: $1" >&2
	exit 1
}

[ -n "${1:-}" ] || fail "name the version: make release V=1.2.3"
cd "$(git rev-parse --show-toplevel)"
version=$(sh scripts/version.sh "$1")

command -v gh >/dev/null || fail "install the GitHub CLI first: https://cli.github.com"
gh auth status >/dev/null 2>&1 || fail "sign in to GitHub first: gh auth login"

[ "$(git branch --show-current)" = dev ] || fail "switch to dev first: git switch dev"
git diff --quiet && git diff --cached --quiet || fail "commit or stash your changes first."
git fetch --quiet origin dev
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/dev)" ] ||
	fail "dev is not the same as origin/dev; run git pull --ff-only, or push or drop your commits."
[ -z "$(gh pr list --base main --head dev --json url --jq '.[].url')" ] ||
	fail "a release PR from dev into main is already open; merge or close it first."

manifests=$(git ls-files gemini-extension.json '*/plugin.json')
for manifest in $manifests; do
	sed "s/\"version\": \"[^\"]*\"/\"version\": \"$version\"/" "$manifest" >"$manifest.tmp"
	mv "$manifest.tmp" "$manifest"
done
git commit --quiet -m "release: v$version" -- $manifests
git push --quiet origin dev
url=$(gh pr create --base main --head dev --title "release: v$version" --body "Releases v$version.

Merge this PR with a merge commit once its checks pass. The merge tags the commit \`v$version\` and publishes the release.")

echo "release: opened $url; merge it with a merge commit once its checks pass."
