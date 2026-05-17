#!/usr/bin/env bash
# Empirical negative test for ralph/check-spec.sh.
#
# Demonstrates the tool would have caught the original marzullo spec
# bug (missing the Helly-1D precondition `correct_intervals_overlap`)
# at operator time, before any agent cycles were spent.
#
# Method:
#   1. Copy exercises/marzullo.rs and exercises/marzullo_witness.rs
#      into a tmpdir.
#   2. Strip the `correct_intervals_overlap(intervals@),` line from
#      the `pub fn marzullo` requires in both copies, simulating the
#      original (pre-Helly-1D) frozen spec.
#   3. Run `verus` directly on the modified witness.
#   4. Assert verus exits non-zero — the witness must fail to verify
#      because the existence lemma cannot derive the supported-point
#      claim without the Helly-1D precondition.
#
# A PASS here means the tool would have flagged the bug before the
# agent ever saw it.

set -u

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if ! command -v verus >/dev/null 2>&1; then
  echo "verus not on PATH; cannot run negative test" >&2
  exit 2
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Copy and strip. The sed pattern matches the line as it appears in
# the function's requires block; leading-comment forms are not
# touched.
sed '/^        correct_intervals_overlap(intervals@),$/d' \
  exercises/marzullo.rs > "$tmp/marzullo.rs"
sed '/^        correct_intervals_overlap(intervals@),$/d' \
  exercises/marzullo_witness.rs > "$tmp/marzullo_witness.rs"

# Quick sanity: both files must have *lost* exactly the one line.
removed_ex=$(diff exercises/marzullo.rs "$tmp/marzullo.rs" | grep -c '^<')
removed_w=$(diff exercises/marzullo_witness.rs "$tmp/marzullo_witness.rs" | grep -c '^<')
if [ "$removed_ex" != "1" ] || [ "$removed_w" != "1" ]; then
  echo "setup error: expected one line removed from each file" >&2
  echo "  exercise removed: $removed_ex" >&2
  echo "  witness removed:  $removed_w" >&2
  exit 2
fi

echo "Running verus on Helly-1D-stripped witness..."
if verus "$tmp/marzullo_witness.rs" --crate-type=lib >/dev/null 2>&1; then
  echo
  echo "FAIL: verus accepted the stripped witness." >&2
  echo "Expected verus to fail without correct_intervals_overlap." >&2
  echo "The negative test is meaningless — investigate." >&2
  exit 1
else
  echo
  echo "PASS: verus rejected the stripped witness (as expected)."
  echo "ralph/check-spec.sh would have flagged the original marzullo"
  echo "spec bug at operator time, before any agent cycles ran."
  exit 0
fi
