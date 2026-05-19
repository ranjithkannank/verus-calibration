## Attempt 1 — 2026-05-18

**Sub-task:** All four sub-tasks at once (the design note's "Expectation: 1-2 attempts" applies — the loop-invariant pattern is the same family as `binary_search`).

**Approach:** Added a two-conjunct `invariant` block (`i <= n` and `ret@.len() == i`) and a `decreases n - i` clause to the `while` loop. No defensive asserts needed inside the body — `Vec::push`'s built-in spec discharges the `ret@.len() == i + 1` step automatically from the SMT solver once the invariant is in place.

**Verifier output:**
```
verification results:: 2 verified, 0 errors
```
exit 0.

**Next idea:** done — hand to reviewer.
