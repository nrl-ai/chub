#!/usr/bin/env bash
# Usage: ./scripts/set-version.sh <version>
# Example: ./scripts/set-version.sh 0.2.0
#
# Updates the package version in all language ecosystems:
#   - Rust:   Cargo.toml (workspace version + chub-core dep)
#   - npm:    npm/chub/package.json + 5 platform packages
#   - Python: python/pyproject.toml + python/chub/__init__.py

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <version>" >&2
  echo "Example: $0 0.2.0" >&2
  exit 1
fi

NEW_VERSION="$1"

# Validate semver-ish format
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
  echo "Error: version must be semver (e.g. 1.2.3 or 1.2.3-beta.1)" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Detect current version from workspace Cargo.toml
CURRENT_VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)

if [ -z "$CURRENT_VERSION" ]; then
  echo "Error: could not detect current version from Cargo.toml" >&2
  exit 1
fi

if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
  echo "Already at version $NEW_VERSION" >&2
  exit 0
fi

echo "Bumping $CURRENT_VERSION → $NEW_VERSION"

# --- Rust ---
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$ROOT/Cargo.toml"
sed -i "s/chub-core = { path = \"crates\/chub-core\", version = \"$CURRENT_VERSION\"/chub-core = { path = \"crates\/chub-core\", version = \"$NEW_VERSION\"/" "$ROOT/Cargo.toml"
sed -i "s/chub-cli = { path = \"crates\/chub-cli\", version = \"$CURRENT_VERSION\"/chub-cli = { path = \"crates\/chub-cli\", version = \"$NEW_VERSION\"/" "$ROOT/Cargo.toml"
echo "  ✓ Cargo.toml"

# --- npm ---
for pkg in "$ROOT"/npm/*/package.json; do
  sed -i "s/\"$CURRENT_VERSION\"/\"$NEW_VERSION\"/g" "$pkg"
  echo "  ✓ $(basename "$(dirname "$pkg")")/package.json"
done

# --- Python ---
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$ROOT/python/pyproject.toml"
echo "  ✓ python/pyproject.toml"

sed -i "s/__version__ = \"$CURRENT_VERSION\"/__version__ = \"$NEW_VERSION\"/" "$ROOT/python/chub/__init__.py"
echo "  ✓ python/chub/__init__.py"

echo ""
echo "Done. Version is now $NEW_VERSION across all packages."
echo "Run 'cargo check' to verify Cargo.lock updates."
