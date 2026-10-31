#!/bin/sh
# Lays out a local release of the celis on PATH, a stub claude that logs its arguments, and install.md
# with its download line pointed at the local release.
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
root="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p release/latest/download bin
cp "$(command -v celis)" celis
name="celis-$(uname -m | sed 's/arm64/aarch64/')-$([ "$(uname -s)" = Darwin ] && echo apple-darwin || echo unknown-linux-musl).tar.gz"
tar -czf "release/latest/download/$name" celis
rm celis
(cd release/latest/download && shasum -a 256 "$name" > "$name.sha256")
cp "$root/install.sh" release/install.sh
printf '#!/bin/sh\necho "claude $*" >> "%s/claude.log"\necho ok\n' "$PWD" > bin/claude
chmod +x bin/claude
sed "s|curl -fsSL https://celis.ai/install.sh \| sh|CELIS_BASE=file://$PWD/release sh $PWD/release/install.sh|" "$root/install.md" > install.md
