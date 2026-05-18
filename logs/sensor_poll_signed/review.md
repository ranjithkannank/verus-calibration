# Review: sensor_poll_signed

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — diff hunks on `check_distinct` (auth.rs lines 75-77), `marzullo` (fusion.rs lines 269-271), and `poll` (main.rs lines 51-69) show only body insertions; the `requires`/`ensures` clauses are untouched. New `proof fn`/`fn` declarations (`lemma_containing_in_range`, `lemma_containing_upto_in_range`, `lemma_correct_indices_in_range`, `lemma_containing_upto_extend`, `count_containing`, `lemma_max_lo_in_set`, `lemma_exists_supported_lo`, `lemma_reports_eq_intervals_containing`) carry their own `requires`/`ensures` but are fresh helpers, not modifications.
2. Pre-existing spec fn bodies unchanged: YES — all frozen spec fns (`distinct_sensors`, `all_signatures_valid`, `valid_report_bundle`, `pk_of`, `signature_valid`, `report_msg`, `well_formed`, `point_in_interval`, `intervals_containing`, `correct_indices`, `correct_intervals_overlap`, `correct_at`, `project_intervals`, `reports_containing`) appear nowhere in the diff hunks; the new `spec fn containing_upto` (fusion.rs lines 59-61) is a fresh proof-only helper.
3. No bypass tokens introduced: YES — `grep` for `assume(|external_body|unimplemented!|unreachable!|panic!|assume_specification` over `*.rs` returns no matches; the three frozen `unimplemented!()` stubs were replaced by real bodies (auth.rs line 78+, fusion.rs line 272+, main.rs line 71+).
4. No trivializing requires: YES — the new exec helper `count_containing` (fusion.rs line 115) has `requires intervals.len() <= u32::MAX as nat`, a non-trivial overflow guard mirroring the parent function. No `requires false` or vacuous preconditions. No new requires were added to any pre-existing exec function.
5. No closed/open toggles: YES — no existing `open spec fn` or `closed spec fn` declaration changed visibility; `containing_upto` is a newly-introduced unqualified `spec fn` (proof-only helper), which is a fresh declaration rather than a toggle.

## Justification

I diffed `spec-frozen-sensor_poll_signed..HEAD` over `exercises/sensor_poll_signed/` and confirmed the diff is purely additive in code positions: function bodies that previously held `unimplemented!()` now contain implementations, and new proof helpers were added alongside. No frozen text was modified or removed. A direct grep over the `*.rs` files in the directory turns up zero cheat tokens. The new helper `count_containing`'s `requires` clause is a standard u32-overflow guard, and the proof-only lemmas have only ghost-side ensures. The implementation strategy — port marzullo's proof skeleton verbatim into `fusion.rs`, lift the bitmap distinct-check into `auth.rs`, and stitch via `lemma_reports_eq_intervals_containing` and a single `assert(valid_report_bundle(reports@))` — matches the design note's intent without touching the spec surface.

## Reviewer notes (optional)

- The signed variant adds the trust-boundary precondition `all_signatures_valid` and conjunction-postcondition `valid_report_bundle` cleanly: the exec layer never reads `sig`, and the proof for the new postcondition collapses to one assert, exactly as the architect intended.
- The byte-for-byte fusion.rs port from `sensor_poll` plus the empty-body `=~=` projection lemma in main.rs is a tight, reproducible pattern — worth filing as the canonical "compose-with-frozen-spec" template for future multi-module exercises.
