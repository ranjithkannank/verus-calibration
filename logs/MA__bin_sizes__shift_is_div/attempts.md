## Attempt 1 — 2026-05-18

**Sub-task:** Sub-tasks 1 + 2 combined — capture scaffold rejection (empty body fails `ensures x >> shift == x as nat / pow2(shift as int)` as expected), then write the smallest proof body that closes the obligation.

**Approach:** Two-piece proof. (a) Bridge lemma `lemma_pow2_eq_vstd(n: nat)` proves `local_pow2(n as int) == vstd::arithmetic::power2::pow2(n)` by induction on `n`, using `lemma2_to64` for the base case and `lemma_pow2_unfold` for the inductive step (both recursions step by 2). (b) `shift_is_div` calls `vstd::bits::lemma_u64_shr_is_div(x, shift)` to get `(x >> shift) == x as nat / vstd_pow2(shift as nat)`, then the bridge lemma collapses `vstd_pow2(shift as nat)` to local `pow2(shift as int)`. One defensive `(shift as nat) as int == shift as int` assert closes the final equality.

**Verifier output:**
```
verification results:: 3 verified, 0 errors
```

**Next idea:** Done. Hand to reviewer.
