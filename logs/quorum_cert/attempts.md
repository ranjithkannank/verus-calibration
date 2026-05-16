## Attempt 1 — 2026-05-16
**Sub-task:** Design step 1 — `verify_qc_structure` skeleton (bitmap, early returns, threshold compare) with only the minimal `decreases` + bound invariants (`i <= qc.votes@.len()`, `seen@.len() == n as nat`, `n > 0`). No correctness invariants yet.
**Approach:** Replaced `unimplemented!()` with the loop body shown in `quorum_cert.design.md` §2 (matches the working pattern from `quorum_count.rs`). Left `lemma_qc_has_honest_voter` body empty (only the original comments).
**Verifier output:** 3 errors, all expected:
- line 118 / line 140: postcondition fails on the `if v_id >= n` early-return path (no witness yet).
- line 118 / line 150: postcondition fails on the final `votes_len >= threshold` path (need `lemma_distinct_voters_len` to relate `voters(qc).len()` to `qc.votes.len()`, plus threshold arithmetic).
- line 173: `lemma_qc_has_honest_voter` postcondition (body still TODO).
```
verification results:: 0 verified, 3 errors
```
**Next idea:** Design step 2/3 — add the correctness invariants `(b)` in-range prefix, `(c)` distinct prefix, `(d)` bitmap-vs-prefix abstraction, and re-establish them in the fall-through branch (with `seen.set` frame asserts and `lemma_prefix_extend`-style extensional equality). Lift `lemma_prefix_extend`, `lemma_push_to_set`, `lemma_to_set_finite`, `lemma_set_insert_new_len`, `lemma_set_insert_existing`, `lemma_range_nodeid_len` from `quorum_count.rs` first since they're verbatim reusable.

## Attempt 2 — 2026-05-16
**Sub-task:** Design step 2 — invariant `(b)` in-range prefix conjunct plus witness asserts for the first early-return path (`v_id >= n`).
**Approach:** Added `forall|j: int| 0 <= j < i ==> qc.votes@[j].voter as int < n as int` (with explicit `#[trigger]` on the indexed voter access) to the loop invariant. Added three asserts on the first early-return path to chain `qc.votes@[i].voter == v_id` → `v_id >= n` → `!all_voters_in_range(*qc, n)`. Fall-through path preserves (b) trivially since the new index `i` is the `v_id` we just verified to be `< n`.
**Verifier output:** 3 errors, all expected and at different locations than attempt 1's first error:
- line 147 (was 140): second early-return (duplicate voter) — needs invariants (c)/(d).
- line 157 (was 150): final threshold return — needs bridging lemma to relate `voters(qc).len()` to `qc.votes.len()`, plus threshold arithmetic.
- line 180 (was 173): `lemma_qc_has_honest_voter` postcondition — body still empty.
```
verification results:: 0 verified, 3 errors
```
The first early return (`v_id >= n`) no longer appears in the error list, confirming step 2 closes it cleanly.
**Next idea:** Design step 3 — add invariant `(d)` bitmap abstraction, re-establish in the fall-through branch with `seen.set` frame asserts + `lemma_prefix_extend` extensional equality (lift from `quorum_count.rs`).
