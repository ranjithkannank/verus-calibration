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
