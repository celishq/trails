#!/bin/sh
# Opens the release PR of a version:
#
#   make release V=1.2.3
#
# It commits the version in each plugin manifest on an up-to-date dev, pushes dev, opens the PR from dev into
# main, waits for its checks, and fast-forwards main to that commit, so dev and main stay the same. The push tags
# the commit `v1.2.3` and publishes the release. It stops before it changes anything when the run is wrong, and
# says what to do.
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
sha=$(git rev-parse HEAD)
url=$(gh pr create --base main --head dev --title "release: v$version" --body "Releases v$version.

\`make release\` fast-forwards main to this commit once its checks pass. The push tags the commit \`v$version\` and publishes the release.")

echo "release: opened $url.
release: keep this terminal open while its checks run. When they pass, this pushes the release commit to main,
release: and the Release run publishes v$version. If you stop it, run this once the checks pass:
release:   git push origin $sha:main"
tries=0
until [ "$(gh pr checks "$url" --json name --jq length 2>/dev/null || echo 0)" -gt 0 ]; do
	tries=$((tries + 1))
	[ "$tries" -le 30 ] || fail "no checks started on $url; once they pass, run: git push origin $sha:main"
	sleep 5
done
gh pr checks "$url" --watch --fail-fast ||
	fail "a check failed on $url; fix it in a PR into dev, then close $url and release the next version."
[ "$(gh pr view "$url" --json headRefOid --jq .headRefOid)" = "$sha" ] ||
	fail "dev moved while the checks ran; close $url and release again."
git push --quiet origin "$sha:main"

echo "release: main is at v$version; the Release run publishes it in a few minutes."
