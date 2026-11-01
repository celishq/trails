#!/bin/sh
# Installs the celis command line:
#
#   curl -fsSL https://celis.ai/install.sh | sh
#
# CELIS_VERSION picks a version, `latest` by default. CELIS_INSTALL_DIR picks the folder, ~/.local/bin by
# default. CELIS_BASE picks where the archives are, for a test against another server. When the folder is
# not on PATH, it adds a line to ~/.profile and to the rc file of your shell, so a new terminal finds celis.
# CELIS_UPDATE=1 is for `celis update`: it installs nothing when the latest is installed, and touches no PATH.
set -eu

base="${CELIS_BASE:-https://github.com/celishq/trails/releases}"
version="${CELIS_VERSION:-latest}"
dir="${CELIS_INSTALL_DIR:-$HOME/.local/bin}"

fail() {
	echo "celis: $1" >&2
	exit 1
}

case "$(uname -s)" in
Darwin) system=apple-darwin ;;
Linux) system=unknown-linux-musl ;;
*) fail "there is no build for $(uname -s) yet. On Windows, run this in WSL." ;;
esac
case "$(uname -m)" in
arm64 | aarch64) cpu=aarch64 ;;
x86_64 | amd64) cpu=x86_64 ;;
*) fail "there is no build for the $(uname -m) processor yet." ;;
esac

if [ -n "${CELIS_UPDATE:-}" ] && [ "$version" = latest ]; then
	tag="$(curl -fsS -o /dev/null -w '%{redirect_url}' "$base/latest")" || fail "could not reach $base/latest"
	version="${tag##*/tag/v}"
	[ "$version" != "$tag" ] || fail "$base/latest names no release"
	case "$("$dir/celis" --version 2>/dev/null)" in
	"celis $version" | "celis $version "*)
		echo "celis $version is the latest."
		exit 0
		;;
	esac
fi

name="celis-$cpu-$system.tar.gz"
case "$version" in
latest) url="$base/latest/download/$name" ;;
*) url="$base/download/v$version/$name" ;;
esac
tmp="$(mktemp -d "${TMPDIR:-/tmp}/celis.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $name ($version)…"
curl -fsSL "$url" -o "$tmp/$name" || fail "could not download $url"
curl -fsSL "$url.sha256" -o "$tmp/$name.sha256" || fail "could not download its checksum"

want="$(cut -d ' ' -f 1 <"$tmp/$name.sha256")"
if command -v sha256sum >/dev/null 2>&1; then
	got="$(sha256sum "$tmp/$name" | cut -d ' ' -f 1)"
else
	got="$(shasum -a 256 "$tmp/$name" | cut -d ' ' -f 1)"
fi
[ "$got" = "$want" ] || fail "the download does not match its checksum. Nothing was installed."

tar -xzf "$tmp/$name" -C "$tmp"
mkdir -p "$dir"
mv "$tmp/celis" "$dir/celis"
chmod 755 "$dir/celis"

echo "Installed $("$dir/celis" --version) to $dir/celis."
[ -z "${CELIS_UPDATE:-}" ] || exit 0

line="export PATH=\"$dir:\$PATH\""
case ":$PATH:" in
*":$dir:"*) ;;
*)
	added=""
	for name in .profile .bash_profile .bashrc .zshrc; do
		case "$name" in
		.bash_profile) [ -f "$HOME/$name" ] || continue ;;
		.bashrc) [ -f "$HOME/$name" ] || [ "${SHELL:+${SHELL##*/}}" = bash ] || continue ;;
		.zshrc) [ -f "$HOME/$name" ] || [ "${SHELL:+${SHELL##*/}}" = zsh ] || continue ;;
		esac
		grep -qxF "$line" "$HOME/$name" 2>/dev/null || printf '\n%s\n' "$line" >>"$HOME/$name"
		added="$added ~/$name"
	done
	echo
	echo "Added $dir to PATH in$added. Open a new terminal, or run:"
	echo "  $line"
	;;
esac
echo
echo "Next, add the plugin to your agents:"
echo "  celis setup"
echo
echo "To use the Celis account you already have, give it its phrase:"
echo "  celis auth login"
