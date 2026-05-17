# marzullo escalation — 2026-05-16

## Status

Escalating after attempt 5. The frozen postcondition is **structurally
unprovable** from the frozen preconditions, and the design predicted
this. Rather than burn two more iterations on doomed variants, I am
escalating now with a constructive counterexample.

## The obligation that won't close

`lemma_exists_supported_endpoint` has to discharge

```
exists|j: int|
    0 <= j < intervals.len()
    && intervals_containing(intervals, intervals[j].lo).len()
       >= intervals.len() - f
```

from these preconditions only:

```
intervals.len() >= 2 * f + 1,
well_formed(intervals),                       // each interval has lo <= hi
correct_indices(intervals.len()).len() >= intervals.len() - f,
```

`correct_at` is `uninterp` and appears nowhere else in the spec.

The single failing verus line is:

```
error: assertion failed
   --> exercises/marzullo.rs:292:20
292 |             assert(intervals[jm].lo <= intervals[k].hi);
```

This is the **Helly-1D step**: for two correct indices `jm` and `k`,
the argmax-over-correct-lo proof needs the cross-interval bound
`intervals[jm].lo <= intervals[k].hi`. No precondition supplies it.

## Counterexample (proves the postcondition is unprovable)

Take

```
n = 3, f = 1
intervals = [[0, 0], [10, 10], [20, 20]]
correct_at(0) = true, correct_at(1) = true, correct_at(2) = false
```

Then all preconditions hold:
- `n = 3 >= 2*1 + 1 = 3`
- Each interval has `lo == hi`, so `well_formed` holds
- `correct_indices(3) = {0, 1}`, so `correct_indices.len() = 2 = n - f`

But for every `p: i64`, `intervals_containing(intervals, p).len() <= 1`
(no two of the three singleton intervals share a point), so the
postcondition's existential

```
intervals_containing(intervals, p).len() >= n - f = 2
```

cannot hold for any `p`. Hence the `marzullo` postcondition is false
in this model, so no exec body can satisfy it without an `assume` or
`external_body` (both forbidden).

The lemma's own existence statement is symmetrically false — pick the
same model and observe that for every `j ∈ {0, 1, 2}`, `intervals[j]`
is a singleton, so `intervals_containing(intervals, intervals[j].lo) =
{j}` has size 1 < 2.

## What I tried (one attempt; further attempts are doomed)

### Attempt 5 — argmax over `correct_indices` of `intervals[j].lo`

The natural shape: pick `jm = argmax{intervals[j].lo : j ∈
correct_indices}`. Show every correct `k` is in
`intervals_containing(intervals, intervals[jm].lo)` by proving both:

  (a) `intervals[k].lo <= intervals[jm].lo` — closes from argmax.

  (b) `intervals[jm].lo <= intervals[k].hi` — **fails**. This is the
      Helly-1D fact "two correct intervals overlap." It would follow
      from a precondition like

      ```
      forall|i: int, j: int|
          0 <= i < n && 0 <= j < n && correct_at(i) && correct_at(j)
              ==> intervals[i].lo <= intervals[j].hi
      ```

      but the frozen spec does not include any such relation between
      `correct_at` and interval geometry.

### Why variants don't help

  - **Argmin on `intervals[j].hi` instead of argmax on `lo`.** Same
    structural gap, mirror-imaged: now we'd need `intervals[k].lo
    <= intervals[jm].hi`, again a Helly-1D claim with no premise.

  - **Different candidate point** (median, mean, etc.). The
    postcondition counts `intervals_containing(intervals@, p)`, which
    is over all intervals. Without any link between `correct_at` and
    geometry, no point can be shown to be in ≥ `n - f` intervals.

  - **Pigeonhole on `correct_indices` against `[lo, hi]`.** Whatever
    pigeon-set we construct, the only "many-elements" set is
    `correct_indices`, and bridging it to `intervals_containing(_, p)`
    *is* the Helly step.

  - **Carry intervals from Byzantine sensors in the pigeon-set.**
    `intervals_containing` does count Byzantine intervals, but
    Byzantine sensors are unconstrained — Verus reasons over all
    interpretations of `correct_at`, including ones where all
    Byzantine intervals are pairwise disjoint from the correct ones.
    The counterexample above is exactly that situation.

In every variant, the unprovable step is the same: linking
`correct_at(_)` to interval geometry.

## Best guess at what's missing

A **Helly-1D precondition** in the frozen spec, e.g.

```rust
pub open spec fn correct_intervals_overlap(intervals: Seq<Interval>) -> bool {
    forall|i: int, j: int|
        0 <= i < intervals.len() && 0 <= j < intervals.len()
        && correct_at(i) && correct_at(j)
        ==> intervals[i].lo <= intervals[j].hi
}
```

added to `marzullo`'s `requires`. This is the standard 1D Helly fact
("any pairwise-overlapping family of 1D intervals has common
intersection") restricted to correct sensors. It corresponds to the
informal justification in the spec comment:

> The intersection of the n - f correct intervals (which all overlap,
> since they all contain any 'true' value) has count >= n - f at every
> point inside it.

With this added, the argmax proof closes immediately:
  - For correct `k`, `intervals[k].lo <= intervals[jm].lo` (argmax),
    and `intervals[jm].lo <= intervals[k].hi` follows directly from
    `correct_intervals_overlap` instantiated at `(jm, k)`.
  - The rest of `lemma_exists_supported_endpoint` is already in
    place: subset of `correct_indices` into `intervals_containing(_,
    intervals[jm].lo)` plus `lemma_len_subset` closes the count.

The implementer-side proof machinery is already complete around this
gap: `lemma_max_lo_in_set`, the structural subset/finiteness lemmas,
`count_containing` with its prefix-set invariant, and the main loop's
trigger-aligned invariant are all verified. The only missing piece is
the cross-interval bound.

## Recommendation to the architect

Either:

1. **Add `correct_intervals_overlap` to the frozen `marzullo`
   precondition** (treat the spec freeze as amendable for this gap)
   and re-invoke the implementer. The remaining proof closes in
   roughly one more iteration.

2. **Mark the exercise blocked-by-spec-gap** and capture this as a
   data point about specification design (the prose justification
   omitted a load-bearing premise; the verifier caught it). The
   implementer-side machinery up to and including the Helly step is
   verified and would serve as a regression for future fixes.
