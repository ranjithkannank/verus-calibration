# Review: marzullo

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — The diff shows no modifications to the `requires` block (lines 341–346 in HEAD: `intervals.len() <= u32::MAX as nat`, `intervals.len() as nat >= 2 * (f as nat) + 1`, `well_formed(intervals@)`, `correct_indices(...).len() >= intervals.len() as nat - f as nat`, `correct_intervals_overlap(intervals@)`) or the `ensures` block (lines 347–351: `result.lo <= result.hi`, `exists|p: Reading| ...`) of `marzullo`. The only diff hunk in this region is the body replacement at lines 352+ where `unimplemented!()` was removed and the implementation was added.
2. Pre-existing spec fn bodies unchanged: YES — `correct_at` (uninterp), `well_formed`, `point_in_interval`, `intervals_containing`, `correct_indices`, and `correct_intervals_overlap` are all untouched by the diff. The diff only adds new helpers (`containing_upto` at +106 and proof fns starting +112).
3. No bypass tokens introduced: YES — Grep over `marzullo.rs` for `assume(|external_body|unimplemented!|unreachable!|panic!` returns no matches. The diff removed the pre-existing `unimplemented!()` placeholder. All `assert(...)` and `assert forall ... by { ... }` uses are legitimate.
4. No trivializing requires: YES — No new `requires` were added to `marzullo`. The new helper `count_containing` (+147) has `requires intervals.len() <= u32::MAX as nat`, which is a non-trivial physical bound needed for the `u32` counter, not a trivializer. The proof fns `lemma_max_lo_in_set` (+207) and `lemma_exists_supported_lo` (+255) have non-trivial requires (finiteness, range bounds, Helly-1D condition) that are genuine mathematical preconditions and discharged at call sites.
5. No closed/open toggles: YES — The new `containing_upto` (+104) is declared as plain `spec fn` (not `pub open` or `closed`), which is a fresh declaration. No pre-existing declaration's openness was changed.

## Justification

I diffed the file against `spec-frozen-marzullo` and inspected every hunk. The diff is purely additive in the spec sense: it adds proof-only spec helpers (`containing_upto`), proof functions (`lemma_containing_in_range`, `lemma_containing_upto_in_range`, `lemma_correct_indices_in_range`, `lemma_containing_upto_extend`, `lemma_max_lo_in_set`, `lemma_exists_supported_lo`), one exec helper (`count_containing`), and the body of `marzullo` replacing `unimplemented!()`. The frozen pre/post-conditions and all six pre-existing spec fn bodies are byte-identical to the baseline. No bypass tokens, no closed/open toggles, and the new requires on helpers are real physical bounds rather than trivializers. The contradiction-at-loop-exit pattern uses `assert(false)` after `lemma_exists_supported_lo` produces a witness — that's `assert`, not `assume`, so it's verified, not assumed.

## Reviewer notes

- The argmax + Helly-1D existence path is genuinely cleaner than the inclusion-exclusion contradiction used in `ft_midpoint`; the implementer's note in `AGENTS.md` calling out "constructive existence beats contradiction" is worth flagging to the architect for future sensor-fusion exercises.
- The `count_containing` exec helper's `c as nat <= i as nat` invariant + `containing_upto` prefix-set is a clean reusable pattern for "exec counter == spec set cardinality" — same shape as ft_midpoint's `le_set_upto`.
