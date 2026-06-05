#!/usr/bin/env bash
# gc-fleet.sh — Fleet-level garbage collection for SuperInstance repos
# ==============================================================================
# For repos that are Level-1 only (on GitHub, no clone needed),
# reclaim local storage by:
#   - Testing if the repo has been modified since clone
#   - If not: inform user it's safe to delete
#   - If user confirms: delete the clone
# ==============================================================================

set -euo pipefail

echo "=== FLEET GC — $(date -u) ==="
echo ""

WORKSPACE="${1:-$HOME/.openclaw/workspace}"

# Legacy repos that are Level-1 reference only
LEGACY_REPOS=(
    "Mycelium"
    "Spreader-tool"
    "egg"
    "neural-plato"
    "polln"
    "seed-oscillate"
    "sunset-ecosystem"
    "the-seed"
)

LEGACY_DIR="$WORKSPACE/pincher-legacy-mine"

for repo in "${LEGACY_REPOS[@]}"; do
    repo_path="$LEGACY_DIR/$repo"
    if [ -d "$repo_path" ]; then
        size=$(du -sh "$repo_path" 2>/dev/null | cut -f1)
        echo "$size $repo (L1 reference, no local compute needed)"
    fi
done
