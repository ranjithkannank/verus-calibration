# sensor_poll attempts

## Attempt 1 — 2026-05-17
**Sub-task:** All three sub-tasks (1: port marzullo into fusion.rs, 2: implement check_distinct in auth.rs, 3: implement poll in main.rs with projection lemma) executed in one attempt, since the design explicitly identifies near-verbatim ports from `exercises/marzullo.rs` and `exercises/quorum_cert.rs`.

**Approach:**
- `fusion.rs`: byte-for-byte port of marzullo's verified body and all proof helpers (`containing_upto`, `lemma_containing_in_range`, `lemma_containing_upto_in_range`, `lemma_correct_indices_in_range`, `lemma_containing_upto_extend`, `count_containing`, `lemma_max_lo_in_set`, `lemma_exists_supported_lo`, the loop with the trigger `#[trigger] intervals@[j2].lo` on the existential invariant). Spec-fns unchanged.
- `auth.rs`: bitmap-backed `check_distinct` lifted from `verify_qc_structure` in `quorum_cert.rs` (the bitmap half). Four-conjunct loop invariant: cursor bound, seen-vector length, pairwise-distinct prefix, bitmap-vs-prefix abstraction. Duplicate path returns false with a witness from invariant (d) showing `j0 < i` has the same `sensor_id` as `i`. Fall-through path captures the contrapositive of (d) before `seen.set`, then re-establishes (c) and (d) at the new `i`.
- `main.rs`: `poll` is three function calls. Call `check_distinct`; if false, return `None`. Project: walk `reports`, push each `.interval` into a fresh `Vec<Interval>`, prove `intervals@ =~= project_intervals(reports@)`. Call `marzullo(&intervals, f)`. Use `choose` to extract `p_witness`, then invoke `lemma_reports_eq_intervals_containing` (extensional-equality lemma, empty body) to bridge `intervals_containing(intervals@, p_witness)` to `reports_containing(reports@, p_witness)`. Return `Some(result)`.

**Verifier output:** `16 verified, 0 errors`. Only trigger-selection notes printed (no errors, no warnings about proof obligations).

**Next idea:** Done. Hand to reviewer.
