#!/usr/bin/env bash
# Symlink scripts/git-hooks/* into .git/hooks/. Idempotent.
set -u
cd "$(dirname "$0")/.."

REPO_ROOT=$(git rev-parse --show-toplevel)
SRC_DIR="$REPO_ROOT/scripts/git-hooks"
DST_DIR="$REPO_ROOT/.git/hooks"

[ -d "$SRC_DIR" ] || { echo "missing $SRC_DIR"; exit 2; }
mkdir -p "$DST_DIR"

for hook in "$SRC_DIR"/*; do
  name=$(basename "$hook")
  dst="$DST_DIR/$name"
  chmod +x "$hook"

  if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$hook" ]; then
    echo "ok    : $name (already symlinked)"
    continue
  fi
  if [ -e "$dst" ]; then
    bak="${dst}.bak.$(date +%Y%m%d%H%M%S)"
    mv "$dst" "$bak"
    echo "backup: $name → $bak"
  fi
  ln -s "$hook" "$dst"
  echo "linked: $name → $hook"
done

echo
echo "Hooks installed. Verify by trying a forbidden commit:"
echo "  echo 'should fail' > /tmp/oops.txt && git add /tmp/oops.txt && git commit -m test"
