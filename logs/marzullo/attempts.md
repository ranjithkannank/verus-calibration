# marzullo attempts

## Attempt 1 — 2026-05-16
**Sub-tasks worked on:** 1 (stub `marzullo` body), 2 (add `contained_set_upto` spec helper), 3 (`lemma_contained_set_in_range`), 4 (`lemma_contained_set_upto_in_range`), 5 (`lemma_contained_set_upto_extend`). Also added `lemma_correct_indices_in_range` (will be needed downstream).

**Approach:** Built the structural foundation by direct adaptation from `ft_midpoint`:
- Added the `contained_set_upto` prefix-set spec helper (`intervals_containing` already exists in the frozen spec — used directly in place of the design's `contained_set`).
- Added three subset/finiteness lemmas (`lemma_contained_set_in_range`, `lemma_contained_set_upto_in_range`, `lemma_correct_indices_in_range`) and one prefix-extend lemma (`lemma_contained_set_upto_extend`). All four bodies are near-empty, using `lemma_int_range` + `lemma_len_subset` (subset cases) or `=~=` (extend case).
- Stubbed `marzullo` to return `Interval { lo: 0, hi: 0 }`. First sub-attempt used `intervals[0]` but `Interval` doesn't implement `Copy`; switched to a direct construction.

**Verifier output:** EXIT=1.
```
verification results:: 4 verified, 1 errors
error: postcondition not satisfied
   --> exercises/marzullo.rs:176:9
    |
176 | /         exists|p: Reading|
177 | |             result.lo <= p && p <= result.hi
178 | |                 && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat,
    | |_________________________________________________________________________________________________^ failed this postcondition
...
183 |       Interval { lo: 0, hi: 0 }
    |       ------------------------- at the end of the function body
```
All four structural lemmas verified cleanly. The only failure is the `marzullo` postcondition on the stub body — exactly the expected outcome for sub-task 1.

**Next idea:** Sub-task 6 — implement `count_containing` with its prefix-set loop invariant, mirroring `count_le` from ft_midpoint.

## Attempt 2 — 2026-05-16
**Sub-tasks worked on:** 6 (`count_containing` exec function with prefix-set loop invariant).

**Approach:** Direct adaptation of `count_le` from `ft_midpoint`:
- Loop iterates `i: usize` from 0 to `intervals.len()`.
- Invariant: `c as nat == contained_set_upto(intervals@, p, i as int).len()` plus finiteness and cursor bounds.
- Per-iteration uses `lemma_contained_set_upto_extend` and `lemma_contained_set_upto_in_range` to bridge `i` → `i+1`.
- On the "interval contains p" branch, defensive `assert(!contained_set_upto(_).contains(i))` before increment.
- Post-loop `=~=` bridge: `contained_set_upto(intervals@, p, intervals@.len() as int) =~= intervals_containing(intervals@, p)`, then `lemma_contained_set_in_range` for finiteness of the final set.

**Verifier output:** EXIT=1. 6 verified, 1 errors (was 4 verified, 1 errors — so `count_containing` and one auto-generated check verified cleanly). The only remaining failure is the still-stubbed `marzullo` postcondition:
```
error: postcondition not satisfied
   --> .../marzullo.rs:221:9
221 | /         exists|p: Reading|
222 | |             result.lo <= p && p <= result.hi
223 | |                 && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat,
    | |_________________________________________________________________________________________________^ failed this postcondition
...
228 |       Interval { lo: 0, hi: 0 }
    |       ------------------------- at the end of the function body
```

**Next idea:** Sub-task 7/8/9 — wire up the main loop skeleton with the threshold computation, overflow-safety assert, and the structural loop invariant tracking `intervals_containing(intervals@, intervals@[j2].lo).len() < threshold` for `j2 < i`. The in-loop return branch should close immediately; the fall-through branch re-establishes the invariant; the post-loop `assert(false)` is left dangling pending sub-task 10/11 (the existence lemma, which is expected to block).
