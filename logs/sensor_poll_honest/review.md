# Review: sensor_poll_honest

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — the diff's only hunks insert content at lines 38–138 (new helper lemmas, before `poll`) and lines 168–237 (replacing `poll`'s body). The `poll` signature, its `requires` block (HEAD lines 142–150), and its `ensures` block (HEAD lines 151–166) appear only as unchanged context in the diff with no `+`/`-` lines.
2. Pre-existing spec fn bodies unchanged: YES — `project_intervals` (HEAD lines 29–31) and `reports_containing` (HEAD lines 33–36) are above the first inserted hunk and appear unmodified in the diff. The sibling modules `fusion.rs` and `auth.rs` have no diff entries (`git diff … --stat` shows only `main.rs` changed).
3. No bypass tokens introduced: YES — grepping the diff shows no `assume(`, no `#[verifier::external_body]`, no `unreachable!()`, no `panic!(`, and no `assume_specification`. The only `unimplemented!()` interaction is its REMOVAL (diff line `-    unimplemented!()`). The body uses `assert(...)` and `assert forall ... by { ... }` blocks only.
4. No trivializing requires: YES — `poll` gains no new `requires` (its block is unchanged). The new helper `lemma_honest_supporter_exists` declares non-trivial requires (diff lines 83–86: `reports.len() >= 2*f+1`, supporter-set cardinality, and correct-set cardinality) that are real obligations discharged at the call site in `poll` from the precondition + marzullo's postcondition. The other two new helpers (`lemma_reports_containing_in_range`, `lemma_correct_indices_in_range`) have no `requires` at all.
5. No closed/open toggles: YES — `project_intervals` and `reports_containing` remain `pub open spec fn` (unchanged). No pre-existing declaration's visibility modifier appears in the diff.

## Justification

I diffed `spec-frozen-sensor_poll_honest..HEAD -- exercises/sensor_poll_honest/` and confirmed only `main.rs` changed; `fusion.rs` and `auth.rs` are byte-identical to the freeze. Within `main.rs`, the additions are confined to (a) four new proof helpers between the two pre-existing `open spec fn`s and `poll`, and (b) `poll`'s body replacing the `unimplemented!()` stub. The frozen spec surface — both `open spec fn` bodies plus `poll`'s requires/ensures — appears only as context with no `+`/`-` markers, and no bypass token (`assume`, `external_body`, `unreachable!`, `panic!`, `assume_specification`) is introduced. The new proof helpers' `requires` clauses are genuine obligations the caller must discharge, not contract-trivializers.

## Reviewer notes

- The implementer followed the discovery-mode design note cleanly: one new lemma per genuinely new fact (projection-equiv, two range/finiteness facts, and pigeonhole intersection). No proof-engineering shortcuts.
- Worth flagging to the architect: the proof block at the end of `poll` (HEAD lines 207–236) explicitly re-asserts each conjunct of the honest-voter existential before closing — defensive but readable; future composition exercises may want this pattern documented as the recommended idiom for two-witness `exists|p, k| ...` clauses.
