## Attempt 1 — 2026-05-18

**Sub-task:** End-to-end verify (design sub-task 3). Before making any edits I ran verus against the scaffold as committed in 7586365 to establish a baseline.

**Approach:** No code changes made on this attempt. Read AGENTS.md, the role definition, the exercise file, and the design note. Observed that the scaffolded `exercises/vec_swap_v2.rs` already contains both the exec body (`v[i]` read, `v[j]` read, two `v.set` calls) and the multiset proof block (broadcast use of `group_to_multiset_ensures`, `group_multiset_axioms`, `group_multiset_properties`; case split on `i == j`; extensional-equality `=~=` on the `update`/`update` view; per-element count assert-by). Ran `verus exercises/vec_swap_v2.rs --crate-type=lib` to establish the baseline.

**Verifier output:**
```
verification results:: 1 verified, 0 errors
```
exit 0.

**Next idea:** Verus passes already. Status = verus_passed. Hand back to reviewer. (Note for the reviewer: I did not author this proof — the scaffold committed in 7586365 already contained the full body and proof block. My diff against `spec-frozen-vec_swap_v2` is empty for `exercises/vec_swap_v2.rs`. The invention-test framing assumes the implementer fills in the proof; on this exercise the operator's scaffold already does that. Flagging so the operator can decide whether the run still counts as an invention data point.)
