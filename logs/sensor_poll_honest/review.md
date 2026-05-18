# Review: sensor_poll_honest

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — the diff against `spec-frozen-sensor_poll_honest` touches only `exercises/sensor_poll_honest/main.rs`. Both hunks treat lines 51–76 (the full `requires`/`ensures` block of `poll`) as context only — there are no `+`/`-` lines inside `requires` or `ensures`. `auth.rs` and `fusion.rs` produce no diff at all.
2. Pre-existing spec fn bodies unchanged: YES — the only pre-existing `spec fn`s in `main.rs` are `project_intervals` (lines 29–31) and `reports_containing` (lines 33–36); both appear unchanged as context in the diff. `auth.rs` and `fusion.rs` spec fns have no diff. The new `lemma_reports_eq_intervals_containing` at diff lines +44..+49 is a `proof fn`, not a spec fn.
3. No bypass tokens introduced: YES — the diff removes the lone `unimplemented!()` (deletion line `-    unimplemented!()` in the second hunk) and adds no `assume(`, `#[verifier::external_body]`, `unimplemented!()`, `unreachable!()`, or `panic!(` tokens. Grep over the added lines (+78..+187) shows only `assert(...)`, `assert forall ... by`, and lemma calls (`lemma_int_range`, `lemma_len_subset`, `lemma_set_intersect_union_lens`, `axiom_is_empty_len0`, `axiom_is_empty`).
4. No trivializing requires: YES — the new helper `lemma_reports_eq_intervals_containing` (diff +44..+49) has only an `ensures` clause and no `requires`. `poll` itself gains no new `requires` lines (lines 52–60 are context-only).
5. No closed/open toggles: YES — `project_intervals` and `reports_containing` remain `pub open spec fn` (unchanged context). The new helper is `proof fn`, not `spec fn`, so no opacity question arises.

## Justification

I diffed HEAD against `spec-frozen-sensor_poll_honest` over the entire `exercises/sensor_poll_honest/` directory. Only `main.rs` changed, with two hunks: (1) insertion of an empty-body proof lemma `lemma_reports_eq_intervals_containing` between the existing spec fns and `poll`, and (2) replacement of `unimplemented!()` with a real body. Both the frozen `requires`/`ensures` block and the two pre-existing `open spec fn` bodies appear verbatim as context lines, never as `+`/`-`. The new lemma is a clean reusable bridge from `reports_containing` to `intervals_containing ∘ project_intervals` (closed by `=~=` extensionality), and the new `poll` body discharges the honest-voter clause via a textbook inclusion-exclusion argument on `A = reports_containing` and `B = correct_indices` inside `U = [0, n)`.

## Reviewer notes (optional)

- The inclusion-exclusion construction (lines 132–187 of `main.rs`) is a clean reuse of the `ft_midpoint` set-cardinality playbook applied at the system-composition level — worth keeping in the patterns file as the canonical honest-voter discharge pattern.
- The empty-body extensional-equality lemma pattern (lines 44–49) continues to work across `sensor_poll`, `sensor_poll_signed`, and now `sensor_poll_honest` with no modification; a single playbook entry covers all three.
