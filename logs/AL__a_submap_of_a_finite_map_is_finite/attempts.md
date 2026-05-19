## Attempt 1 — 2026-05-18

**Sub-task:** sub-task 1 (baseline) + sub-task 2 (smallest body) combined; baseline rejection captured then minimal proof tried.

**Approach:** Empty-body baseline produced `postcondition not satisfied` on `m1.dom().finite()`. Filled body with the standard subset-of-finite chain: `assert(m1.dom().subset_of(m2.dom()));` followed by `vstd::set_lib::lemma_len_subset(m1.dom(), m2.dom());`. The first line lets Verus derive subset from `submap_of`'s definition (key-inclusion forall); the second line is vstd's subset-of-finite lemma that ensures `s1.finite()` given `s1.subset_of(s2)` and `s2.finite()`.

**Verifier output:**
```
verification results:: 1 verified, 0 errors
```

**Next idea:** done.
