#!/usr/bin/env bash
# push_to_github.sh — Commit (if needed) and push dist_agent_lang to GitHub (user-facing only).
# Use for initial push and for ongoing updates: run after making changes to sync to GitHub.
#
# Usage: run from dist_agent_lang:
#   cd dist_agent_lang && ./scripts/push_to_github.sh
# Or from repo root:
#   ./dist_agent_lang/scripts/push_to_github.sh
#
# Optional: custom commit message
#   ./scripts/push_to_github.sh "Fix dependabot config"
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

# --- Use existing .gitignore (don't overwrite) ---
if [[ ! -f .gitignore ]]; then
  echo "Warning: .gitignore not found. Creating basic one..." >&2
  cat > .gitignore << 'GITIGNORE'
# Build and install
/target/
**/target/

# Mutation testing and logs (do not publish)
/mutants.out/
/mutants.out.old/
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
GITIGNORE
else
  echo "Using existing .gitignore (excludes development/project/testing docs and internal planning docs)"
fi

# --- Remove tracked files that are now ignored ---
if [[ -d .git ]]; then
  echo "Removing tracked files that match .gitignore patterns..."
  
  # List of files/directories to remove (from .gitignore)
  IGNORED_FILES=(
    "docs/PRODUCTION_ROADMAP.md"
    "docs/ADVANCED_SECURITY_BEST_PRACTICES.md"
    "docs/ADVANCED_SECURITY_BLOCKING_BEHAVIOR.md"
    "docs/ADVANCED_SECURITY_DESIGN.md"
    "docs/IMPLEMENTATION_SUMMARY.md"
    "docs/RUNTIME_TESTING_STATUS.md"
    "docs/TESTING_FRAMEWORK_PROPOSAL.md"
    "docs/TESTING_ATTRIBUTES.md"
    "docs/EXAMPLE_TESTING_GUIDE.md"
    "docs/MEV_FIX_EXAMPLE.md"
    "docs/MEV_PROTECTION_MANUAL.md"
    "docs/SECURE_FUNCTION_LEVEL_AND_MEV.md"
    "docs/SECURE_SCOPE.md"
    "docs/SEMANTIC_VALIDATION_FEATURE.md"
    "docs/REENTRANCY_CLARITY.md"
    "docs/READINESS_CHECKLIST.md"
    "docs/PARSE_COMMAND_SUMMARY.md"
    "docs/AI_ENHANCED_TOOLS_SUMMARY.md"
    "docs/USER_FACING_DOCS.md"
    "docs/ORACLE_DEVELOPMENT_README.md"
    "docs/XNFT_DYNAMIC_RWA_GUIDE.md"
    "docs/guides/PACKAGING_STRATEGY.md"
    "docs/guides/DECENTRALIZED_DISTRIBUTION_ALTERNATIVE.md"
    "docs/guides/PROJECT_PRIORITIES_TRACKER.md"
    "docs/guides/DEVELOPMENT_PHASES.md"
    "docs/guides/SEPARATION_INTEGRATION_PLAN.md"
    "docs/guides/ARCHITECTURE_SEPARATION.md"
    "docs/guides/SMART_CONTRACT_INTERFACE_SEPARATION.md"
    "docs/guides/LLM_ADOPTION_ANALYSIS.md"
    "docs/guides/GENERAL_PURPOSE_LANGUAGE_ANALYSIS.md"
    "docs/guides/PROJECT_STRUCTURE.md"
    "docs/guides/FIXES_SUMMARY.md"
    "docs/guides/BETA_RELEASE_SUMMARY.md"
    "docs/guides/AUDIT_REPORT.md"
    "docs/guides/RUNTIME_IMPLEMENTATION.md"
    "docs/guides/ORIGINAL_VISION.md"
    "docs/guides/SMART_CONTRACTS_WITH_DAL_REVIEW.md"
    "docs/guides/TXN_ATTRIBUTE_GUIDE.md"
  )
  
  REMOVED_COUNT=0
  for file in "${IGNORED_FILES[@]}"; do
    if git ls-files --error-unmatch "$file" &>/dev/null; then
      echo "  Removing from git: $file"
      git rm --cached "$file" 2>/dev/null || true
      ((REMOVED_COUNT++)) || true
    fi
  done
  
  # Also remove any files matching patterns (e.g., *_PLAN.md, *_STATUS.md, PHASE*.md)
  git ls-files | grep -E '(_PLAN\.md|_STATUS\.md|^PHASE.*\.md)$' | while read -r file; do
    if [[ -n "$file" ]]; then
      echo "  Removing from git: $file"
      git rm --cached "$file" 2>/dev/null || true
      ((REMOVED_COUNT++)) || true
    fi
  done
  
  if [[ $REMOVED_COUNT -gt 0 ]]; then
    echo "Removed $REMOVED_COUNT tracked file(s) that are now ignored."
  else
    echo "No tracked files to remove (they're already ignored or not tracked)."
  fi
fi

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

# --- Stage, commit (if needed), and push ---
git add -A
# Also ensure .gitignore is staged
git add -f .gitignore 2>/dev/null || true

if git diff --cached --quiet && git diff --quiet; then
  echo "Nothing to commit (working tree clean)."
else
  git status --short
  git commit -m "$COMMIT_MSG"
  echo "Committed."
fi

echo "Pushing to origin $BRANCH (okjason-source/dist_agent_lang)..."
git push -u origin "$BRANCH"

echo "Done. https://github.com/okjason-source/dist_agent_lang"
