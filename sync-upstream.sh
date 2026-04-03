#!/bin/bash
# Sync latest changes from lean-ctx upstream into betterctx-client
# Run this whenever yvgude pushes new changes:  bash sync-upstream.sh

set -e

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
REBRAND_SCRIPT="/Users/azeno/Desktop/wayToGO/rebrand-inplace.sh"

cd "$REPO_DIR"

echo "==> Fetching upstream (lean-ctx)..."
git fetch upstream

echo ""
echo "==> Checking for new commits..."
BEHIND=$(git rev-list HEAD..upstream/main --count 2>/dev/null || echo "0")

if [ "$BEHIND" = "0" ]; then
  echo "✅ Already up to date with upstream. Nothing to do."
  exit 0
fi

echo "    $BEHIND new commit(s) from upstream."

# ── Version check ─────────────────────────────────────────────────────────────
UPSTREAM_VERSION=$(git show upstream/main:rust/Cargo.toml | grep '^version' | head -1 | sed 's/version = "\(.*\)"/\1/')
LOCAL_VERSION=$(grep '^version' rust/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo ""
echo "==> Version comparison:"
echo "    Current (yours): v$LOCAL_VERSION"
echo "    Upstream (his):  v$UPSTREAM_VERSION"

VERSION_CHANGED=false
if [ "$UPSTREAM_VERSION" != "$LOCAL_VERSION" ]; then
  VERSION_CHANGED=true
  echo "    ⚡ New version available: v$LOCAL_VERSION → v$UPSTREAM_VERSION"
else
  echo "    Same version, code-only changes."
fi

# ── API check ─────────────────────────────────────────────────────────────────
echo ""
echo "==> Checking for API endpoint changes in cloud_client.rs..."
API_DIFF=$(git diff HEAD upstream/main -- rust/src/cloud_client.rs | grep '^[+-].*\/api\/' | grep -v '^---\|^+++' || true)
if [ -n "$API_DIFF" ]; then
  echo "  ⚠️  WARNING: cloud_client.rs API endpoints changed!"
  echo "  You may need to update betterctx-api backend."
  echo "  Changed lines:"
  echo "$API_DIFF"
else
  echo "    ✅ No API endpoint changes — backend is still compatible."
fi

# ── Merge ─────────────────────────────────────────────────────────────────────
echo ""
echo "==> Merging upstream/main..."
git merge upstream/main --no-edit -m "chore: merge upstream lean-ctx v$UPSTREAM_VERSION"

# ── Rebrand ───────────────────────────────────────────────────────────────────
echo ""
echo "==> Re-applying betterctx rebranding..."
bash "$REBRAND_SCRIPT" "$REPO_DIR"

# ── Version bump in Cargo.toml and package.json files ─────────────────────────
if [ "$VERSION_CHANGED" = true ]; then
  echo ""
  echo "==> Updating version to $UPSTREAM_VERSION in all package files..."

  # Cargo.toml
  sed -i '' "s/^version = \"$LOCAL_VERSION\"/version = \"$UPSTREAM_VERSION\"/" rust/Cargo.toml

  # npm packages
  for pkg in packages/better-ctx-bin packages/pi-better-ctx; do
    if [ -f "$pkg/package.json" ]; then
      sed -i '' "s/\"version\": \"$LOCAL_VERSION\"/\"version\": \"$UPSTREAM_VERSION\"/" "$pkg/package.json"
      echo "    Updated $pkg/package.json → $UPSTREAM_VERSION"
    fi
  done
fi

# ── Commit ────────────────────────────────────────────────────────────────────
echo ""
echo "==> Committing changes..."
git add -A
git diff --cached --quiet || git commit -m "chore: sync upstream lean-ctx v$UPSTREAM_VERSION + rebrand"

# ── Push ──────────────────────────────────────────────────────────────────────
echo ""
echo "==> Pushing to origin..."
git push origin main

# ── Tag new version ───────────────────────────────────────────────────────────
if [ "$VERSION_CHANGED" = true ]; then
  echo ""
  echo "==> New version detected. Creating release tag v$UPSTREAM_VERSION..."
  git tag "v$UPSTREAM_VERSION"
  git push origin "v$UPSTREAM_VERSION"
  echo ""
  echo "🚀 Tag v$UPSTREAM_VERSION pushed — GitHub Actions will now:"
  echo "   • Build binaries for all 6 platforms"
  echo "   • Create GitHub Release"
  echo "   • Publish to crates.io"
  echo "   • Publish to npm (better-ctx-bin + pi-better-ctx)"
fi

echo ""
echo "✅ Sync complete."
if [ -n "$API_DIFF" ]; then
  echo ""
  echo "⚠️  ACTION REQUIRED: API changes detected in cloud_client.rs"
  echo "   Review and update your betterctx-api backend before releasing."
fi
