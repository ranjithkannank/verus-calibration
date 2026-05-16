## Attempt 1 — 2026-05-15T00:00:00Z
**Approach:** Implemented half-open binary search `[lo, hi)` with a 5-conjunct loop invariant: `is_sorted`, cursor bounds, `hi <= v.len()`, and two `forall` exclusion ranges. Added sortedness `assert_forall` blocks in each branch to help SMT instantiate the sorted order, plus a `decreases hi - lo` termination clause.
**Verifier output:** `verification results:: 2 verified, 0 errors`
**Next idea:** Passed on attempt 1 — no further iteration needed.
