# Review: binary_search

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — `git diff spec-frozen-binary_search..HEAD -- exercises/binary_search.rs` shows the only hunk replaces the body between `{` (line 26) and `}` (line 64). The `requires is_sorted(v@),` and both `ensures` conjuncts (the `result.is_some() ==> { ... }` block and the `result.is_none() ==> forall|i: int| ...` line) are outside the hunk and byte-identical to the frozen tag.
2. Pre-existing spec fn bodies unchanged: YES — `pub open spec fn is_sorted(s: Seq<i64>) -> bool { forall|i: int, j: int| 0 <= i <= j < s.len() ==> s[i] <= s[j] }` (lines 12-14 of HEAD) does not appear in the diff.
3. No bypass tokens introduced: YES — the diff removes `unimplemented!()` and introduces only `assert(...)` and `assert forall ... implies ... by { ... }` constructs (lines 39, 45-49, 50, 54-58, 59 of HEAD). Grep of the added lines shows no `assume(`, no `#[verifier::external_body]`, no `unreachable!()`, no `panic!(`, no `assume_specification`.
4. No trivializing requires: YES — the diff adds no `requires` clauses anywhere. The loop has an `invariant` block (lines 30-35) and `decreases hi - lo` (line 36), neither of which is a function-level `requires`.
5. No closed/open toggles: YES — `is_sorted` remains `pub open spec fn` (line 12); no `closed`/`open` keywords appear in the diff.

## Justification

I diffed `exercises/binary_search.rs` against tag `spec-frozen-binary_search` and confirmed the diff is confined to the function body of `binary_search` between the opening `{` on line 26 and the closing `}` on line 64. The spec surface (the `requires is_sorted(v@)` precondition and the two-conjunct postcondition) and the `is_sorted` helper are untouched. The implementation uses a standard half-open `[lo, hi)` binary search with an invariant, a `decreases` clause, and `assert forall ... by { assert(is_sorted(v@)); }` blocks to trigger sortedness instantiation — these are legitimate proof hints, not bypasses. No `assume`, `external_body`, panic, or unreachable construct appears anywhere in the diff.

## Reviewer notes (optional)

- The `assert forall ... implies ... by { assert(is_sorted(v@)); }` pattern is a clean, repeatable way to instantiate a frozen `open spec fn` quantifier without weakening the spec; worth noting in `AGENTS.md` discovered patterns (the implementer already did so).
- Half-open window with `hi = mid` rather than `mid - 1` cleanly avoids `usize` underflow obligations; useful precedent for `bounded_log`.
