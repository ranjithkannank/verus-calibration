# Review: ft_midpoint

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — the diff against `spec-frozen-ft_midpoint` leaves `pub uninterp spec fn correct_at` (line 58), `pub open spec fn correct_indices` (lines 62-64), `pub open spec fn some_correct_le` (lines 66-69), `pub open spec fn some_correct_ge` (lines 71-74), and the `requires`/`ensures` of `pub fn ft_midpoint` (lines 598-604) byte-identical. The only spec-region additions are new `spec fn` helpers (`le_set`, `ge_set`, `le_set_upto`, `ge_set_upto`) and `proof fn` lemmas, all added below the frozen spec block.
2. Pre-existing spec fn bodies unchanged: YES — `correct_indices`, `some_correct_le`, `some_correct_ge` bodies appear unmodified in the diff (no `-` lines touch lines 62-74); `correct_at` remains uninterpreted.
3. No bypass tokens introduced: YES — grep over the file finds zero `assume(`, `external_body`, `unimplemented!()`, `unreachable!()` macro invocations, or `panic!(` in code; the single match on "unreachable" (line 654) is inside a comment. The previously present `unimplemented!()` in `ft_midpoint` is removed and replaced by a real implementation with a proof-of-unreachability `assert(false)` block followed by a concrete `readings[0]` return.
4. No trivializing requires: YES — `ft_midpoint`'s `requires` clauses are unchanged (lines 598-601). New helpers have natural preconditions (`readings.len() <= u32::MAX`, `0 <= m <= readings.len()`, `0 <= i < readings.len()`, `s.len() >= 1`, `readings.len() >= 2*f+1`, etc.); none are `false` or other trivializing forms.
5. No closed/open toggles: YES — `correct_indices`, `some_correct_le`, `some_correct_ge` remain `pub open spec fn`. New helpers (`le_set`, `ge_set`, `le_set_upto`, `ge_set_upto`) are introduced as plain `spec fn` (closed) but they did not exist in the baseline, so there is no toggle on a pre-existing declaration.

## Justification

Verified by running `git diff spec-frozen-ft_midpoint..HEAD -- exercises/ft_midpoint.rs` and inspecting every hunk: the only modifications between the frozen tag and HEAD are (a) the added `use vstd::set_lib::*;` import, (b) the new proof-only spec helpers and lemmas in the section delimited by `// --- Proof-only spec helpers ---`, and (c) the body of `ft_midpoint` replacing `unimplemented!()` with the search-loop implementation. No `requires`, `ensures`, `pub open spec fn` body, or `uninterp` declaration is touched. The new lemmas use legitimate vstd primitives (`lemma_int_range`, `lemma_len_subset`, `lemma_set_intersect_union_lens`, `axiom_is_empty_len0`, `axiom_is_empty`) — none of which are bypasses — and the post-loop `readings[0]` return is gated behind a proof block that derives `assert(false)` from `lemma_exists_midpoint` versus the strengthened loop invariant, which is a legitimate unreachability proof rather than an escape hatch.

## Reviewer notes (optional)

- The `assert(false)` followed by a concrete return value (`readings[0]`) is a clean way to handle a provably-unreachable tail without invoking `unreachable!()` — worth noting in `AGENTS.md`'s "Discovered patterns" as a pattern to prefer when proof-of-unreachability is available.
- The pigeonhole proof pairs `lemma_set_intersect_union_lens` with the universe-bounded subset reasoning twice (once inside `lemma_pigeonhole_*`, once inside `lemma_exists_midpoint`); future sensor-fusion exercises may benefit from extracting that into a reusable cardinality lemma.
