#!/usr/bin/env bash
# Unit test for the pre-commit hook's spec-preservation logic.
#
# Builds an in-process pair of "frozen" and "staged" file fixtures, runs the
# same extraction-and-verbatim-match logic the hook uses, and confirms that
# body-line modifications (the gap the old hook missed) are now caught.

set -u

pass=0
fail=0

# Inlined extractor — keep in sync with scripts/git-hooks/pre-commit.
extract_clauses() {
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
  '
}

# Run the verbatim check. Echoes "PASS" if every extracted clause line from
# the frozen file appears verbatim in the staged content, else "FAIL".
verify_preservation() {
  local frozen_path="$1" staged_path="$2"
  local frozen staged_content
  frozen=$(extract_clauses < "$frozen_path")
  staged_content=$(cat "$staged_path")
  local missing=0
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    if ! echo "$staged_content" | grep -qFx "$line"; then
      missing=$((missing + 1))
    fi
  done <<< "$frozen"
  if [ "$missing" -eq 0 ]; then echo PASS; else echo "FAIL ($missing missing)"; fi
}

run_scenario() {
  local label="$1" expected="$2" frozen="$3" staged="$4"
  local fdir
  fdir=$(mktemp -d)
  printf '%s\n' "$frozen" > "$fdir/frozen.rs"
  printf '%s\n' "$staged" > "$fdir/staged.rs"
  local actual
  actual=$(verify_preservation "$fdir/frozen.rs" "$fdir/staged.rs")
  rm -rf "$fdir"
  if [ "$actual" = "$expected" ]; then
    printf '  [%-52s] PASS  (result=%s)\n' "$label" "$actual"
    pass=$((pass + 1))
  else
    printf '  [%-52s] FAIL  expected=%s got=%s\n' "$label" "$expected" "$actual"
    fail=$((fail + 1))
  fi
}

echo
echo "pre-commit hook spec-preservation tests"
echo "========================================"

# --- Scenario 1: bounded_log v1 violation (body-line rewrite) -----------------
# This is the actual regression the new check is designed to catch.

FROZEN_BOUNDED='pub fn append(&mut self, msg: Message) -> (result: Result<(), ()>)
    requires old(self).well_formed(),
    ensures
        self.well_formed(),
        self.capacity() == old(self).capacity(),
        result.is_ok() ==> {
            &&& self.view().len() == old(self).view().len() + 1
        },
{
    // body
}'

STAGED_BOUNDED_REWRITE='pub fn append(&mut self, msg: Message) -> (result: Result<(), ()>)
    requires old(self).well_formed(),
    ensures
        final(self).well_formed(),
        final(self).capacity() == old(self).capacity(),
        result.is_ok() ==> {
            &&& final(self).view().len() == old(self).view().len() + 1
        },
{
    // body
}'

run_scenario "body-line rewrite (self → final(self))" "FAIL (3 missing)" \
  "$FROZEN_BOUNDED" "$STAGED_BOUNDED_REWRITE"

# --- Scenario 2: keyword-line rewrite (the original old-hook test case) -------

STAGED_BOUNDED_NO_REQUIRES='pub fn append(&mut self, msg: Message) -> (result: Result<(), ()>)
    ensures
        self.well_formed(),
        self.capacity() == old(self).capacity(),
        result.is_ok() ==> {
            &&& self.view().len() == old(self).view().len() + 1
        },
{
    // body
}'

run_scenario "keyword line removed (requires gone)" "FAIL (1 missing)" \
  "$FROZEN_BOUNDED" "$STAGED_BOUNDED_NO_REQUIRES"

# --- Scenario 3: legitimate body preserved + impl added (should PASS) ---------

STAGED_BOUNDED_IMPL='pub fn append(&mut self, msg: Message) -> (result: Result<(), ()>)
    requires old(self).well_formed(),
    ensures
        self.well_formed(),
        self.capacity() == old(self).capacity(),
        result.is_ok() ==> {
            &&& self.view().len() == old(self).view().len() + 1
        },
{
    self.msgs.push(msg);
    assert(self.msgs@.len() == old(self).msgs@.len() + 1);
    Ok(())
}'

run_scenario "frozen lines preserved, body implemented" "PASS" \
  "$FROZEN_BOUNDED" "$STAGED_BOUNDED_IMPL"

# --- Scenario 4: cosmetic whitespace change (intentionally rejected) ----------

STAGED_BOUNDED_REINDENT='pub fn append(&mut self, msg: Message) -> (result: Result<(), ()>)
    requires old(self).well_formed(),
    ensures
         self.well_formed(),
         self.capacity() == old(self).capacity(),
        result.is_ok() ==> {
            &&& self.view().len() == old(self).view().len() + 1
        },
{
    // body
}'

run_scenario "cosmetic reindent of body lines" "FAIL (2 missing)" \
  "$FROZEN_BOUNDED" "$STAGED_BOUNDED_REINDENT"

# --- Scenario 5: new helper spec fn added (should still PASS) -----------------
# Adding a NEW spec fn ABOVE the function does not touch any existing
# requires/ensures clause.

FROZEN_WITH_SPEC='pub open spec fn is_sorted(s: Seq<i64>) -> bool {
    forall|i: int, j: int| 0 <= i <= j < s.len() ==> s[i] <= s[j]
}

pub fn binary_search(v: &Vec<i64>, target: i64) -> (result: Option<usize>)
    requires
        is_sorted(v@),
    ensures
        result.is_some() ==> v@[result.unwrap() as int] == target,
{
    // body
}'

STAGED_WITH_NEW_HELPER='pub open spec fn is_sorted(s: Seq<i64>) -> bool {
    forall|i: int, j: int| 0 <= i <= j < s.len() ==> s[i] <= s[j]
}

pub open spec fn is_in_range(s: Seq<i64>, lo: int, hi: int) -> bool {
    forall|i: int| lo <= i < hi ==> 0 <= i < s.len()
}

pub fn binary_search(v: &Vec<i64>, target: i64) -> (result: Option<usize>)
    requires
        is_sorted(v@),
    ensures
        result.is_some() ==> v@[result.unwrap() as int] == target,
{
    let mut lo: usize = 0;
    // ...
}'

run_scenario "new helper spec fn added, frozen unchanged" "PASS" \
  "$FROZEN_WITH_SPEC" "$STAGED_WITH_NEW_HELPER"

echo
echo "Summary: passed=$pass failed=$fail"
[ "$fail" -eq 0 ] && exit 0 || exit 1
