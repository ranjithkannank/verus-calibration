# Design: marzullo.rs

## Critical caveat — read this first

The frozen postcondition is a **purely structural property of
`intervals@`**:

```
exists|p: Reading|
    result.lo <= p && p <= result.hi
        && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat
```

It never mentions `correct_at`. The only precondition that touches
`correct_at` is its *count* (`correct_indices(n).len() >= n - f`). The
only constraints on `intervals@` are `well_formed` (each interval has
`lo <= hi`) and the size bound `n >= 2f + 1`.

Because `correct_at` is `uninterp`, the verifier reasons over all
interpretations. The postcondition's `p` is independent of
`correct_at`, so the spec writer's prose justification ("the n - f
correct intervals all overlap, since they all contain any 'true'
value") is **not derivable from the spec** — that overlap assumption
is stated only in the prose comment, not in `requires`.

> **Counterexample to provability.** Take `intervals = [[0,0],
> [10,10], [20,20]]`, `f = 1`, `n = 3`. Pick `correct_at(0) =
> correct_at(1) = true`, `correct_at(2) = false`. Then
> `correct_indices.len() = 2 = n - f`, `n >= 2f + 1 = 3`, and
> well-formedness holds (each interval has `lo == hi`). All
> preconditions are satisfied, but no point lies in two of these
> intervals: `intervals_containing(p).len() <= 1` for every `p`, so
> the postcondition cannot hold.

The likely fix (out of scope — spec is frozen) is to add a Helly-1D
precondition linking `correct_at` to intervals, e.g.

```
forall|i: int, j: int|
    0 <= i < n && 0 <= j < n && correct_at(i) && correct_at(j)
        ==> intervals[i].lo <= intervals[j].hi
```

This is missing.

**Implementer guidance.** Build the brute-force machinery anyway: the
exec scan, the `intervals_containing` prefix-set abstraction, the
counting loop, and the structural pieces (subset/finiteness lemmas,
inclusion-exclusion). The only thing that *should* fail to discharge
is the existence lemma (§Helper Lemmas item 5 below). When you hit
it, do not `assume` it and do not weaken the spec. Try the brute-force
proof structure honestly; if it cannot close, write
`logs/marzullo/escalation.md` and let the architect re-confirm the
structural gap. If re-confirmed, exit via `blocked.md`.

## Representation choice

No new struct. `intervals: &Vec<Interval>` is read-only, so the exec
work is purely a scan with counting. The result is built directly as
`Interval { lo: p, hi: p }` — the spec's `result.lo <= result.hi`
trivially holds, and the existential witness is `p` itself.

Why not a non-trivial interval (`lo < hi`)? Because the spec only
requires *existence* of a supported `p` in `[lo, hi]`, and a
single-point interval makes both the equality `lo <= p <= hi`
trivial and the witness obvious. There is zero proof advantage to a
wider interval.

## Algorithmic sketch

```
fn marzullo(intervals, f):
    let n = intervals.len();
    let threshold = (n - f) as u32;
    // Scan candidate points = intervals[i].lo for i in [0, n).
    for i in 0..n:
        let p = intervals[i].lo;
        let c = count_containing(intervals, p);
        if c >= threshold:
            return Interval { lo: p, hi: p };
    // Provably unreachable (modulo the existence lemma).
    assert(false);
    return intervals[0];
```

`intervals[i].lo` is a sound candidate set because, if any point `q`
is contained in `k` intervals, then the largest `intervals[i].lo`
among those `k` is also contained in all `k` of them (each of those
intervals contains `q`, and their `lo <= q`, and the largest `lo` is
still `<= q` while being `>= lo_i` for each). Translation: scanning
input-`lo` endpoints suffices to find a maximal-overlap candidate.

## Spec helpers (proof-only, new in this file)

Mirror `ft_midpoint`'s prefix-set abstraction:

```rust
spec fn contained_set(intervals: Seq<Interval>, p: Reading) -> Set<int> {
    Set::new(|i: int| 0 <= i < intervals.len() && point_in_interval(p, intervals[i]))
}

spec fn contained_set_upto(intervals: Seq<Interval>, p: Reading, m: int) -> Set<int> {
    Set::new(|i: int|
        0 <= i < m && i < intervals.len() && point_in_interval(p, intervals[i]))
}
```

