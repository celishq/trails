#!/bin/sh
# Gives the project one landing page with a hero and a card that lifts on hover.
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# orto\n' > README.md
mkdir -p site
cat > site/index.html <<'HTML'
<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>Orto</title>
<style>
body { margin: 0; background: #14211B; color: #F2EDE4; font-family: Georgia, serif; }
.hero { padding: 96px 48px 48px; }
.hero h1 { font-size: 72px; font-weight: 900; margin: 0; }
.hero p { font-size: 20px; opacity: 0.7; max-width: 32ch; }
.cards { display: flex; gap: 24px; padding: 0 48px 96px; }
.card { width: 240px; padding: 24px; background: #1F3028; border-radius: 4px; transition: transform 240ms cubic-bezier(0.2, 0.8, 0.2, 1), box-shadow 240ms cubic-bezier(0.2, 0.8, 0.2, 1); }
.card:hover { transform: translateY(-6px); box-shadow: 0 12px 24px rgba(0, 0, 0, 0.4); }
</style>
</head>
<body>
<section class="hero">
<h1>Orto</h1>
<p>Vegetables from one farm, picked this morning, at your door tonight.</p>
</section>
<section class="cards">
<div class="card"><h2>Box</h2><p>Six kinds, chosen by the farmer.</p></div>
<div class="card"><h2>Herbs</h2><p>Cut an hour before they leave.</p></div>
<div class="card"><h2>Eggs</h2><p>From hens that walk the rows.</p></div>
</section>
</body>
</html>
HTML
