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

## Attempt 3 — 2026-05-16
**Sub-task:** Design step 3 — invariant `(d)` bitmap abstraction (`seen[k] <==> exists|j| 0 <= j < i && qc.votes@[j].voter as int == k`) plus initial-state asserts and fall-through re-establishment block (case-split on `k == v_ghost as int`).
**Approach:** Added invariant (d) with explicit `#[trigger]` annotations on `seen@[k]` and `qc.votes@[j].voter`. Pre-loop asserts the initial bitmap state from `vec![false; n as usize]`. In the fall-through branch, after `seen.set(v, true); i = i + 1;`, an `assert forall ... by` block discharges (d) at the new i: case `k == v_ghost as int` witnesses by `j = old_i`; case `k != v_ghost as int` relies on Vec::set frame for `seen@[k]` and shows the new existential collapses to the old one (only differing j == old_i has voter == v_ghost != k, contradiction).
**Verifier output:** 2 errors (was 3; one closed by adding (d)):
- line 201: final `votes_len >= threshold` — needs invariant (c) distinct prefix + bridging lemma + threshold arithmetic.
- line 224: `lemma_qc_has_honest_voter` — body still empty.
```
verification results:: 1 verified, 2 errors
```
The duplicate-voter early return (was line 147 in attempt 2) is no longer in the error list: (d) instantiated at `m = v_id as int` provides the existential witness for `!voters_distinct`, so Verus discharges that path automatically.
**Next idea:** Design step 4 — add invariant `(c)` pairwise distinctness, re-established in the fall-through using (d) at `m = v_ghost as int` (which read backwards says `v_ghost` is not yet in the prefix). This gives `voters_distinct(*qc)` at loop exit, which is the missing conjunct in the final-return postcondition path.

## Attempt 4 — 2026-05-16
**Sub-task:** Design step 4 — invariant `(c)` pairwise distinct voters in prefix, re-established in the fall-through branch using (d) at `k = v_ghost as int` (which, since `seen[v] == false`, reads as: no prior index has voter == v_ghost).
**Approach:** Added invariant `(c) forall|j,k| 0 <= j < k < i ==> qc.votes@[j].voter != qc.votes@[k].voter`. Before `seen.set(v, true)` in the fall-through branch, captured the helper fact `forall|j: int| 0 <= j < old_i ==> qc.votes@[j].voter as int != v_ghost as int` by reading invariant (d) at `k = v_ghost as int` as a contrapositive on the inner exists. After `seen.set + i += 1`, an `assert forall|j: int, k: int| 0 <= j < k < i implies ...` block case-splits on `k < old_i` (covered by old (c)) vs `k == old_i` (new index has voter == v_ghost, and the captured fact rules out any earlier `j` matching v_ghost).
**Verifier output:** 2 errors, same as attempt 3 (no regression, and (c) re-establishment was accepted with no new errors):
- line 118 / line 229: final `votes_len >= threshold` postcondition — needs bridging lemma `voters(qc).len() == qc.votes.len()` (under distinct) + threshold arithmetic. This is step 6.
- line 252: `lemma_qc_has_honest_voter` postcondition — body still empty. This is steps 7–8.
```
verification results:: 1 verified, 2 errors
```
Notably the new asserts/invariants at lines 140, 171, 183 (the (c) invariant, the captured helper, and the re-establishment) all verified — no errors fired at any of those locations. Invariant (c) holds at loop exit ⇒ `voters_distinct(*qc)` is now part of the loop-exit context for the final return.
**Next idea:** Design step 6 — lift `lemma_prefix_extend`, `lemma_push_to_set`, `lemma_to_set_finite`, `lemma_set_insert_new_len`, `lemma_range_nodeid_len` verbatim from `quorum_count.rs`. Write `lemma_distinct_seq_to_set_len`, the internal `voter_seq` spec fn, `lemma_voters_as_to_set`, and the bridging `lemma_distinct_voters_len(*qc)`. Call it after the loop, assert `threshold as nat == byzantine_threshold(n)`, return `votes_len >= threshold`.