Note `contained_set` is extensionally equal to `intervals_containing`
(the spec helper). The implementer should keep `intervals_containing`
as the externally visible name and use `contained_set` only if
internal-naming hygiene helps — they can also just inline the
`Set::new` in helper lemmas.

## Key invariants

Loop invariant for `count_containing` (analogous to ft_midpoint's
`count_le`):

```
0 <= i as int <= intervals@.len() as int,
intervals.len() <= u32::MAX as nat,
c as nat == contained_set_upto(intervals@, p, i as int).len(),
contained_set_upto(intervals@, p, i as int).finite(),
c as nat <= i as nat,
```

Loop invariant for the main `marzullo` scan (mirrors ft_midpoint's
final loop):

```
0 <= i as int <= n as int,
n == intervals@.len(),
threshold as nat == n as nat - f as nat,
intervals.len() <= u32::MAX as nat,
intervals.len() as nat >= 2 * (f as nat) + 1,
well_formed(intervals@),
correct_indices(intervals.len() as nat).len() >= intervals.len() as nat - f as nat,
forall|j2: int| 0 <= j2 < i as int ==>
    intervals_containing(intervals@, #[trigger] intervals@[j2].lo).len()
        < intervals.len() as nat - f as nat,
```

Note the explicit `#[trigger] intervals@[j2].lo` — this is the
ft_midpoint trigger-matching trick. Without it, the post-loop
instantiation of the loop invariant at the witness index from the
(would-be) existence lemma will not fire.

## Helper lemmas predicted

1. `lemma_contained_set_in_range(intervals, p)` — `contained_set` is
   a subset of `set_int_range(0, n)`, hence finite, hence
   `.len() <= n`. Modelled on `lemma_le_set_in_range` from
   ft_midpoint.

2. `lemma_contained_set_upto_in_range(intervals, p, m)` — same for
   the prefix variant. Modelled on `lemma_le_set_upto_in_range`.

3. `lemma_contained_set_upto_extend(intervals, p, i)` — prefix
   extension:
   ```
   point_in_interval(p, intervals[i]) ==>
       contained_set_upto(intervals, p, i+1)
           =~= contained_set_upto(intervals, p, i).insert(i),
   !point_in_interval(p, intervals[i]) ==>
       contained_set_upto(intervals, p, i+1)
           =~= contained_set_upto(intervals, p, i),
   ```
   Body should be empty (`=~=` should close on its own); if not, an
   `assert forall|x: int| ... by {}` block bridges. Modelled on
   `lemma_le_set_upto_extend`.

4. `count_containing(intervals: &Vec<Interval>, p: Reading) -> (c:
   u32)` — exec counting function, postcondition `c as nat ==
   contained_set(intervals@, p).len()`. Body is the prefix-set loop
   from `count_le` adapted to use `point_in_interval` instead of
   `<=`.

5. **`lemma_exists_supported_endpoint(intervals: Seq<Interval>, f:
   nat)`** — the load-bearing existence lemma. Signature:
   ```
   proof fn lemma_exists_supported_endpoint(intervals: Seq<Interval>, f: nat)
       requires
           intervals.len() >= 2 * f + 1,
           well_formed(intervals),
           correct_indices(intervals.len()).len() >= intervals.len() - f,
       ensures
           exists|j: int|
               0 <= j < intervals.len()
               && intervals_containing(intervals, #[trigger] intervals[j].lo).len()
                  >= intervals.len() - f,
   ```
   **This is the proof that should not close from the frozen
   preconditions** (see Critical caveat). The natural shape — pick
   `j = argmax over correct indices of intervals[j].lo`, show every
   correct interval contains `intervals[j].lo` — requires the
   missing Helly-1D assumption to bound `intervals[j].lo <=
   intervals[k].hi` for every correct `k != j`. Without it, that
   step is unsound.

## SMT trouble spots

- **Existence lemma #5** is the main wall. Expect 3 consecutive
  failed attempts on the same `forall|k: int| correct_at(k) ==>
  intervals[k].hi >= intervals[j].lo` step; that's the escalation
  trigger.

