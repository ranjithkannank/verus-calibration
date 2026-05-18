# Review: counter_filler

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — `git diff spec-frozen-counter_filler..HEAD -- exercises/counter_filler` shows zero modifications inside any `requires` or `ensures` block. The hunks in `counter.rs` (lines 35, 47, 56), `filler.rs` (lines 22–31), and `main.rs` (lines 22–24) all replace `unimplemented!()` bodies with implementations; the surrounding spec lines (counter.rs 30–33, 39–45, 51–54; filler.rs 13–20; main.rs 19–20) are unchanged context.
2. Pre-existing spec fn bodies unchanged: YES — `closed spec fn value`, `closed spec fn bound`, and `closed spec fn invariant` in `counter.rs` (lines 17–27) are not touched by any diff hunk. No new `spec fn` declarations were added.
3. No bypass tokens introduced: YES — diff contains no `assume(`, no `#[verifier::external_body]`, no `unimplemented!()` in HEAD (the three pre-existing ones were replaced with real exec code at counter.rs:35, counter.rs:47, counter.rs:56), no `unreachable!()`, no `panic!(`. `assume_specification` is also absent.
4. No trivializing requires: YES — no new function-level `requires` clauses were added on any exec function. The new `while` loop in `filler.rs` carries an `invariant` block (lines 23–27) and a `decreases` clause (line 28), but those are loop annotations, not function preconditions.
5. No closed/open toggles: YES — the three `closed spec fn` declarations in `counter.rs` retain their `closed` qualifier; no `open` was introduced and no existing `open` was closed.

## Justification

I diffed HEAD against `spec-frozen-counter_filler` and inspected every hunk. All three hunks in `counter.rs` swap `unimplemented!()` for straightforward exec implementations (`Counter { value: 0, bound: bound }`, `self.value = self.value + 1`, `self.value`). The `filler.rs` hunk replaces `unimplemented!()` with a `while` loop whose body is a single `c.incr()`; its four-conjunct invariant and `decreases target - c.value()` are inside the loop, not in the function spec. The `main.rs` hunk implements `pipeline` as a three-line composition of `Counter::new`, `fill_to`, `get`. No spec text changed, no cheat tokens present, no visibility toggles.

## Reviewer notes (optional)

- Clean cross-family transfer from `counter_producer`: the invariant shape `(c.invariant(), c.value() <= target, c.bound() == old(c).bound(), target <= c.bound())` is exactly the target-bounded analogue the design note hinted at, with no `start` snapshot and no separate index counter.
- Loop guard reads `c.get()` directly; the precondition for `get` is discharged by `c.invariant()` in the loop invariant. Worth flagging as a clean idiom for closed-spec APIs whose exec accessors require the invariant.
