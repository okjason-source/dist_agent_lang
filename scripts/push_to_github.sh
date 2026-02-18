#!/usr/bin/env bash
# push_to_github.sh — Commit and push dist_agent_lang to GitHub (user-facing only).
#
# Usage: run from dist_agent_lang:
#   cd dist_agent_lang && ./scripts/push_to_github.sh
# Or from repo root:
#   ./dist_agent_lang/scripts/push_to_github.sh
#
# Remote: https://github.com/okjason-source/dist_agent_lang
# Excludes: docs/development, docs/project, docs/beta, docs/testing,
#           target/, mutants.out/, logs/, and other dev/test artifacts.

set -e

# Ensure we are in dist_agent_lang (parent of scripts/)
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ ! -f Cargo.toml ]] || [[ "$(basename "$ROOT")" != "dist_agent_lang" ]]; then
  echo "Error: Script must be run from dist_agent_lang (or with dist_agent_lang/scripts/push_to_github.sh)." >&2
  exit 1
fi

REPO_URL="https://github.com/okjason-source/dist_agent_lang.git"
BRANCH="${GIT_BRANCH:-main}"
COMMIT_MSG="${1:-Sync dist_agent_lang: user-facing package and docs}"

# --- .gitignore for publish (exclude development/project/testing) ---
cat > .gitignore << 'GITIGNORE'
# Build and install
/target/
**/target/

# Mutation testing and logs (do not publish)
/mutants.out/
/logs/
*.log

# Development / project / testing documentation (do not publish)
/docs/development/
/docs/project/
/docs/beta/
/docs/testing/

# Root-level internal planning docs (optional)
*_PLAN.md
*_STATUS.md
PHASE*.md

# IDE and OS
.idea/
.vscode/
.DS_Store
*.swp
*~

# Fuzz artifacts (keep corpus if you want; uncomment to exclude)
# /fuzz/corpus/
# /fuzz/artifacts/
GITIGNORE

echo "Using .gitignore (publish): excludes docs/development, docs/project, docs/beta, docs/testing, target/, mutants.out/, logs/"

# --- Git init (no-op if already a repo) ---
if [[ ! -d .git ]]; then
  git init
  echo "Initialized git in $ROOT"
else
  echo "Existing .git found in $ROOT"
fi

# --- Remote ---
if ! git remote get-url origin &>/dev/null; then
  git remote add origin "$REPO_URL"
  echo "Added remote origin: $REPO_URL"
else
  echo "Remote origin already set: $(git remote get-url origin)"
fi

# --- Branch ---
git branch -M "$BRANCH" 2>/dev/null || true

# --- Stage and commit ---
git add -A
if git diff --cached --quiet; then
  echo "Nothing to commit (working tree clean after .gitignore)."
  exit 0
fi
git status --short
git commit -m "$COMMIT_MSG"

# --- Push ---
echo "Pushing to origin $BRANCH (okjason-source/dist_agent_lang)..."
git push -u origin "$BRANCH"

echo "Done. Open: https://github.com/okjason-source/dist_agent_lang"
