# ft_midpoint attempts

## Attempt 1 — 2026-05-16
**Sub-tasks worked on:** 1–7 (spec helpers, L1 via vstd, L2 subset/finiteness
lemmas, L3 prefix-extension lemmas, `count_le`, `count_ge`). The main
`ft_midpoint` body remains a stub returning `readings[0]`.

**Approach:**
- Added proof-only spec helpers `le_set`, `ge_set`, `le_set_upto`, `ge_set_upto`.
- L2 (subset+finiteness): `lemma_le_set_in_range`, `lemma_ge_set_in_range`,
  `lemma_correct_indices_in_range`, plus `_upto` variants. Each one calls
  `vstd::set_lib::lemma_int_range(0, n)` then `lemma_len_subset` against
  `set_int_range(0, n)`.
- L3: `lemma_le_set_upto_extend` and the `ge_set_upto` mirror — empty bodies;
  the `=~=` in the ensures is discharged by Verus extensional reasoning.
- `count_le`/`count_ge` exec loops with five-conjunct invariants (bounds,
  len bound, count = prefix-set len, prefix-set finite, c <= i). Loop body
  calls the extend lemma + the `_upto_in_range` lemma to lift finiteness, then
  bumps `c` if the comparison holds. After the loop, an extensional `=~=`
  collapses the prefix set to the full set.

**Verifier output:**
```
verification results:: 11 verified, 1 errors
error: postcondition not satisfied
   --> ft_midpoint.rs:292:9
292 |         some_correct_le(readings@, result),
        failed this postcondition
... readings[0] at end of function body
```
The 11 verified items cover every helper and the two count functions. The
only failure is the placeholder `readings[0]` return in `ft_midpoint`.

**Next idea:** Land L4 pigeonhole lemmas (`lemma_pigeonhole_le` /
`lemma_pigeonhole_ge`) and the main `while` loop in `ft_midpoint` with the
in-loop early return. Leave the post-loop case to a later attempt (existence
lemma L6).

## Attempt 2 — 2026-05-16
**Sub-tasks worked on:** 8 and 9 (L4 pigeonhole lemmas only). The main
`ft_midpoint` body remains the `readings[0]` stub; the main loop is deferred
to attempt 3.

**Approach:**
- Added `lemma_pigeonhole_le(readings, v, f)` and the symmetric
  `lemma_pigeonhole_ge`. Both follow the same skeleton:
  1. Lift `correct_indices(n)` and `le_set/ge_set(readings, v)` into the
     finite universe `set_int_range(0, n)` via the L2 lemmas plus
     `lemma_int_range`.
  2. Show `(a + b).subset_of(set_int_range(0, n))` with a defensive
     `assert forall ... by { assert(a.contains(x) || b.contains(x)); }`,
     then `lemma_len_subset` to get `|a + b| <= n`.
  3. Apply `vstd::set_lib::lemma_set_intersect_union_lens(a, b)` for the
     inclusion-exclusion identity `|a+b| + |a∩b| == |a| + |b|`.
  4. Conclude `|a∩b| >= (n − f) + (f + 1) − n = 1`. Use
     `axiom_is_empty_len0` to flip `len >= 1` into `!is_empty()`, then
     `axiom_is_empty` to extract the existential witness `i`.
  5. From `a.contains(i)` get `correct_at(i)` and from `b.contains(i)` get
     the reading comparison; `assert(0 <= i < n && correct_at(i) && ...)`
     fires the auto-chosen `correct_at` trigger to close
     `some_correct_le` / `some_correct_ge`.

**Verifier output:**
```
verification results:: 13 verified, 1 errors
error: postcondition not satisfied
   --> ft_midpoint.rs:385:9
385 |         some_correct_le(readings@, result),
        failed this postcondition
... readings[0] at the end of the function body
```
The two new pigeonhole lemmas (`lemma_pigeonhole_le`, `lemma_pigeonhole_ge`)
verified clean (13 verified = previous 11 + 2 new). The only remaining
failure is the placeholder `readings[0]` in `ft_midpoint`, identical to
attempt 1.

