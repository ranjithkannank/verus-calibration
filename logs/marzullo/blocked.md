# marzullo: blocked by spec gap

## Summary

The frozen `marzullo` postcondition is **structurally unprovable** under
its frozen preconditions. This is not an SMT timeout, a missing lemma,
or a proof-search failure. It is a logical gap in the specification:
there exists a model satisfying every precondition under which the
postcondition is false.

Per AGENTS.md hard rule #4 ("Never weaken a spec to make verification
pass") and rule #2 ("Never use `#[verifier::external_body]`") and rule
#3 ("Never add `assume(...)`"), the exercise cannot be closed by any
admissible technique. This is a clean blocker. The result of the
exercise is the data point ("frozen marzullo spec is unprovable") plus
the partial verified machinery that would close the proof in roughly
one iteration if the missing precondition were added.

The architect has reviewed and re-confirmed this analysis three times
(design.md revisions `20260517T044650Z`, `20260517T044911Z`, and
`20260517T045216Z`), each time directing this `blocked.md` to be
written and the exercise to be exited.

## The structural gap

The frozen postcondition is a purely geometric property of the
**input intervals** (it never mentions `correct_at`):

```rust
exists|p: Reading|
    result.lo <= p && p <= result.hi
        && intervals_containing(intervals@, p).len()
           >= intervals.len() as nat - f as nat
```

The only precondition that touches `correct_at` is its *cardinality*:

```rust
correct_indices(intervals.len() as nat).len()
    >= intervals.len() as nat - f as nat
```

The remaining preconditions constrain only `intervals@`:

- `intervals.len() <= u32::MAX as nat` (overflow guard)
- `intervals.len() as nat >= 2 * (f as nat) + 1` (size bound)
- `well_formed(intervals@)` (each interval has `lo <= hi`)

Because `correct_at` is `uninterp spec fn` (no body, no axioms), the
verifier reasons over all interpretations of it. The standard
Marzullo proof needs every pair of correct intervals to overlap —
a Helly-1D condition — but **nothing in the frozen spec links
`correct_at` to the geometry of `intervals`**. Two correct sensors
are allowed to report disjoint intervals under the frozen
preconditions, which immediately breaks the standard proof.

## Concrete counterexample

```text
intervals = [Interval{lo:0, hi:0},
             Interval{lo:10, hi:10},
             Interval{lo:20, hi:20}]
n = 3
f = 1
correct_at(0) = true
correct_at(1) = true
correct_at(2) = false
```

Verification of preconditions:

- `intervals.len() = 3 <= u32::MAX` ✓
- `3 >= 2*1 + 1 = 3` ✓
- `well_formed`: each `Interval` has `lo == hi`, so `lo <= hi` ✓
- `correct_indices(3) = {0, 1}` has `len = 2 >= 3 - 1 = 2` ✓

Postcondition: for any `p` and any `result.lo <= p <= result.hi`,
each `intervals[i]` is a singleton `{x}` with `x ∈ {0,10,20}`, so
`point_in_interval(p, intervals[i])` holds iff `p == x`. At most one
of those three equalities holds simultaneously, so
`intervals_containing(intervals@, p).len() <= 1 < 2 = n - f` for
every `p`. The existential cannot be satisfied. ⊥

This is a proof of structural unprovability, not just hardness.

## The single failing verus obligation

The only inadmissible step in the partial proof is the Helly bound at
`exercises/marzullo.rs:292`, inside `lemma_exists_supported_endpoint`:

```
error: assertion failed
   --> exercises/marzullo.rs:292:20
292 |             assert(intervals[jm].lo <= intervals[k].hi);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ assertion failed
```

where `jm` is the argmax of `intervals[j].lo` over `correct_indices`
and `k` is an arbitrary correct index. There is no premise in the
spec that bounds `intervals[k].hi` from below by any quantity
mentioning a different index `jm`. The verifier is correct to reject.

The downstream `marzullo` postcondition failure at line 334 is the
direct consequence — the existence lemma cannot fire, so the
post-loop `Interval { lo: 0, hi: 0 }` placeholder cannot be shown
unreachable.

Final verifier state: **8 verified, 2 errors**.

## Suggested spec amendment (DOCUMENTATION ONLY — not a request to apply)

If a future un-freezing of the spec is on the table, the minimal
addition that would close the proof is a Helly-1D precondition on
the **pairs of correct sensors**:

```rust
forall|i: int, j: int|
    0 <= i < intervals.len() && 0 <= j < intervals.len()
    && correct_at(i) && correct_at(j)
        ==> intervals[i].lo <= intervals[j].hi
```

This is the standard sensor-fusion assumption that all honest sensors
are reporting bounds around some shared true value, so any honest
sensor's `lo` is below any honest sensor's `hi`. With this in scope,
the Helly step at line 292 closes by:

```rust
// jm and k are both in correct_indices
assert(correct_at(jm) && correct_at(k));
assert(intervals[jm].lo <= intervals[k].hi);  // direct from the new premise
```

This documentation is **not** a request to amend the spec — the spec
is frozen and the experiment's hard rule is symmetric. It is recorded
here so that a future operator can re-run the exercise with one line
of spec change and observe the partial proof close immediately.

## Verified machinery that survives the spec gap

The following obligations all verify under the frozen spec and form
the load-bearing machinery the Marzullo proof needs. If the
`correct_intervals_overlap` precondition above were added, none of
these would need to change — only the Helly assertion at line 292
becomes admissible, and the post-loop unreachable claim at line 387
follows from the existence lemma.

1. **Prefix-set abstraction** (lines 86-89):
   `contained_set_upto(intervals, p, m)` — restriction of
   `intervals_containing` to indices `[0, m)`.

