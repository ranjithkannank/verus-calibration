# Blocked: OS__array__impl4__init2none

**Reason:** Verus version mismatch — upstream task uses pre-`final(self)` postcondition syntax that the current Verus build rejects.

**Status:** blocked before any agent attempt. Iteration count: 0.

## What happened

The upstream task (copied byte-for-byte from
`microsoft/verus-proof-synthesis/benchmarks/VeruSAGE-Bench/tasks/OS__array__impl4__init2none.rs`)
declares an `&mut self` function whose `ensures` clause reads
`self.seq@ ...` and `self@[index].is_None()`. The current Verus build
in this repo rejects this with:

```
error: to dereference a mutable reference parameter in a postcondition,
disambiguate by wrapping it in either `old` or `final`
```

`is_None()` is also flagged as deprecated (use `matches !`-style
patterns instead). These are not bugs in the task — they are signs
that the upstream benchmark was authored against an earlier Verus
release.

## What this is a data point on

This is exactly the failure mode the witness-file mechanism is
designed to catch — AGENTS.md's "Pre-spec verification (operator)"
section lists "spec syntax that no longer compiles" as the second of
two bug classes the witness check catches. The witness for this task
fails to verify under our Verus; the spec itself is the problem.

Distinct from task 6's harness-side `external_body` finding:
that one was a hook policy gap on our side. This one is an
upstream/downstream Verus version gap.

## Fix sketch (not done tonight)

Two paths, both substantial:

1. **Pin Verus to the version VeruSAGE-Bench was authored against.**
   Cleanest scientific framing — same Verus as AutoVerus and VeruSAGE.
   Cost: pinned-Verus install in parallel with our current one, plus
   per-task selection between the two builds.

2. **Modify the spec to current Verus.** Means the task is no longer
   the upstream task byte-for-byte. Defensible methodology change
   (annotate "ported to Verus N.M.k") but loses direct comparability
   to AutoVerus/VeruSAGE's published per-task results — if there are
   any. Per the upstream README the per-task leaderboard is "TO COME"
   so direct comparability is already a future-tense claim.

Path 1 is the right scientific move. Out of scope for tonight.

## What this does NOT block

The methodology probe continues with task 8
(`NO__spec__unbounded_log__get_fresh_nat_not_in`), whose witness
verifies cleanly under the current Verus.
