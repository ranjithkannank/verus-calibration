## Attempt 1 — 2026-05-15T00:00:00Z
**Approach:** Initial implementation using bitmap-backed distinct-count with four-conjunct loop invariant (cursor bounds, bitmap abstraction, counter abstraction, count bound). Wrote helper lemmas `lemma_prefix_extend`, `lemma_push_to_set`, `lemma_to_set_finite`, `lemma_set_insert_new_len`, `lemma_set_insert_existing`. Found that `vstd::set::axiom_set_insert_len` is the correct name (not `lemma_set_insert_len`).

**Verifier output:**
```
error: invariant not satisfied before loop
   --> exercises/quorum_count.rs:153:13
    |
153 |             count as nat == voters@.subrange(0, i as int).to_set().len(),
    | (at i=0, count=0: empty subrange's to_set().len() not automatically seen as 0)

error: assertion failed
   --> exercises/quorum_count.rs:254:20
    |
254 |             assert(count as nat <= n as nat);
    | (in Case A after count += 1: need count_old < n, requires pigeonhole/subset-size argument)

verification results:: 5 verified, 2 errors
```

**Next idea:** 
1. Add explicit hint before loop: `voters@.subrange(0, 0).to_set() =~= Set::<NodeId>::empty()` and use `Set::empty().len() == 0`.
2. For Case A overflow/bound: Add `lemma_range_set_finite_len(n)` proving `|{k: NodeId | k < n}| = n as nat`, then use subset monotonicity to prove `pref_old.to_set().len() < n` when there's a missing element.
