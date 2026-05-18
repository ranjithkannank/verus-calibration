# Review: vec_swap

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — The diff hunk at `exercises/vec_swap.rs` lines 34–35 shows the only context-line change involves replacing the function body. The `requires` clauses (`i < old(v)@.len()`, `j < old(v)@.len()`) and the five `ensures` clauses (len-preservation, two positional swap clauses, the frame `forall|k|`, and `final(v)@.to_multiset() == old(v)@.to_multiset()`) appear only as unchanged context lines in the diff. No `+`/`-` markers touch any line under `requires` or `ensures`.
2. Pre-existing spec fn bodies unchanged: YES — `spec-frozen-vec_swap` declares no `spec fn` in this file; the diff adds none. Only `pub fn swap(...)` exists.
3. No bypass tokens introduced: YES — The diff removes `unimplemented!()` (line `-    unimplemented!()`) and replaces it with concrete exec code plus a `proof { }` block. No `assume(`, no `#[verifier::external_body]`, no `unreachable!()`, no `panic!(`, no `assume_specification`. The `broadcast use group_to_multiset_ensures, group_multiset_axioms, group_multiset_properties;` on diff lines 42–43 brings vstd-verified lemmas into scope and is not a bypass. All `assert` / `assert forall ... by { ... }` uses (diff lines 47, 50–51, 53, 56–67) are legitimate proof aids.
4. No trivializing requires: YES — No `requires` clauses added to `swap` or any new helper. No new functions were introduced at all.
5. No closed/open toggles: YES — No `closed spec` or `open spec` declarations exist in the file; nothing to toggle.

## Justification

I ran `git diff spec-frozen-vec_swap..HEAD -- exercises/vec_swap.rs` and inspected every hunk. The single hunk (lines 34–73 of the new file) replaces only `unimplemented!()` with two `v.set(...)` exec calls and a `proof { }` block containing broadcast-uses and `assert` / `assert forall ... by` statements. The frozen `requires`/`ensures` clauses appear only as unchanged context. No spec fns existed in the baseline and none were added. No bypass tokens, no new `requires`, no spec visibility toggles. The proof structure mirrors the count-by-element pattern documented in AGENTS.md (vec_swap notes), with explicit `i == j` case split — clean implementation.

## Reviewer notes (optional)

- The `broadcast use` of vstd's verified groups (`group_to_multiset_ensures`, `group_multiset_axioms`, `group_multiset_properties`) is a legitimate use of vstd-provided lemma surface, not a trust-boundary bypass; future reviewers should not flag broadcast-uses of vstd groups.
- The witness file gave the implementer the proof skeleton verbatim, per the AGENTS.md note. If future exercises aim to measure pure invention, hiding the witness from the implementer would be needed.
