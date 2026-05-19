# Review: MA__bin_sizes__shift_is_div

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — `requires 0 <= shift < 64,` (HEAD line 36) and `ensures x >> shift == x as nat / pow2(shift as int),` (HEAD line 37) are byte-identical to the frozen file. The diff hunk does not touch any line inside `shift_is_div`'s clauses; the only insertions in that function are inside the body (HEAD lines 39–41).
2. Pre-existing spec fn bodies unchanged: YES — the only pre-existing spec fn is `pub open spec fn pow2(i: int) -> nat` (HEAD lines 8–16). The diff shows no changes to its `decreases i` clause or to the `if i <= 0 { 1 } else { pow2(i - 1) * 2 }` body.
3. No bypass tokens introduced: YES — the diff contains no `assume(`, no `#[verifier::external_body]`, no `unimplemented!()`, no `unreachable!()`, no `panic!(`, and no `assume_specification`. All added proof steps are `assert(...)` calls (HEAD lines 24, 29–31, 41) and calls to vstd lemmas (`lemma2_to64`, `lemma_pow2_unfold`, `lemma_u64_shr_is_div`) plus a self-recursive call to the new helper.
4. No trivializing requires: YES — the only new function is `proof fn lemma_pow2_eq_vstd(n: nat)` (HEAD lines 18–33), which has no `requires` clause at all. Its `ensures pow2(n as int) == vstd::arithmetic::power2::pow2(n)` is a substantive bridge lemma, not a trivial obligation. `shift_is_div`'s `requires` clause is unchanged.
5. No closed/open toggles: YES — `pub open spec fn pow2` (HEAD line 8) remains `open`; the new helper `lemma_pow2_eq_vstd` is a `proof fn`, not a spec declaration. No `closed`/`open` modifier appears anywhere in the diff.

## Justification

I diffed `spec-frozen-MA__bin_sizes__shift_is_div..HEAD -- exercises/MA__bin_sizes__shift_is_div.rs` and audited every hunk. The diff is purely additive: a new `proof fn lemma_pow2_eq_vstd` helper (HEAD lines 18–33) and three lines inside `shift_is_div`'s previously empty body (HEAD lines 39–41). The frozen `pow2` spec fn, the `shift_is_div` signature, its `requires`, and its `ensures` are byte-identical to the frozen baseline. The helper lemma's proof is the legitimate `vstd::arithmetic::power2::pow2` bridge documented in the AGENTS.md playbook entry for this exercise (lemma2_to64 base case + lemma_pow2_unfold inductive step). No cheat tokens, no spec edits, no visibility flips.

## Reviewer notes (optional)

- Clean port of the playbook-documented bridge pattern; the helper's structure (base via `lemma2_to64`, step via `lemma_pow2_unfold`) is the textbook approach and worth keeping in the playbook for future `pow2`-shaped tasks.
- The defensive `assert((shift as nat) as int == shift as int)` on HEAD line 41 is a useful nudge across the `u64 → nat → int` cast chain — pattern worth flagging for other bit-shift / div tasks.
