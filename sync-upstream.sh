#!/bin/bash
# Sync latest changes from lean-ctx upstream into betterctx-client
# Run this whenever yvgude pushes new changes:  bash sync-upstream.sh

set -e

echo "==> Fetching upstream (lean-ctx)..."
git fetch upstream

echo "==> Checking for new commits..."
BEHIND=$(git rev-list HEAD..upstream/main --count 2>/dev/null || echo "0")

if [ "$BEHIND" = "0" ]; then
  echo "✅ Already up to date with upstream. Nothing to do."
  exit 0
fi

echo "    $BEHIND new commit(s) from upstream."
echo ""
echo "==> Merging upstream/main..."
git merge upstream/main --no-edit -m "chore: merge upstream lean-ctx changes"

echo ""
echo "==> Re-applying betterctx rebranding..."
bash "$(dirname "$0")/../rebrand-inplace.sh" "$(pwd)"

echo ""
echo "==> Committing rebranding fixes..."
git add -A
git diff --cached --quiet || git commit -m "chore: rebrand after upstream sync"

echo ""
echo "==> Pushing to origin..."
git push origin main

echo ""
echo "✅ Sync complete. betterctx-client is up to date with lean-ctx upstream."
echo ""
echo "⚠️  Check cloud_client.rs for any new API endpoints:"
echo "   git diff HEAD~2 -- rust/src/cloud_client.rs"
