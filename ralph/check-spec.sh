#!/usr/bin/env bash
# Pre-spec verification: confirm a frozen spec admits a model.
#
# Usage:
#   ./ralph/check-spec.sh <exercise>
#
# Two layouts are supported:
#
#   Single-file:
#     exercises/<name>.rs                    # spec (frozen)
#     exercises/<name>_witness.rs            # operator's reference impl
#
#   Multi-file:
#     exercises/<name>/main.rs               # entry, declares submodules
#     exercises/<name>/<module>.rs           # per-module spec files
#     exercises/<name>_witness/main.rs       # operator's reference impl
#     exercises/<name>_witness/<module>.rs   # operator's reference impl
#
# In either case the tool runs verus on the witness, applies the
# cheat-token filter, and verifies that every `requires`/`ensures`
# clause body in the exercise file(s) appears verbatim in the
# corresponding witness file(s).
#
# Exit codes:
#   0  spec is satisfiable
#   1  spec failed verification (verus error)
#   2  witness missing or input error
#   3  witness contains cheat tokens

set -u

if [ $# -lt 1 ]; then
  echo "usage: $0 <exercise>" >&2
  exit 2
fi

EXERCISE="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# --- Detect layout -----------------------------------------------------------

EXFILE_SINGLE="exercises/${EXERCISE}.rs"
WITNESS_SINGLE="exercises/${EXERCISE}_witness.rs"
EXDIR_MULTI="exercises/${EXERCISE}"
WITNESS_MULTI="exercises/${EXERCISE}_witness"

if [ -d "$EXDIR_MULTI" ] && [ -d "$WITNESS_MULTI" ]; then
  LAYOUT="multi"
elif [ -f "$EXFILE_SINGLE" ] && [ -f "$WITNESS_SINGLE" ]; then
  LAYOUT="single"
elif [ -f "$EXFILE_SINGLE" ] && [ ! -f "$WITNESS_SINGLE" ]; then
  echo "error: witness file not found: $WITNESS_SINGLE" >&2
  echo
  echo "Before tagging spec-frozen-${EXERCISE}, write a reference" >&2
  echo "implementation in $WITNESS_SINGLE that satisfies the same spec" >&2
  echo "as $EXFILE_SINGLE." >&2
  exit 2
elif [ -d "$EXDIR_MULTI" ] && [ ! -d "$WITNESS_MULTI" ]; then
  echo "error: witness directory not found: $WITNESS_MULTI" >&2
  echo
  echo "Before tagging spec-frozen-${EXERCISE}, write a reference" >&2
  echo "implementation under $WITNESS_MULTI/ mirroring the layout of" >&2
  echo "$EXDIR_MULTI/." >&2
  exit 2
else
  echo "error: no exercise found at $EXFILE_SINGLE or $EXDIR_MULTI" >&2
  exit 2
fi

# --- File lists for each mode ------------------------------------------------

if [ "$LAYOUT" = "single" ]; then
  EXERCISE_FILES=("$EXFILE_SINGLE")
  WITNESS_FILES=("$WITNESS_SINGLE")
  VERUS_ENTRY="$WITNESS_SINGLE"
else
  EXERCISE_FILES=()
  WITNESS_FILES=()
  while IFS= read -r f; do EXERCISE_FILES+=("$f"); done \
    < <(find "$EXDIR_MULTI" -maxdepth 1 -name '*.rs' -type f | sort)
  while IFS= read -r f; do WITNESS_FILES+=("$f"); done \
    < <(find "$WITNESS_MULTI" -maxdepth 1 -name '*.rs' -type f | sort)
  VERUS_ENTRY="$WITNESS_MULTI/main.rs"

  if [ ! -f "$VERUS_ENTRY" ]; then
    echo "error: multi-file witness needs $VERUS_ENTRY (entry point)" >&2
    exit 2
  fi

  # Pair-check: every file in EXERCISE_FILES has a same-named file in WITNESS_FILES.
  for ex in "${EXERCISE_FILES[@]}"; do
    base=$(basename "$ex")
    if [ ! -f "$WITNESS_MULTI/$base" ]; then
      echo "error: witness is missing module file: $WITNESS_MULTI/$base" >&2
      echo "       (exercise has $ex; witness must mirror)" >&2
      exit 2
    fi
  done
fi

# --- 1. Cheat-token check on every witness file ------------------------------

CHEAT_RC=0
check_cheat_file() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if grep -nE "$pattern" "$file" >/dev/null; then
    echo "  [cheat] $file contains '$label':" >&2
    grep -nE "$pattern" "$file" | sed 's/^/    /' >&2
    CHEAT_RC=1
  fi
}
for wf in "${WITNESS_FILES[@]}"; do
  check_cheat_file "$wf" '\bassume[[:space:]]*\('     'assume(...)'
  check_cheat_file "$wf" 'external_body'              'external_body'
  check_cheat_file "$wf" 'unreachable!\(\)'           'unreachable!()'
  check_cheat_file "$wf" '\bpanic!\('                 'panic!(...)'
  check_cheat_file "$wf" 'assume_specification'       'assume_specification'
done

if [ $CHEAT_RC -ne 0 ]; then
  echo >&2
  echo "Witness uses verification-bypass tokens; rewrite without them." >&2
  exit 3
fi

# --- 2. Spec-line presence check ---------------------------------------------

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

SPEC_MISMATCH=0
for ex in "${EXERCISE_FILES[@]}"; do
  if [ "$LAYOUT" = "single" ]; then
    paired="$WITNESS_SINGLE"
  else
    paired="$WITNESS_MULTI/$(basename "$ex")"
  fi

  ex_spec=$(extract_spec_clauses "$ex")
  [ -z "$ex_spec" ] && continue
  paired_body=$(cat "$paired")

  while IFS= read -r line; do
    [ -z "$line" ] && continue
    if ! echo "$paired_body" | grep -qFx "$line"; then
      if [ $SPEC_MISMATCH -eq 0 ]; then
        echo "  [spec] witness is missing spec lines from $ex:" >&2
        SPEC_MISMATCH=1
      fi
      echo "    (in $paired) $line" >&2
    fi
  done <<< "$ex_spec"
done

if [ $SPEC_MISMATCH -ne 0 ]; then
  echo >&2
  echo "The witness must carry the same requires/ensures clauses as" >&2
  echo "the exercise file(s). Fix the witness to match, then re-run." >&2
  exit 2
fi

# --- 3. Verus verification of the witness ------------------------------------

echo "Verifying witness: $VERUS_ENTRY"
if verus "$VERUS_ENTRY" --crate-type=lib; then
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
