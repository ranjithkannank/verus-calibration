# Review: NR__definitions_u__lemma_maxphyaddr_facts

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — the only diff hunk touches lines inside the body of `lemma_maxphyaddr_facts` (between `{` on line 33 and `}` on line 38). The `ensures 0xFFFFFFFF <= MAX_PHYADDR <= 0xFFFFFFFFFFFFF` on line 32 is untouched, and no other `requires`/`ensures` clauses appear in the diff. The axiom `axiom_max_phyaddr_width_facts` and the `MAX_PHYADDR` exec const's `ensures` (lines 13–28) are unchanged.
2. Pre-existing spec fn bodies unchanged: YES — the only pre-existing spec construct is `pub spec const MAX_PHYADDR_SPEC: usize = ((1usize << MAX_PHYADDR_WIDTH) - 1usize) as usize;` (line 18), which does not appear in the diff. No `spec fn` declarations existed in the freeze; no new ones added.
3. No bypass tokens introduced: YES — the diff adds only `axiom_max_phyaddr_width_facts();` (line 34, calling the pre-existing frozen axiom), two `assert(...) by (compute)` lines (35–36), and one `assert(forall ...) by (bit_vector)` (37). No `assume(`, no new `#[verifier::external_body]` (the existing one on line 10 is on the frozen `MAX_PHYADDR_WIDTH` constant and is not in the diff), no `unimplemented!()`, `unreachable!()`, or `panic!(`.
4. No trivializing requires: YES — no new `requires` clauses appear in the diff.
5. No closed/open toggles: YES — no `closed`/`open` modifiers appear anywhere in the diff.

## Justification

I diffed `spec-frozen-NR__definitions_u__lemma_maxphyaddr_facts..HEAD` against `exercises/NR__definitions_u__lemma_maxphyaddr_facts.rs`. The entire diff consists of four lines inserted into the body of `lemma_maxphyaddr_facts` between its `{` and `}`: one call to the pre-existing frozen axiom, two `by (compute)` literal-value asserts on `1usize << 32` and `1usize << 52`, and one `by (bit_vector)` monotonicity forall. None of these touch the spec surface (ensures clause, MAX_PHYADDR_SPEC body, axiom signature, or exec const ensures). The proof strategy is clean: mix compute (for concrete shift endpoints) and bit_vector (for universal monotonicity over the abstract width), with the axiom call to bring `32 <= MAX_PHYADDR_WIDTH <= 52` into scope.

## Reviewer notes

- The compute/bit_vector split for "shift bounds over an axiomatized width" is a reusable pattern; it is already captured in the AGENTS.md "Discovered patterns" section.
- The `when_used_as_spec` annotation on the exec const seamlessly resolves `MAX_PHYADDR` in the lemma's spec-position ensures — no manual unfolding was needed in the proof body, confirming the annotation pattern works through this harness.
