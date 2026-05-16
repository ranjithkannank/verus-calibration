# Review: quorum_cert

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — diff contains no hunks modifying `requires` or
   `ensures` lines. `verify_qc_structure` keeps `requires n > 0,` and `ensures
   result == (voters_distinct(*qc) && all_voters_in_range(*qc, n) &&
   has_quorum(*qc, n)),` (HEAD lines 298–301, identical to frozen baseline
   lines 113–116). `lemma_qc_has_honest_voter` keeps its full 5-clause
   `requires` block and the `exists honest …` `ensures` (HEAD lines 437–444,
   identical to frozen baseline lines 137–144).
2. Pre-existing spec fn bodies unchanged: YES — the frozen file's
   `pk_of`, `signature_valid` (both `uninterp`, no body), `all_signatures_valid`,
   `voters`, `voters_distinct`, `byzantine_threshold`, `all_voters_in_range`,
   `has_quorum`, and `is_valid_qc` (HEAD lines 62–105) are not touched by any
   hunk in the diff. New helper `spec fn voter_seq` at HEAD line 110 is a
   freshly-added internal helper, permitted.
3. No bypass tokens introduced: YES — `grep` for `assume(|external_body|unimplemented!|unreachable!|panic!|assume_specification`
   in `exercises/quorum_cert.rs` returns zero hits. The diff removes the two
   `unimplemented!()` placeholders (frozen lines 121 and 162) and replaces
   them with real bodies.
4. No trivializing requires: YES — no new `requires` were added to
   pre-existing functions. New helper lemmas carry only the meaningful
   preconditions needed for their conclusions: `lemma_set_insert_new_len`
   has `s.finite(), !s.contains(x)` (HEAD line 161), the natural antecedent
   for the cardinality axiom; `lemma_distinct_seq_to_set_len` has the
   `forall i,j … s[i] != s[j]` distinctness hypothesis (HEAD lines 192–193),
   without which the conclusion is false; `lemma_distinct_voters_len` has
   `voters_distinct(qc)` (HEAD line 272), again the natural antecedent.
   None of these short-circuit a verification obligation.
5. No closed/open toggles: YES — no `open spec` ↔ `closed spec` change on
   any pre-existing declaration. New `spec fn voter_seq` (HEAD line 110) is
   freshly added; pre-existing `pub open spec fn` declarations retain their
   `pub open` modifiers verbatim.

## Justification

I diffed `spec-frozen-quorum_cert..HEAD -- exercises/quorum_cert.rs` and read
the full HEAD file. Every diff hunk lies strictly below the frozen spec
block: a new helpers section (HEAD lines 107–288) and bodies for the two
obligation functions (HEAD lines 302–421 and 445–516). I separately grepped
for the full set of bypass tokens listed in AGENTS.md and the reviewer
checklist and found none. New helper lemmas have sensible, non-trivializing
preconditions, and the lemma bodies use only `assert`, `choose`, calls to
named `vstd` lemmas (`axiom_set_insert_len`, `lemma_len_subset`,
`lemma_fundamental_div_mod`), and standard `nonlinear_arith` discharges —
all permitted tools. Approving.

## Reviewer notes

- The pigeonhole-via-contradiction shape and the `lemma_fundamental_div_mod`
  detour are now reusable patterns for any future BFT-shaped exercise
  needing `2n/3 + 1` vs. `n/3` reasoning; consider lifting them into a
  shared notes section in AGENTS.md.
- The bitmap-abstraction invariant (d) plus the explicit "capture old (d)
  contrapositive before mutating" pattern is the clean way to combine
  `Vec::set` framing with an existential invariant; worth noting for the
  next exec-loop exercise.
