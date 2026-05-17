# Review: counter_producer

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — The diff hunks in `counter.rs` (lines 32-35, 44-47, 53-56), `producer.rs` (lines 16-21), and `main.rs` (lines 19-22) only replace `unimplemented!()` bodies (or add the `pipeline` body) with real exec code. No `requires` or `ensures` line is touched in any hunk.
2. Pre-existing spec fn bodies unchanged: YES — The three closed spec fns in `counter.rs` (`value` line 17-19, `bound` line 21-23, `invariant` line 25-27) do not appear in the diff at all.
3. No bypass tokens introduced: YES — No `assume(`, `#[verifier::external_body]`, `unreachable!()`, or `panic!(` appears in any added line. The three `unimplemented!()` occurrences are removed and not reintroduced.
4. No trivializing requires: YES — The diff adds no `requires` clauses anywhere. The producer loop's `invariant` block (producer.rs lines 24-30) is a loop invariant, not a function precondition.
5. No closed/open toggles: YES — No `closed`/`open` keyword appears anywhere in the diff.

## Justification

I diffed `spec-frozen-counter_producer..HEAD -- exercises/counter_producer` and inspected every hunk. All three modified files (`counter.rs`, `producer.rs`, `main.rs`) show only body-replacement changes for the four previously `unimplemented!()` functions (`Counter::new`, `Counter::incr`, `Counter::get`, `pipeline`) plus a real body for `produce`. No spec text, no spec fn body, no visibility annotation, no `requires` clause, and no bypass token is touched. The loop invariant in `produce` is the six-conjunct snapshot+bound-preservation pattern flagged in AGENTS.md's `counter_producer` notes — it lives in the function body and is appropriate for the exec proof, not a spec weakening.

## Reviewer notes (optional)

- Clean cross-module composition: producer reasons about Counter purely through the closed spec fns (`invariant`, `value`, `bound`) — no defensive asserts needed, matching the pattern noted for `counter_multifile`.
- The `let start = c.get()` snapshot is the load-bearing exec hook that lets the invariant name a u32 anchor for `c.value() == start + i`; worth keeping in the multi-module playbook.
