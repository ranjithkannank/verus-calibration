# Review: NR__extra__lemma_set_of_first_n_nat_is_finite

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — The `ensures Set::new(|i: nat| i < n).finite()` clause at HEAD line 9 is byte-identical to the frozen baseline; no `requires` clause exists in either version. The added `decreases n` at HEAD line 10 is a termination measure (required for the recursive proof), not a spec change.
2. Pre-existing spec fn bodies unchanged: YES — The exercise file contains no `spec fn` declarations; `lemma_set_of_first_n_nat_is_finite` is a `proof fn`. No spec fn bodies to compare.
3. No bypass tokens introduced: YES — Diff adds only `if`/`else`, a `let`, a recursive call, and two `assert(... =~= ...)` lines (HEAD lines 12–18). No `assume(`, no `#[verifier::external_body]`, no `unimplemented!()`, no `unreachable!()`, no `panic!(`.
4. No trivializing requires: YES — No `requires` clauses are added anywhere in the diff.
5. No closed/open toggles: YES — No `spec fn` declarations exist in this file, so no `closed`/`open` toggles are possible.

## Justification

I ran `git diff spec-frozen-NR__extra__lemma_set_of_first_n_nat_is_finite..HEAD -- exercises/NR__extra__lemma_set_of_first_n_nat_is_finite.rs` and inspected the single hunk. The implementer added a `decreases n` termination measure and an induction body (base case asserts extensional equality with `Set::<nat>::empty()`; step recurses on `(n-1) as nat` and asserts extensional equality with `.insert(m)`). The `ensures` clause is untouched; no spec helpers, no requires, no bypass tokens. The proof matches the playbook entry already recorded for this exercise.

## Reviewer notes (optional)

- `decreases` is a termination obligation, not a spec clause — additions are routine for recursive proof fns and do not constitute spec weakening.
- The `=~=` extensional collapse + `axiom_set_insert_finite` default-broadcast chain is clean; no `assume_specification` or surrogate axiom calls.
