## Attempt 1 — 2026-05-17

**Sub-task:** Implement `poll`'s body end-to-end in `main.rs`, including the
new honest-voter clause discharge. Sub-tasks 1 (port `marzullo`) and 2
(port `check_distinct`) were already complete on the way in (`fusion.rs`
and `auth.rs` already filled out).

**Approach:** Ported `poll` body verbatim from
`exercises/sensor_poll_signed/main.rs` (check_distinct → bundle assert →
projection loop → marzullo → projection-lemma bridge giving
`reports_containing(reports@, p_witness).len() >= n - f`). Added the new
honest-voter clause via a pigeonhole lemma
`lemma_honest_supporter_exists(reports, p, f)`:

1. `s = reports_containing(reports, p)` and `c = correct_indices(n)` are
   each subsets of `set_int_range(0, n)`.
2. Standard subset/finiteness lemma chain (`lemma_int_range`,
   `lemma_len_subset`) makes both finite with cardinality bounds.
3. `s ∪ c ⊆ [0, n)` so `|s ∪ c| <= n`.
4. `lemma_set_intersect_union_lens(s, c)` plus `s + c =~= s.union(c)`
   gives `|s ∩ c| + |s ∪ c| == |s| + |c|`.
5. Substituting the lower bounds (`|s|, |c| >= n - f`, `|s ∪ c| <= n`,
   `n >= 2f + 1`) yields `|s ∩ c| >= n - 2f >= 1`.
6. `axiom_is_empty_len0` + `axiom_is_empty` pull a witness `k` out of
   the non-empty intersection. `s.contains(k)` and `c.contains(k)`
   unfold to the honest-voter conjunction.

Then in `poll`'s proof block, after the existing
`p_witness` selection and the bridge to `reports_containing`, call
`lemma_honest_supporter_exists(reports@, p_witness, f as nat)` and
`choose` a `k_witness`. The two-existential ensures clause closes from
the conjunction of the `p_witness`/`k_witness` facts.

**Verifier output:**

```
verification results:: 19 verified, 0 errors
```

(No errors; remaining lines are auto-trigger notes only.)

**Next idea:** Done. Hand off to reviewer.
