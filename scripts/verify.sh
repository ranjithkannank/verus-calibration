#!/usr/bin/env bash
# Verify every exercise. Exit non-zero if any fails.
set -u

cd "$(dirname "$0")/.."

EXERCISES=(
  exercises/binary_search.rs
  exercises/bounded_log.rs
  exercises/quorum_count.rs
)

fail=0
for ex in "${EXERCISES[@]}"; do
  printf '\n=== %s ===\n' "$ex"
  if verus "$ex" --crate-type=lib; then
    printf '  PASS\n'
  else
    printf '  FAIL\n'
    fail=1
  fi
done

exit "$fail"
