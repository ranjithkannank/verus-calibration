#!/usr/bin/env bash
# Run every exercise through the Ralph loop, in order. Stops on the first
# blocker — a blocked exercise is a data point worth examining before
# continuing.
#
# Usage:
#   ./ralph/run-all.sh           # actual run
#   ./ralph/run-all.sh --dry-run # print state transitions only

set -u

DRY_RUN_FLAG=""
if [ "${1:-}" = "--dry-run" ]; then
  DRY_RUN_FLAG="--dry-run"
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

EXERCISES=(binary_search bounded_log quorum_count quorum_cert)

for ex in "${EXERCISES[@]}"; do
  echo
  echo "##############################"
  echo "# $ex"
  echo "##############################"
  if ! ./ralph/run-exercise.sh "$ex" $DRY_RUN_FLAG; then
    echo
    echo "Exercise $ex did not complete cleanly. Stopping the sweep."
    echo "Inspect logs/$ex/ and decide whether to skip or re-run before continuing."
    exit 1
  fi
done

echo
echo "All exercises DONE. Time to fill writeup/results_template.md."
