# Review: AL__a_submap_of_a_finite_map_is_finite

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — diff touches only lines 12–14 (proof body); `requires` (lines 7–9) and `ensures` (lines 10–11) hunks are untouched.
2. Pre-existing spec fn bodies unchanged: YES — frozen file declares no `spec fn`; only a single `pub proof fn` whose body was empty and is now filled.
3. No bypass tokens introduced: YES — added lines are `assert(m1.dom().subset_of(m2.dom()));` (line 13) and `vstd::set_lib::lemma_len_subset(m1.dom(), m2.dom());` (line 14). No `assume(`, `external_body`, `unimplemented!()`, `unreachable!()`, or `panic!(`.
4. No trivializing requires: YES — no `requires` clauses added anywhere (no new helper functions either).
5. No closed/open toggles: YES — no `closed spec` or `open spec` declarations exist in the file; nothing toggled.

## Justification

I ran `git diff spec-frozen-AL__a_submap_of_a_finite_map_is_finite..HEAD -- exercises/AL__a_submap_of_a_finite_map_is_finite.rs` and confirmed the only change is a two-line addition inside the previously empty proof body of `a_submap_of_a_finite_map_is_finite`. Both added lines are legitimate proof steps: an `assert` of the subset relation and a call to `vstd::set_lib::lemma_len_subset`, neither of which constitutes a bypass token. The signature, generics, `requires`, and `ensures` are byte-identical to the frozen baseline. The exercise contains no `spec fn` declarations, so item 2 has nothing to compare.

## Reviewer notes (optional)

- Proof is a clean, minimal application of the `lemma_len_subset` pattern (already documented in AGENTS.md "Discovered patterns" under quorum_cert). The intermediate `subset_of` assert before the lemma call is the standard nudge to make the SMT trigger fire — consistent with prior pigeonhole-bound proofs.
- No new helper `spec fn`s or `proof fn`s were introduced; the patch is two lines inside an already-spec'd function.
