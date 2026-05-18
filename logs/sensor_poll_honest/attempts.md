## Attempt 1 — 2026-05-18

**Sub-task:** Implement `poll` in `main.rs` (sub-task 3) — the only remaining
unfilled `unimplemented!()` body. `fusion.rs` and `auth.rs` were already
filled in (verbatim ports from `sensor_poll_signed`). End-to-end verify
(sub-task 4) is concurrent with this since `poll` is the entry point.

**Approach:** Ported the byte-equivalent `poll` skeleton from
`exercises/sensor_poll_signed/main.rs` (check_distinct → assert valid_bundle
→ build projected Vec<Interval> via loop → call marzullo → bridge to
reports-frame via the projection lemma `lemma_reports_eq_intervals_containing`
with empty body and `=~=` ensures). Discharged the **new** honest-voter
ensures clause via inclusion-exclusion pigeonhole, exactly as in
`ft_midpoint::lemma_pigeonhole_le` / `lemma_pigeonhole_ge`:

- `A := reports_containing(reports@, p_witness)` (supporters, `|A| >= n - f`)
- `B := correct_indices(reports.len())` (honest sensors, `|B| >= n - f`)
- Universe `U := set_int_range(0, n)` (size `n`)
- `(A + B) ⊆ U` ⇒ `|A ∪ B| <= n`
- Inclusion-exclusion `|A ∪ B| + |A ∩ B| = |A| + |B|` ⇒ `|A ∩ B| >= n - 2f >= 1`
- `axiom_is_empty_len0` + `axiom_is_empty` ⇒ extract witness `k ∈ A ∩ B`
- `k ∈ A` gives `point_in_interval(p_witness, reports[k].interval)`
- `k ∈ B` gives `correct_at(k)`

**Verifier output:** `verification results:: 16 verified, 0 errors`, exit 0.

**Next idea:** Done — hand to reviewer.
