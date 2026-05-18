# Review: counter_filler

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — every diff hunk touches only exec bodies. In `counter.rs` the hunks at lines 32–38, 44–50, 53–59 each replace `unimplemented!()` with a body while leaving the surrounding `requires`/`ensures` blocks in context only. In `filler.rs` the hunk at lines 19–32 replaces `unimplemented!()` with a `while` loop body; the `requires` (lines 13–16) and `ensures` (lines 17–20) blocks appear only as diff context. In `main.rs` the hunk at lines 19–25 replaces `unimplemented!()`; the `ensures r == target` clause is context only.
2. Pre-existing spec fn bodies unchanged: YES — the three `pub closed spec fn`s `value`, `bound`, `invariant` (counter.rs lines 17–27) are not touched by any diff hunk.
3. No bypass tokens introduced: YES — added code is `Counter { value: 0, bound: bound }` (counter.rs:35), `self.value = self.value + 1;` (counter.rs:47), `self.value` (counter.rs:56), the `while` loop in filler.rs:22–31, and the three-line `pipeline` body in main.rs:22–24. No `assume(`, `#[verifier::external_body]`, `unreachable!()`, or `panic!(` appear in any addition. The four `unimplemented!()` markers in the baseline are removed (replaced by real implementations), which is the intended direction.
4. No trivializing requires: YES — no new `requires` clauses anywhere in the diff. The loop in filler.rs:23–28 uses `invariant` and `decreases` only.
5. No closed/open toggles: YES — `closed spec fn value`, `closed spec fn bound`, `closed spec fn invariant` (counter.rs:17, 21, 25) retain their `closed` visibility; no spec declarations are added or modified.

## Justification

I diffed HEAD against `spec-frozen-counter_filler` and inspected every hunk. All five hunks are pure exec-body fills: three method bodies in `counter.rs`, the `fill_to` loop body in `filler.rs`, and the `pipeline` composition in `main.rs`. Spec clauses (`requires`, `ensures`), pre-existing closed `spec fn` bodies, and module structure are byte-identical to the frozen baseline. The implementer's loop invariant in `fill_to` (`c.invariant()`, `c.value() <= target`, `c.bound() == old(c).bound()`, `target <= c.bound()`) is a clean adaptation of the `counter_producer` snapshot+bound-preservation family to the variable-free loop shape — no spec-side machinery introduced, no helper spec fns added.

## Reviewer notes (optional)

- The diff is minimal and tidy: 4 removed `unimplemented!()` markers, ~12 lines of real implementation, zero proof scaffolding required (no `assert`, no `proof fn` helpers).
- The `target <= c.bound()` conjunct in the loop invariant is the load-bearing one that threads the precondition `target <= old(c).bound()` through each iteration to discharge `incr`'s `value() < bound()` precondition at the call site — same threading pattern as `counter_producer`, applied to a 4-conjunct invariant instead of 6.