2. **In-range/finiteness lemmas** (lines 93-129):
   - `lemma_contained_set_in_range` — `intervals_containing` is
     subset of `set_int_range(0, n)`, finite, len `<= n`.
   - `lemma_contained_set_upto_in_range` — same for the prefix
     variant.
   - `lemma_correct_indices_in_range` — same for `correct_indices`.
   Each uses `lemma_int_range` + `lemma_len_subset`. Bodies near-empty.

3. **Prefix-extension lemma** (lines 133-143):
   `lemma_contained_set_upto_extend` — `_set_upto(_, p, i+1)` is
   either `_set_upto(_, p, i).insert(i)` or `_set_upto(_, p, i)`
   depending on `point_in_interval(p, intervals[i])`. Body uses `=~=`
   (extensional equality auto-closes).

4. **Exec counting function** (lines 147-188):
   `count_containing(intervals, p)` — returns `u32` count of
   intervals containing `p`. Prefix-set loop invariant,
   `lemma_contained_set_upto_extend` at each iteration, defensive
   `assert(!_set_upto(_).contains(i))` before increment to close
   frame on the `c = c + 1` branch, post-loop `=~=` bridge to
   `intervals_containing`.

5. **Argmax recursion** (lines 192-236):
   `lemma_max_lo_in_set(s, intervals)` — over a finite index set,
   picks the index whose `intervals[j].lo` is maximum. Standard
   recursion on `s.len()` via `axiom_is_empty_len0` + `axiom_is_empty`
   + `choose|x| s.contains(x)`, with `assert forall ... by` blocks
   in each branch to bridge the IH forall back to `s` via
   `s2.contains(j)` when `j != j0`.

6. **Main scan loop** (lines 350-383):
   - Overflow safety: `assert(f as nat + 1 <= u32::MAX as nat) by { ... };`
   - Threshold computation: `let threshold: u32 = (n as u32) - f;`
   - Loop invariant has 8 conjuncts including the structural piece
     `forall|j2: int| 0 <= j2 < i ==>
        intervals_containing(intervals@, #[trigger] intervals@[j2].lo)
        .len() < intervals.len() as nat - f as nat`.
     The `#[trigger] intervals@[j2].lo` matches the witness shape
     from the existence lemma (ft_midpoint trigger pattern).
   - "Return on hit" branch (`c >= threshold`) closes immediately
     using `p` as the existential witness.
   - "Fall through" branch (`c < threshold`) re-establishes the
     structural invariant at `j2 = i`.

7. **Existence lemma scaffolding** (lines 245-301):
   Everything in `lemma_exists_supported_endpoint` up to and
   including the `argmax` step and the structural setup of the
   subset claim verifies. Only the Helly bound on line 292 and the
   downstream `intervals_containing.len() >= n - f` conclusion fail.

If the architect's suggested spec amendment ever lands, the proof
of `lemma_exists_supported_endpoint` closes by replacing the failed
assertion with one line invoking the new precondition; nothing else
in the file needs to change.

## Attempt history

See `logs/marzullo/attempts.md`. Summary:

- **Attempts 1-4**: structural sub-tasks 1-10 (stub, helpers,
  lemmas 1-3, `count_containing`, main loop with structural
  invariant, threshold + overflow guard, fall-through branch, stub
  existence lemma). Each one closed cleanly; verifier progressed
  from `4 verified, 1 errors` to `7 verified, 2 errors`.
- **Attempt 5**: sub-task 11. Implemented `lemma_max_lo_in_set` and
  the natural argmax-based body of `lemma_exists_supported_endpoint`.
  Result: `8 verified, 2 errors`. The single new error is exactly
  the Helly assertion the design predicted, at exactly line 292.
- **Attempt 6**: wrote `escalation.md` instead of trying further
  variants (no code change). Result unchanged.
- **This attempt (7)**: writing `blocked.md` per the architect's
  third-revision direction (`20260517T045216Z`). No code change.
  Verifier output unchanged at `8 verified, 2 errors`.

The architect's design.md predicted both the failure (at sub-task 11)
and the structural gap. The constructive counterexample above
upgrades that prediction from "expected to block" to "provably
unprovable", which is what makes this a clean blocker rather than
an SMT-difficulty issue.

## Forbidden alternatives — explicitly rejected

For audit clarity, the following paths were considered and rejected:

- **Add `assume(intervals[jm].lo <= intervals[k].hi)` at line 292.**
  Rejected: violates AGENTS.md rule #3.
- **Mark `lemma_exists_supported_endpoint` `#[verifier::external_body]`.**
  Rejected: violates AGENTS.md rule #2.
- **Weaken the `intervals_containing(...).len() >= n - f` clause in the
  postcondition to `>= 1` or similar.** Rejected: violates AGENTS.md
  rule #1 (spec is frozen) and rule #4 (spec weakening).
- **Add a new precondition to `marzullo` linking `correct_at` to
  intervals.** Rejected: same — preconditions are frozen.
- **Try further argmax/argmin/midpoint variants** (argmin on
  `intervals[j].hi`; argmax on `intervals[j].lo + intervals[j].hi`).
  Rejected: the counterexample falsifies the postcondition, so no
  algorithmic choice of `result` can satisfy it. The variant search
  cannot terminate at success.
- **Use a non-trivial result interval (`lo < hi`).** Rejected: same.
  The postcondition requires existence of a point `p` in the result
  interval supported by `>= n - f` input intervals. In the
  counterexample no such `p` exists anywhere on the real line, let
  alone inside any chosen result interval.

## Status

Blocked. Final verifier state `8 verified, 2 errors`. The partial
proof is the deliverable; this `blocked.md` is the rationale.
`logs/marzullo/status` set to `escalated` (the closest valid
status token in the implementer prompt's vocabulary; semantically
equivalent to "blocked-by-spec-gap" for this exercise).
