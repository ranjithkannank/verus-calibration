#!/usr/bin/env bash
# Pre-spec verification: confirm a frozen spec admits a model.
#
# Usage:
#   ./ralph/check-spec.sh <exercise>
#
# Contract: a "witness" file exercises/<name>_witness.rs exists,
# contains a reference implementation of the same spec as
# exercises/<name>.rs, and verifies under Verus with no cheat tokens.
# If it does, the spec is provably satisfiable. If verus fails, the
# spec is either unprovable or the witness is wrong — either way the
# operator must fix something before tagging spec-frozen-<name>.
#
# This catches two classes of bug the agent loop cannot:
#   1. The spec is logically unsatisfiable (e.g. marzullo's original
#      frozen spec, missing the Helly-1D precondition — no algorithm
#      can verify against that postcondition).
#   2. The spec uses syntax that no longer compiles (e.g. bounded_log's
#      pre-final(self) syntax under newer Verus releases).
#
# Both cost agent cycles when caught downstream. This tool catches
# them at operator time.
#
# Exit codes:
#   0  spec is satisfiable (witness verifies, no cheat tokens)
#   1  spec failed verification (verus error)
#   2  witness file missing or input error
#   3  witness contains cheat tokens

set -u

if [ $# -lt 1 ]; then
  echo "usage: $0 <exercise>" >&2
  exit 2
fi

EXERCISE="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

EXFILE="exercises/${EXERCISE}.rs"
WITNESS="exercises/${EXERCISE}_witness.rs"

if [ ! -f "$EXFILE" ]; then
  echo "error: exercise file not found: $EXFILE" >&2
  exit 2
fi

if [ ! -f "$WITNESS" ]; then
  echo "error: witness file not found: $WITNESS" >&2
  echo
  echo "Before tagging spec-frozen-${EXERCISE}, write a reference" >&2
  echo "implementation in $WITNESS that satisfies the same spec as" >&2
  echo "$EXFILE. The witness proves the frozen spec admits a model." >&2
  exit 2
fi

# --- 1. Cheat-token check on the witness -------------------------------------
#
# Mirrors the pre-commit hook's cheat-token logic but applies to the
# witness file's full body, not just the staged diff. A cheat in the
# witness would silently invalidate the satisfiability claim.

CHEAT_RC=0
check_cheat() {
  local pattern="$1"
  local label="$2"
  if grep -nE "$pattern" "$WITNESS" >/dev/null; then
    echo "  [cheat] witness contains '$label':" >&2
    grep -nE "$pattern" "$WITNESS" | sed 's/^/    /' >&2
    CHEAT_RC=1
  fi
}
check_cheat '\bassume[[:space:]]*\('     'assume(...)'
check_cheat 'external_body'              'external_body'
check_cheat 'unreachable!\(\)'           'unreachable!()'
check_cheat '\bpanic!\('                 'panic!(...)'
check_cheat 'assume_specification'       'assume_specification'

if [ $CHEAT_RC -ne 0 ]; then
  echo >&2
  echo "Witness uses verification-bypass tokens; rewrite without them." >&2
  exit 3
fi

# --- 2. Spec-line presence check ---------------------------------------------
#
# Every requires/ensures clause body in the exercise file must also
# appear verbatim in the witness. We deliberately re-use the same
# indentation-tracking awk from the pre-commit hook so the checks
# stay consistent.

extract_spec_clauses() {
  awk '
    function indent_of(s,    i, n, c) {
      n = length(s)
      for (i = 1; i <= n; i++) {
        c = substr(s, i, 1)
        if (c != " " && c != "\t") return i - 1
      }
      return n
    }
    {
      ind = indent_of($0)
      first = $0; sub(/^[[:space:]]+/, "", first)
      if (in_clause && ind > clause_indent) { print; next }
      in_clause = 0
      if (match(first, /^(requires|ensures)([^a-zA-Z0-9_]|$)/)) {
        print
        in_clause = 1
        clause_indent = ind
      }
    }
  ' "$1"
}

EX_SPEC=$(extract_spec_clauses "$EXFILE")
WITNESS_BODY=$(cat "$WITNESS")
SPEC_MISMATCH=0

while IFS= read -r line; do
  [ -z "$line" ] && continue
  if ! echo "$WITNESS_BODY" | grep -qFx "$line"; then
    if [ $SPEC_MISMATCH -eq 0 ]; then
      echo "  [spec] witness is missing spec lines from $EXFILE:" >&2
      SPEC_MISMATCH=1
    fi
    echo "    $line" >&2
  fi
done <<< "$EX_SPEC"

if [ $SPEC_MISMATCH -ne 0 ]; then
  echo >&2
  echo "The witness must carry the same requires/ensures clauses as" >&2
  echo "the exercise file. Fix the witness to match, then re-run." >&2
  exit 2
fi

# --- 3. Verus verification of the witness ------------------------------------

echo "Verifying witness: $WITNESS"
if verus "$WITNESS" --crate-type=lib; then
  echo
  echo "OK: spec is satisfiable. Witness verifies, no cheat tokens."
  echo "You may now tag spec-frozen-${EXERCISE} and start the agent loop."
  exit 0
else
  rc=$?
  echo
  echo "FAIL: witness did not verify (verus exit $rc)." >&2
  echo "Either the witness implementation is wrong, or the frozen" >&2
  echo "spec is unprovable. If the latter, fix the spec BEFORE" >&2
  echo "tagging spec-frozen-${EXERCISE} — the agent loop cannot." >&2
  exit 1
fi
