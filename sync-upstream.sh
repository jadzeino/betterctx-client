#!/bin/bash
# Sync latest changes from lean-ctx upstream into betterctx-client
# Run this whenever yvgude pushes new changes:  bash sync-upstream.sh

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
if ! git merge upstream/main --no-edit -m "chore: merge upstream lean-ctx v$UPSTREAM_VERSION" 2>&1; then
  echo ""
  echo "❌ MERGE CONFLICT detected."
  echo ""
  echo "   Files with conflicts:"
  git diff --name-only --diff-filter=U
  echo ""
  echo "   What to do:"
  echo "   1. Open each conflicting file in your editor"
  echo "   2. Look for <<<<<<< HEAD markers and resolve them"
  echo "      - Keep YOUR changes (betterctx branding etc)"
  echo "      - Take HIS new feature code"
  echo "   3. After resolving all files run:"
  echo "      git add -A"
  echo "      git commit -m 'chore: merge upstream lean-ctx v$UPSTREAM_VERSION'"
  echo "      bash sync-upstream.sh --skip-merge"
  echo ""
  echo "   Tip: Run 'git status' to see which files still need resolving."
  exit 1
fi

# ── Rebrand ───────────────────────────────────────────────────────────────────
echo ""
echo "==> Re-applying betterctx rebranding..."
bash "$REBRAND_SCRIPT" "$REPO_DIR"

# ── Version bump ──────────────────────────────────────────────────────────────
if [ "$VERSION_CHANGED" = true ]; then
  echo ""
  echo "==> Updating version to $UPSTREAM_VERSION in all package files..."

  sed -i '' "s/^version = \"$LOCAL_VERSION\"/version = \"$UPSTREAM_VERSION\"/" rust/Cargo.toml

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
if ! git push origin main 2>&1; then
  echo ""
  echo "❌ Push failed. Try:"
  echo "   ssh-add --apple-use-keychain ~/.ssh/id_ed25519"
  echo "   git push origin main"
  exit 1
fi

# ── Tag new version ───────────────────────────────────────────────────────────
if [ "$VERSION_CHANGED" = true ]; then
  echo ""
  echo "==> Creating release tag v$UPSTREAM_VERSION..."
  git tag "v$UPSTREAM_VERSION"
  if ! git push origin "v$UPSTREAM_VERSION" 2>&1; then
    echo "❌ Tag push failed. Try:"
    echo "   ssh-add --apple-use-keychain ~/.ssh/id_ed25519"
    echo "   git push origin v$UPSTREAM_VERSION"
    exit 1
  fi
  echo ""
  echo "🚀 Tag v$UPSTREAM_VERSION pushed — GitHub Actions will now:"
  echo "   • Build binaries for all 6 platforms"
  echo "   • Create GitHub Release"
  echo "   • Publish to crates.io"
  echo "   • Publish to npm (better-ctx-bin + pi-better-ctx)"
  echo "   • Update Homebrew formula"
fi

echo ""
echo "✅ Sync complete."
if [ -n "$API_DIFF" ]; then
  echo ""
  echo "⚠️  ACTION REQUIRED: API changes detected in cloud_client.rs"
  echo "   Review and update your betterctx-api backend before users upgrade."
fi