**Next idea:** Sub-task 10: replace the stub with the main `while` loop
that calls `count_le` / `count_ge` per index, uses the pigeonhole lemmas at
the threshold-met branch to discharge `some_correct_le` / `some_correct_ge`,
and falls through to a `proof { assert(false); } unreachable!()` (to be
discharged later by `lemma_exists_midpoint`). Until L6 lands, the
post-loop path will fail — but the per-iteration return should already
discharge.

## Attempt 3 — 2026-05-16
**Sub-tasks worked on:** 10 (main `while` loop in `ft_midpoint`,
in-loop early return via pigeonhole). The post-loop fall-through retains
the `readings[0]` placeholder; existence lemma (L6) deferred to a later
attempt.

**Approach:**
- Added an overflow-safety `assert(f as nat + 1 <= u32::MAX as nat) by { ... }`
  before computing `threshold = f + 1`, justified by `2*f + 1 <= u32::MAX`.
- Six-conjunct loop invariant: `0 <= j <= n`, `n == readings.len()`,
  `threshold == f + 1`, `readings.len() <= u32::MAX`,
  `readings.len() >= 2*f + 1`, `correct_indices(...).len() >= n - f`. The
  last three carry the function-level precondition through the loop so
  that `count_le`/`count_ge` preconditions and the pigeonhole lemma
  preconditions both have what they need.
- In-loop body: call `count_le` and `count_ge` (each returns the set
  cardinality), and if both `>= threshold`, invoke
  `lemma_pigeonhole_le` and `lemma_pigeonhole_ge` to discharge both
  postcondition existentials, then `return v`.
- Post-loop: kept the `readings[0]` placeholder. The verifier will
  isolate the post-loop existence obligation as the only remaining
  failure.

**Verifier output:**
```
verification results:: 14 verified, 1 errors
error: postcondition not satisfied
   --> ft_midpoint.rs:385:9
385 |         some_correct_le(readings@, result),
        failed this postcondition
... readings[0] at the end of the function body
error: postcondition not satisfied
   --> ft_midpoint.rs:386:9
386 |         some_correct_ge(readings@, result),
        failed this postcondition
```
14 verified (= previous 13 + the `ft_midpoint` body now verifying for the
in-loop early return path). Only the post-loop fall-through fails.

**Next idea:** Sub-tasks 11–14: land `lemma_max_reading_in_set` and
`lemma_min_reading_in_set` (recursive over `s.len()`), then
`lemma_exists_midpoint` (L6) using the argmax/argmin lemmas on the
`Lo` / `Hi` partition. Wire L6 into the post-loop block via an
existential-witness `choose` + `assert(false)` to discharge the
fall-through, then change `readings[0]` to a `proof { ... }
unreachable!()` (legitimately unreachable, discharged by `assert(false)`).

## Attempt 4 — 2026-05-16
**Sub-tasks worked on:** 11 (L5 argmax — `lemma_max_reading_in_set`). The
mirror `lemma_min_reading_in_set` (sub-task 12) is deferred to the next
attempt. The post-loop `readings[0]` placeholder remains.

**Approach:**
- Added `lemma_max_reading_in_set(s: Set<int>, readings: Seq<Reading>) -> (jm: int)`
  with `decreases s.len()` recursion. Skeleton:
  1. `axiom_is_empty_len0(s)` + `axiom_is_empty(s)` to extract a witness
     `j0 = choose|x: int| s.contains(x)` from `s.len() >= 1` + `s.finite()`.
  2. Let `s2 = s.remove(j0)`. Defensive asserts on `s2.finite()` and
     `s2.len() == s.len() - 1` to satisfy the decreases measure.
  3. Base case `s2.len() == 0`: prove `s.contains(j) ==> j == j0` by
     contradiction using `axiom_is_empty_len0(s2)` + `axiom_is_empty(s2)`;
     return `j0`.
  4. Recursive case: get `jm2` from the recursive call. If
     `readings[j0] >= readings[jm2]`, return `j0` (assert the forall via
     case-split on `j == j0`). Else return `jm2` (symmetric).

