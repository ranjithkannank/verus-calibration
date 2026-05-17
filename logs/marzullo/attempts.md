# Attempts log for `marzullo`

## Attempt 1 — 2026-05-16
**Sub-task:** All of 1-10 in one pass — full port from ft_midpoint, with the
Marzullo-specific argmax-`lo` + Helly-1D existence lemma replacing
ft_midpoint's contradiction-via-inclusion-exclusion existence proof.

**Approach:** Direct implementation following the architect's design verbatim:
  - Added `containing_upto` spec helper.
  - Ported L1 (subset/finiteness) lemmas: `lemma_containing_in_range`,
    `lemma_containing_upto_in_range`, `lemma_correct_indices_in_range`.
  - Ported L2 `lemma_containing_upto_extend` (empty body, two `=~=` ensures).
  - Wrote `count_containing` mirroring `count_le` from ft_midpoint, reading
    `iv.lo` and `iv.hi` separately to avoid `Copy` issues on `Interval`.
  - Ported L3 argmax `lemma_max_lo_in_set`.
  - Wrote the simpler L4 `lemma_exists_supported_lo` (constructive: argmax
    over correct indices gives p = intervals[jm].lo, then Helly-1D + argmax
    show every correct k contains p).
  - Wrote `marzullo` main loop with strengthened invariant (no earlier
    candidate worked), in-loop early return on threshold hit, post-loop
    `assert(false)` via the existence-lemma witness.

**Verifier output:** `verification results:: 10 verified, 0 errors` (exit 0).
Only auto-trigger info notes printed; no errors.

**Next idea:** Done — hand to reviewer.