- **`u32` arithmetic for the threshold.** `n - f` must be computed
  as `u32`. The precondition gives `intervals.len() <= u32::MAX as
  nat` and `n >= 2f + 1 >= f`, so `n - f >= f + 1 >= 1` and fits in
  `u32`. Mirror the overflow guard from ft_midpoint:
  ```
  assert(f as nat + 1 <= u32::MAX as nat) by {
      assert(2 * (f as nat) + 1 <= u32::MAX as nat);
  };
  ```
  Compute `let threshold: u32 = (n as u32) - f;`.

- **Frame on `c = c + 1`.** Standard — defensive
  `assert(!contained_set_upto(intervals@, p, i as int).contains(i as int))`
  before the increment, mirroring `count_le`.

- **`=~=` for the post-loop bridge** in `count_containing`:
  ```
  assert(contained_set_upto(intervals@, p, intervals@.len() as int)
      =~= contained_set(intervals@, p));
  ```
  matches `ft_midpoint`'s pattern verbatim.

- **Trigger on `intervals@[j2].lo` in the main loop invariant.** Make
  sure the trigger is on `intervals@[j2].lo` (not `intervals@[j2]`),
  because the post-loop witness chosen from the existence lemma will
  also be `intervals@[jw].lo`. Mismatch here means SMT never
  instantiates.

- **Post-loop `assert(false)` requires the existence lemma to fire.**
  If lemma #5 cannot be proved, the post-loop unreachable claim also
  fails. They block as a pair.

## Suggested order of operations

1. Stub `marzullo` returning `intervals[0]` with empty body and a
   stub `assert(false); ` (placeholder for unreachable) — confirm the
   file parses but verification fails as expected.
2. Add the prefix-set spec helpers `contained_set` /
   `contained_set_upto`.
3. Prove the three structural lemmas (`lemma_contained_set_in_range`,
   `lemma_contained_set_upto_in_range`,
   `lemma_contained_set_upto_extend`). These should all be near-empty
   bodies thanks to `=~=`.
4. Write `count_containing` with its loop invariant (copy-and-adapt
   from `count_le`). Should verify cleanly.
5. Wire up the main loop with the structural invariant
   (`forall|j2 < i| contained_set_upto.len() < threshold`). The
   "return on hit" branch should close.
6. **Now attempt `lemma_exists_supported_endpoint`.** This is where
   the spec gap bites. Try the natural argmax-over-correct-indices
   shape; if the Helly step can't close, write `escalation.md`.
7. (If #6 closes, miraculously) Connect the existence lemma at the
   post-loop point and discharge the final `assert(false)`.

## Sub-tasks

1. Stub `marzullo` returning a placeholder and confirm the file
   parses.
2. Add `contained_set` and `contained_set_upto` spec helpers.
3. Prove `lemma_contained_set_in_range` (subset + finiteness +
   length bound).
4. Prove `lemma_contained_set_upto_in_range` (same shape, prefix
   variant).
5. Prove `lemma_contained_set_upto_extend` (insert / no-op via
   `=~=`).
6. Implement `count_containing` with its loop invariant, mirroring
   `count_le`. Confirm it verifies.
7. Write the main loop skeleton in `marzullo` with the structural
   loop invariant; verify the in-loop return branch closes (assumes
   #5/#6 are landed).
8. Add the threshold computation and overflow-safety assert before
   the loop.
9. Re-establish the loop invariant in the fall-through branch of
   each iteration (the `count < threshold` recording step).
10. Stub `lemma_exists_supported_endpoint` with an empty proof body
    and confirm Verus reports the existence as the only remaining
    failure.
11. Attempt the proof of `lemma_exists_supported_endpoint` via
    argmax-over-correct-indices + Helly step. **Expected to block
    here.** If three consecutive attempts on the Helly step fail,
    write `logs/marzullo/escalation.md` describing the spec gap.
12. (Conditional on #11 closing) Use the lemma at the post-loop
    point, extract the witness, contradict the invariant, discharge
    `assert(false)`.

## Summary: brute-force endpoint scan with prefix-set counting, mirroring ft_midpoint; expected to block on the existence lemma because the frozen spec lacks the Helly-1D "correct intervals overlap" precondition the algorithm needs.