**Verifier output:**
```
verification results:: 15 verified, 1 errors
error: postcondition not satisfied
   --> ft_midpoint.rs:434:9
434 |         some_correct_le(readings@, result),
        failed this postcondition
... readings[0] at end of function body
```
15 verified (= attempt 3's 14 + the new lemma_max_reading_in_set). Only
the unchanged post-loop placeholder fails, which is expected. The new
lemma — including the recursion on `decreases s.len()` and the two
`axiom_is_empty*` invocations — verified standalone.

**Next idea:** Sub-task 12: land the mirror `lemma_min_reading_in_set`
(identical skeleton, picks the index with minimum reading instead of
maximum).

## Attempt 5 — 2026-05-16
**Sub-tasks worked on:** 12 (L5 argmin — `lemma_min_reading_in_set`). The
post-loop `readings[0]` placeholder remains; sub-tasks 13–14 (L6 existence
lemma + wiring into post-loop) are deferred to the next attempt.

**Approach:**
- Added `lemma_min_reading_in_set(s, readings) -> jm` as the mirror of
  `lemma_max_reading_in_set`. Same skeleton:
  1. Extract a witness `j0` from `s.len() >= 1` + `s.finite()` via
     `axiom_is_empty_len0` + `axiom_is_empty`.
  2. `s2 = s.remove(j0)`; show `s2.finite()` and `s2.len() == s.len() - 1`.
  3. Base case `s2.len() == 0`: contradiction on `j != j0` via
     `axiom_is_empty_len0(s2)` + `axiom_is_empty(s2)`; return `j0`.
  4. Recursive case: get `jm2`; if `readings[j0] <= readings[jm2]`,
     return `j0` (case-split assert forall on `j == j0`). Else return
     `jm2`.

**Verifier output:**
```
verification results:: 16 verified, 1 errors
error: postcondition not satisfied
   --> ft_midpoint.rs:434:9
434 |         some_correct_le(readings@, result),
        failed this postcondition
... readings[0] at end of function body
```
16 verified (= attempt 4's 15 + new `lemma_min_reading_in_set`). Only
the unchanged post-loop placeholder fails, identical to the prior
attempt. The new lemma's recursion and case structure verified
standalone.

**Next idea:** Sub-task 13: land `lemma_exists_midpoint` using L5
(both argmax on `Lo` and argmin on `Hi`). Builds the `Lo`/`Hi`
partition over `[0, n)`, derives `|Lo| + |Hi| >= n >= 2f + 1`,
case-splits on which side reaches `f + 1`, applies the argmax/argmin
to extract a witness index `jm` with `le_set(_, readings[jm]).len() >=
f + 1` and `ge_set(_, readings[jm]).len() >= f + 1`. Wire L6 into
the post-loop block via a `choose` witness + `assert(false)` in a
subsequent attempt.

## Attempt 6 — 2026-05-16
**Sub-tasks worked on:** 13 (L6 existence — `lemma_exists_midpoint`). Sub-task 14
(wiring into the post-loop, plus loop invariant strengthening) is deferred to
the next attempt. The post-loop `readings[0]` placeholder remains untouched.

**Approach:**
- Added `lemma_exists_midpoint(readings, f)` between the argmin lemma and the
  `ft_midpoint` comment block. Proof by contradiction:
  1. `if !(exists|j: int| 0 <= j < n && le_set_len >= f + 1 && ge_set_len >= f + 1) { ... assert(false); }`
  2. Build `lo := { j ∈ [0, n) : le_set(_, readings[j]).len() <= f }` and
     `hi := { j ∈ [0, n) : ge_set(_, readings[j]).len() <= f }`. Both subset
     of `u := set_int_range(0, n)`, so finite with `len <= n` via
     `lemma_len_subset`.
  3. Under the negated existential, `(lo + hi) =~= u`: for any `x ∈ u`,
     instantiate the negated existential at `j = x` (explicit
     `assert(!(...))` inside the assert forall block to force trigger
     match on `le_set(readings, readings[x])`), then case-split on which
     count is `<= f`.
  4. `(lo + hi).len() == n` (from extensional equality); then
     `lemma_set_intersect_union_lens(lo, hi)` gives
     `lo.len() + hi.len() >= n >= 2*f + 1`.
  5. Case `lo.len() >= f + 1`: call `lemma_max_reading_in_set(lo, readings)`
     to get `jm ∈ lo` with `readings[j] <= readings[jm]` for all `j ∈ lo`.
     Show `lo ⊆ le_set(_, readings[jm])` and apply `lemma_len_subset` to
     get `|le_set(_, readings[jm])| >= |lo| >= f + 1`. But `jm ∈ lo`
     gives `|le_set(_, readings[jm])| <= f`: contradiction.
  6. Case else: `hi.len() >= f + 1` (linear-arith from `lo.len() + hi.len() >= 2f + 1`,
     `lo.len() <= f`). Symmetric via `lemma_min_reading_in_set`.

**Verifier output:**
```
verification results:: 17 verified, 1 errors
error: postcondition not satisfied
   --> ft_midpoint.rs:603:9
603 |         some_correct_le(readings@, result),
        failed this postcondition
... readings[0] at end of function body
error: postcondition not satisfied
   --> ft_midpoint.rs:604:9
604 |         some_correct_ge(readings@, result),
        failed this postcondition
... readings[0] at end of function body
```
17 verified (= attempt 5's 16 + new `lemma_exists_midpoint`). The
contradiction-style proof of L6 closed cleanly — both the `(lo+hi) =~= u`
extensional set equality, the inclusion-exclusion arithmetic, and both
argmax/argmin cases discharged on the first try. Only the unchanged
post-loop placeholder still fails, identical to the previous attempt.

**Next idea:** Sub-task 14: wire L6 into the post-loop. Two pieces are
required: (a) strengthen the main `while` invariant with a "no earlier
index satisfied both thresholds" forall:
```
forall|j2: int| 0 <= j2 < j as int ==>
    le_set(readings@, readings@[j2]).len() < f as nat + 1
    || ge_set(readings@, readings@[j2]).len() < f as nat + 1
```
and (b) in the post-loop block, call `lemma_exists_midpoint(readings@, f as nat)`,
`choose` the witness `jw`, derive a contradiction with the invariant at `j2 = jw`,
then replace `readings[0]` with `unreachable!()` discharged by `assert(false)`.

## Attempt 7 — 2026-05-16
**Sub-tasks worked on:** 14 (wire L6 into the post-loop). End-to-end verification.

**Approach:**
- Added a seventh conjunct to the main `while` invariant in `ft_midpoint`:
  `forall|j2: int| 0 <= j2 < j as int ==>
    le_set(readings@, #[trigger] readings@[j2]).len() < f as nat + 1
    || ge_set(readings@, readings@[j2]).len() < f as nat + 1`.
  The explicit `#[trigger] readings@[j2]` annotation matches the trigger that
  Verus chose for the same shape in `lemma_exists_midpoint`'s ensures (so the
  witness produced by `choose` later instantiates this invariant cleanly).
- Maintained the new invariant at the bottom of the loop body (the
  if-not-taken path) with four defensive asserts: `lec < threshold || gec
  < threshold`, `v == readings@[j as int]`, the two count-equals-set-len
  bridges, and the final disjunction at `j2 = j as int`.
- Post-loop proof block: `assert(j == n)`, `lemma_exists_midpoint(readings@, f as nat)`,
  `let jw = choose|jx: int| ...`, then the four bridge asserts that put
  `jw` in `[0, j as int)`, surface the existential's two `>= f + 1`
  bounds, instantiate the loop invariant at `j2 = jw` to get the
  contradictory `< f + 1` disjunction, and conclude `assert(false)`.
- The trailing `readings[0]` is now in a dead-code position (post
  `assert(false)`); both postconditions discharge vacuously.

**Verifier output:**
```
verification results:: 18 verified, 0 errors
```
18 verified (= attempt 6's 17 + `ft_midpoint` body). No errors. Verus
exited 0.

**Next idea:** Done — hand off to reviewer.
