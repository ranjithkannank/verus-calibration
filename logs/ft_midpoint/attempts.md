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
