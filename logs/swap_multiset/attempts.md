# swap_multiset attempts

## Attempt 1 — 2026-05-18

**Sub-task:** Full implementation — exec body (sub-task 1) + multiset clause (sub-task 2) + end-to-end verify (sub-task 3). Single-shot attempt with no prior log to consult.

**Approach:**
- Exec body: read both values, then write swapped via index-set assignment (`v[i] = vj; v[j] = vi;`). Vec's `IndexSetTrustedSpec` gives `new_container@ === self@.update(index as int, val)`, so after the two writes `v@ == s0.update(i_, a).update(j_, b)` where `a = s0[j_]` and `b = s0[i_]`.
- Multiset clause: brought in the entire broadcast group `group_to_multiset_ensures` (in `vstd::seq_lib`) which includes the load-bearing lemma `to_multiset_update`: `s.update(i, a).to_multiset() == s.to_multiset().insert(a).remove(s[i])`. Applied it implicitly twice to reduce the two updates to a chain `s0.to_multiset().insert(a).remove(b).insert(b).remove(a)`. Closed via extensional equality `=~=` on Multiset.
- Key bridging assert: `s1[j_] == s0[j_]` (the value at j_ after updating i_ equals the original, even when i_ == j_ because in that case we wrote `a = s0[j_]` to position i_, which equals s0[j_]).

**Verifier output:** `verification results:: 1 verified, 0 errors`.

**Next idea:** done; hand to reviewer.

**Notes for AGENTS.md:**
- `vstd::seq_lib::group_to_multiset_ensures` is the broadcast group bundling `to_multiset_build`, `to_multiset_remove`, `to_multiset_len`, `to_multiset_contains`, `to_multiset_insert`, `to_multiset_update`. One `broadcast use` line unlocks all of them.
- The load-bearing lemma for in-place mutation proofs is `to_multiset_update(s, i, a) : s.update(i, a).to_multiset() == s.to_multiset().insert(a).remove(s[i])`.
- `Multiset` is `#[verifier::ext_equal]`, so `=~=` discharges count-by-count equality of arbitrary insert/remove chains automatically. No need to spell out per-element count arithmetic when the underlying counts trivially match.
- Vec assignment via `v[i] = val` syntax compiles to `vstd::std_specs::core::index_set` which uses `IndexSetTrustedSpec::spec_index_set_ensures`. For Vec<T,A>, this is `new_container@ === self@.update(index as int, val)`. No explicit `set` method call needed.
