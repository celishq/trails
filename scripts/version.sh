#!/bin/sh
# Prints the version a release would publish:
#
#   scripts/version.sh          the manifests' version, or nothing when it is 0.0.0 or already released
#   scripts/version.sh 1.2.3    1.2.3, when it can be the next release
#
# It fails, with a hint, on a version that is not like 1.2.3 or not higher than the latest release.
set -eu

fail() {
	echo "release: $1" >&2
	exit 1
}

cd "$(git rev-parse --show-toplevel)"
if [ $# -eq 0 ]; then
	version=$(sed -n 's/.*"version": "\([^"]*\)".*/\1/p' gemini-extension.json)
	[ "$version" != 0.0.0 ] || exit 0
	! git ls-remote --exit-code --tags origin "refs/tags/v$version" >/dev/null || exit 0
else
	version=$1
fi

case "$version" in
v*) fail "write the version without the v: ${version#v}" ;;
esac
echo "$version" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' ||
	fail "$version is not a version like 1.2.3 (three numbers, no leading zeros, no suffix)."
[ "$version" != 0.0.0 ] || fail "0.0.0 is the version of a build from the repo; pick a higher one."

latest=$(git ls-remote --tags --refs origin 'v*' | sed 's|.*refs/tags/v||' | sort -V | tail -n 1)
if [ -n "$latest" ] && { [ "$latest" = "$version" ] ||
	[ "$(printf '%s\n%s\n' "$latest" "$version" | sort -V | tail -n 1)" != "$version" ]; }; then
	fail "$version is not higher than the latest release, $latest; pick a higher one."
fi

echo "$version"
