# NR__extra__lemma_set_of_first_n_nat_is_finite — attempts

## Attempt 1 — 2026-05-18

**Sub-task:** Step 2 from the design note — smallest possible body that could close the obligation.

**Approach:** Induction on `n` with `decreases n`. Base case `n == 0`: extensional equality with `Set::<nat>::empty()` via `=~=`. Inductive case: recurse on `(n - 1) as nat`, then state extensional equality `Set::new(|i: nat| i < n) =~= Set::new(|i: nat| i < n - 1).insert(n - 1)`. Finiteness of the recursive call's set is preserved by `.insert`.

**Verifier output:**
```
verification results:: 1 verified, 0 errors
```

**Next idea:** Done. Hand to reviewer.
