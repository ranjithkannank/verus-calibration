# Review: cross_module_counter

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — diff hunks only touch function bodies (lines 55, 67, 76, 89-102 of HEAD). The `ensures` blocks for `Counter::new` (lines 50-53), `Counter::incr` (lines 62-65), `Counter::get` (lines 73-74) and `count_up_to` (lines 86-87), and the `requires` blocks for `Counter::incr` (lines 59-61) and `Counter::get` (lines 71-72), are byte-identical to the frozen baseline.
2. Pre-existing spec fn bodies unchanged: YES — the three `pub closed spec fn`s `value()`, `bound()`, and `invariant()` at lines 37-47 are not in any diff hunk.
3. No bypass tokens introduced: YES — the diff only *removes* `unimplemented!()` (replacing with real bodies); no `assume(`, `#[verifier::external_body]`, `unreachable!()`, or `panic!(` is introduced. Verified by inspecting all four diff hunks (lines 55, 67, 76, 89-102).
4. No trivializing requires: YES — no new `requires` clauses added on any function. The `while` loop at lines 91-101 uses `invariant` and `decreases` (not `requires`), which are loop-scoped and not function contracts.
5. No closed/open toggles: YES — `value()`, `bound()`, `invariant()` remain `pub closed spec fn` at lines 37, 41, 45; no diff hunk touches those declarations.

## Justification

I diffed `spec-frozen-cross_module_counter..HEAD -- exercises/cross_module_counter.rs` and inspected every hunk. Exactly four hunks exist, each replacing an `unimplemented!()` body with a real implementation: `Counter::new` constructs the struct literal, `Counter::incr` increments the private `value` field, `Counter::get` returns it, and `client::count_up_to` runs a `while` loop with a four-conjunct invariant in the closed spec-fn vocabulary. No spec clause, spec-fn body, visibility modifier, or function signature is altered. The added loop invariant lives inside the function body (not a function contract) so it does not count under check 4.

## Reviewer notes (optional)

- The implementer's client-side proof rests entirely on the postconditions of `new`/`incr`/`get`, demonstrating the closed-spec-fn opacity boundary holds — clean exemplar of the intended pattern.
- No new helper `spec fn`s were introduced; the bodies are minimal and the loop invariant matches the design note's predicted four conjuncts.
