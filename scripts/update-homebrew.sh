#!/usr/bin/env bash
#
# Update the Homebrew formula with correct SHA-256 hashes.
#
# Usage:
#   ./scripts/update-homebrew.sh <version>
#
# Prerequisites:
#   - GitHub release v<version> must exist with binaries uploaded
#   - curl must be available
#
# This script:
#   1. Downloads each platform binary from the GitHub release
#   2. Computes SHA-256 hashes
#   3. Updates homebrew/chub.rb with the correct hashes and version
#   4. Copies the formula to the homebrew-tap repo (if present)

set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
REPO="nrl-ai/chub"
FORMULA="homebrew/chub.rb"
TAP_REPO="../homebrew-tap"

BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

PLATFORMS=(
  "chub-darwin-arm64"
  "chub-darwin-x64"
  "chub-linux-arm64"
  "chub-linux-x64"
)

PLACEHOLDERS=(
  "PLACEHOLDER_DARWIN_ARM64"
  "PLACEHOLDER_DARWIN_X64"
  "PLACEHOLDER_LINUX_ARM64"
  "PLACEHOLDER_LINUX_X64"
)

echo "Updating Homebrew formula for v${VERSION}"
echo ""

# Start with a clean copy
cp "${FORMULA}" "${FORMULA}.tmp"

# Update version
sed -i "s/version \".*\"/version \"${VERSION}\"/" "${FORMULA}.tmp"

for i in "${!PLATFORMS[@]}"; do
  platform="${PLATFORMS[$i]}"
  placeholder="${PLACEHOLDERS[$i]}"
  url="${BASE_URL}/${platform}"

  echo -n "  Fetching ${platform}... "
  sha=$(curl -fsSL "${url}" | sha256sum | awk '{print $1}')
  echo "${sha}"

  sed -i "s/${placeholder}/${sha}/" "${FORMULA}.tmp"
  # Also replace any existing hash for this platform
  # (for subsequent runs where placeholders are already replaced)
done

mv "${FORMULA}.tmp" "${FORMULA}"
echo ""
echo "Updated ${FORMULA}"

# Copy to tap repo if it exists
if [ -d "${TAP_REPO}" ]; then
  mkdir -p "${TAP_REPO}/Formula"
  cp "${FORMULA}" "${TAP_REPO}/Formula/chub.rb"
  echo "Copied to ${TAP_REPO}/Formula/chub.rb"
  echo ""
  echo "Next steps:"
  echo "  cd ${TAP_REPO}"
  echo "  git add Formula/chub.rb"
  echo "  git commit -m 'chub ${VERSION}'"
  echo "  git push"
else
  echo ""
  echo "Tap repo not found at ${TAP_REPO}"
  echo ""
  echo "To set up the tap:"
  echo "  1. Create repo: gh repo create nrl-ai/homebrew-tap --public"
  echo "  2. Clone it:    git clone https://github.com/nrl-ai/homebrew-tap.git ${TAP_REPO}"
  echo "  3. Copy formula: mkdir -p ${TAP_REPO}/Formula && cp ${FORMULA} ${TAP_REPO}/Formula/chub.rb"
  echo "  4. Push:        cd ${TAP_REPO} && git add . && git commit -m 'chub ${VERSION}' && git push"
  echo ""
  echo "Users install with: brew install nrl-ai/tap/chub"
fi
