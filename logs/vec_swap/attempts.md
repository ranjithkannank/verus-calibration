## Attempt 1 — 2026-05-18T00:00:00Z
**Sub-task:** Full implementation — exec body (sub-task 1) + multiset-preservation proof (sub-task 2) + end-to-end verify (sub-task 3).
**Approach:** Ported the witness's proof structure: exec body is `let tmp = v[i]; let v_j = v[j]; v.set(i, v_j); v.set(j, tmp);`. Proof block opens three broadcast groups (`group_to_multiset_ensures`, `group_multiset_axioms`, `group_multiset_properties`) and builds `s1 = old(v)@.update(i, old[j])`, `s2 = s1.update(j, old[i])`, asserts `v@ =~= s2`, case-splits on `i == j` (no-op via two `=~=` collapses) vs. `i != j` (count-by-element `assert forall|x: u32| s2.to_multiset().count(x) == m.count(x)` with the two `m.count(a) > 0` / `m.count(b) > 0` witnesses to handle the remove-then-insert ordering wrinkle, then close `s2.to_multiset() =~= m`).
**Verifier output:** `verification results:: 1 verified, 0 errors` (exit 0).
**Next idea:** Hand to reviewer; append discovery pattern to `AGENTS.md`.
