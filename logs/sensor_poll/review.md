# Review: sensor_poll

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — `git diff spec-frozen-sensor_poll..HEAD -- exercises/sensor_poll` shows no hunk modifying any `requires` or `ensures` clause. The `check_distinct` (auth.rs lines 17–35 pre-image, unchanged), `marzullo` (fusion.rs lines 54–72 pre-image, unchanged), and `poll` (main.rs lines 35–58 pre-image, unchanged) signatures and clauses are byte-identical; only their bodies (each previously `unimplemented!()`) were filled in.
2. Pre-existing spec fn bodies unchanged: YES — `distinct_sensors` (auth.rs), `well_formed` / `point_in_interval` / `intervals_containing` / `correct_indices` / `correct_intervals_overlap` (fusion.rs), `correct_at` (uninterp, fusion.rs), and `project_intervals` / `reports_containing` (main.rs) are not touched by any diff hunk. New helper `spec fn containing_upto` in fusion.rs (added at the top of the new "Proof-only spec helpers" section) is a fresh addition, which is permitted.
3. No bypass tokens introduced: YES — grep for `assume(|external_body|unimplemented!|unreachable!|panic!|assume_specification` over `exercises/sensor_poll` returns zero matches. The three pre-existing `unimplemented!()` placeholders were removed in favor of real bodies (auth.rs:34, fusion.rs:70, main.rs:55 in the pre-image), not replaced with panicking stubs.
4. No trivializing requires: YES — no new `requires` clauses are added to the three pre-spec'd `exec`/`pub fn` declarations (`check_distinct`, `marzullo`, `poll`). New helpers have only sensible bounds: `count_containing` requires `intervals.len() <= u32::MAX as nat`; `lemma_containing_upto_in_range` requires `0 <= m <= intervals.len()`; `lemma_containing_upto_extend` requires `0 <= i < intervals.len()`; `lemma_max_lo_in_set` requires non-empty + in-range; `lemma_exists_supported_lo` mirrors marzullo's frozen preconditions. None are trivializing.
5. No closed/open toggles: YES — `pub open spec fn` modifiers on `well_formed`, `point_in_interval`, `intervals_containing`, `correct_indices`, `correct_intervals_overlap`, `distinct_sensors`, `project_intervals`, `reports_containing` are unchanged. The newly added internal helpers `containing_upto` and the `proof fn`s are plain `spec fn`/`proof fn` declarations, not toggles of existing ones.

## Justification

I diffed `spec-frozen-sensor_poll..HEAD` for the entire `exercises/sensor_poll/` tree and inspected every hunk. All three frozen function signatures retained their exact `requires`/`ensures` text; the only changes are body replacements of `unimplemented!()` and additions of new proof scaffolding (helper `spec fn`, finiteness/subset/extend/argmax lemmas in fusion.rs, the empty-body projection lemma in main.rs). I confirmed no bypass tokens exist anywhere in the module via Grep. All new `requires` are on freshly-added helpers and reflect natural invariants (range bounds, mirrors of marzullo's preconditions for the existence lemma), not service-of-proof trivializations.

## Reviewer notes (optional)

- The empty-body `=~=` projection lemma pattern (`lemma_reports_eq_intervals_containing` in main.rs:42-47) is a clean composition idiom worth promoting in the playbook — the seam between two BFT primitives stated in different domains closes on a single extensional-equality identity.
- The bitmap-backed `check_distinct` in auth.rs mirrors the `verify_qc_structure` pattern from `quorum_cert`; the re-establishment blocks for invariants (c) and (d) after `seen.set` (auth.rs lines ~73–115 in HEAD) are the right shape and could be templated.
