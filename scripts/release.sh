#!/bin/sh
# Opens the release PR of a version:
#
#   make release V=1.2.3
#
# It writes the version in each plugin manifest on a `release/v1.2.3` branch from an up-to-date main, and opens
# a PR. Squash-merge the PR once its checks pass: the merge tags the commit `v1.2.3` and publishes the release.
# It stops before it changes anything when the run is wrong, and says what to do.
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

[ "$(git branch --show-current)" = main ] || fail "switch to main first: git switch main"
git diff --quiet && git diff --cached --quiet || fail "commit or stash your changes first."
git fetch --quiet origin main
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] ||
	fail "main is not the same as origin/main; run git pull --ff-only, or push or drop your commits."

branch="release/v$version"
git ls-remote --exit-code --heads origin "$branch" >/dev/null &&
	fail "the branch $branch is already on GitHub; finish or close its PR, or delete the branch."
git show-ref --quiet --verify "refs/heads/$branch" &&
	fail "the branch $branch is already here; delete it first: git branch -D $branch"

manifests=$(git ls-files gemini-extension.json '*/plugin.json')
git switch --quiet -c "$branch"
for manifest in $manifests; do
	sed "s/\"version\": \"[^\"]*\"/\"version\": \"$version\"/" "$manifest" >"$manifest.tmp"
	mv "$manifest.tmp" "$manifest"
done
git commit --quiet -m "release: v$version" -- $manifests
git push --quiet -u origin "$branch"
url=$(gh pr create --title "release: v$version" --body "Writes v$version in each plugin manifest.

Squash-merge this PR once its checks pass. The merge tags the commit \`v$version\` and publishes the release.")
git switch --quiet main

echo "release: opened $url; squash-merge it once its checks pass."
