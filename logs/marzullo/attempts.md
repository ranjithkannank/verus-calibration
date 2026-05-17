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

## Attempt 3 — 2026-05-16
**Sub-tasks worked on:** 7 (main loop skeleton + structural invariant), 8 (threshold computation + overflow-safety assert), 9 (re-establish invariant in the fall-through branch).

**Approach:** Direct adaptation of `ft_midpoint`'s main loop:
- Overflow guard: `assert(f as nat + 1 <= u32::MAX as nat) by { assert(2*f + 1 <= u32::MAX as nat); };`
- `let n_u32: u32 = n as u32;` cast (Verus discharges via `intervals.len() <= u32::MAX as nat`), followed by `let threshold: u32 = n_u32 - f;` (subtraction safety from `n_u32 as nat >= f + 1`).
- Loop invariant has 8 conjuncts mirroring ft_midpoint, with the structural piece `forall|j2: int| 0 <= j2 < i ==> intervals_containing(intervals@, #[trigger] intervals@[j2].lo).len() < intervals.len() as nat - f as nat`. Explicit `#[trigger] intervals@[j2].lo` matches the post-loop witness shape (ft_midpoint pattern).
- Early-return branch: `if c >= threshold` returns `Interval { lo: p, hi: p }`. With `p = intervals[i].lo`, both `result.lo <= result.hi` (trivially) and the existential (witness `p`) close immediately.
- Fall-through branch: four-line proof block recording `c < threshold`, `c == intervals_containing(_, p).len()`, hence the structural invariant at `j2 = i`.

**Verifier output:** EXIT=1. **7 verified, 1 errors** (was 6 verified, 1 errors — so the main loop, both branches, and the threshold arithmetic all verified). The single remaining failure is the post-loop placeholder return, exactly as expected at this stage:
```
error: postcondition not satisfied
   --> .../marzullo.rs:221:9
221 | /         exists|p: Reading|
222 | |             result.lo <= p && p <= result.hi
223 | |                 && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat,
    | |_________________________________________________________________________________________________^ failed this postcondition
...
274 |       Interval { lo: 0, hi: 0 }
    |       ------------------------- at the end of the function body
```

**Next idea:** Sub-task 10 — add a `lemma_exists_supported_endpoint` stub (empty body) and confirm Verus reports its `exists` postcondition as the only remaining failure. If sub-task 11's argmax-over-correct-indices proof can't close the Helly step in 3 attempts, escalate per the design's prediction.

## Attempt 4 — 2026-05-16
**Sub-tasks worked on:** 10 (stub `lemma_exists_supported_endpoint` with empty body).

**Approach:** Added a `proof fn lemma_exists_supported_endpoint(intervals: Seq<Interval>, f: nat)` with the design's signature (requires `intervals.len() >= 2*f+1`, `well_formed`, and `correct_indices(n).len() >= n - f`; ensures `exists|j| 0 <= j < n && intervals_containing(intervals, intervals[j].lo).len() >= n - f`), `#[trigger]` placed on `intervals[j].lo` to match the main loop's invariant trigger. Body is empty as prescribed — the lemma's own existence postcondition is expected to fail until sub-task 11 attempts the real proof.

**Verifier output:** EXIT=1. **7 verified, 2 errors** (was 7 verified, 1 errors — same 7 verifications, plus the new lemma stub adds its expected unproven `exists` postcondition). Two failures, both expected:
```
error: postcondition not satisfied
   --> .../marzullo.rs:205:9
205 | /         exists|j: int|
206 | |             0 <= j < intervals.len()
207 | |             && intervals_containing(intervals, #[trigger] intervals[j].lo).len()
208 | |                >= intervals.len() - f,
    | |_____________________________________^ failed this postcondition

error: postcondition not satisfied
   --> .../marzullo.rs:243:9
... (same marzullo postcondition as before)
```

**Next idea:** Sub-task 11 — attempt the proof of `lemma_exists_supported_endpoint` via argmax-over-correct-indices + Helly-1D. The design predicts this will block on the Helly step (the spec lacks the `correct_at(i) && correct_at(j) ==> intervals[i].lo <= intervals[j].hi` precondition). Try the natural shape first; if three consecutive attempts on the Helly obligation fail, write `escalation.md` per the design.

## Attempt 5 — 2026-05-16
**Sub-tasks worked on:** 11 (attempt the natural argmax-over-correct-indices + Helly-1D proof of `lemma_exists_supported_endpoint`).

**Approach:** Direct adaptation of ft_midpoint's `lemma_max_reading_in_set` to interval-`lo` keys:
- Added `lemma_max_lo_in_set(s: Set<int>, intervals: Seq<Interval>) -> (jm: int)` — argmax of `intervals[j].lo` over a finite index set, by recursion on `s.len()`.
- Replaced the empty body of `lemma_exists_supported_endpoint` with: argmax `jm` over `correct_indices(n)`, then prove `correct_indices(n).subset_of(intervals_containing(intervals, intervals[jm].lo))` by showing each correct `k` has both `intervals[k].lo <= intervals[jm].lo` (argmax property) and `intervals[jm].lo <= intervals[k].hi` (the Helly-1D step). Then `lemma_len_subset` closes `intervals_containing.len() >= correct_indices.len() >= n - f`.

**Verifier output:** EXIT=1. **8 verified, 2 errors** (was 7 verified, 2 errors — so `lemma_max_lo_in_set` verified, and most of the existence lemma scaffolding verified; the single new failure is exactly the predicted Helly step):
```
error: assertion failed
   --> .../marzullo.rs:292:20
292 |             assert(intervals[jm].lo <= intervals[k].hi);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ assertion failed
```
The remaining error is still the `marzullo` postcondition (blocked on the same lemma).

**Next idea:** This is attempt 1 on the Helly obligation; per the design (cap of 3 consecutive attempts on the same obligation before escalation) I have two more shots. Candidate variants to try next:
- (Attempt 6) Argmin over `correct_indices` of `intervals[j].hi` instead of argmax of `intervals[j].lo` — but the postcondition specifically picks `intervals[j].lo` as the candidate point, so the change of pivot doesn't shift the gap; both directions need the Helly link.
- (Attempt 7) Try to derive the Helly fact directly from the existing preconditions via a different angle (e.g., pigeonhole on the union of correct intervals against `[lo, hi]` of `intervals[jm]`) — but `correct_at` is uninterpreted with no relational hypothesis, so any such derivation must fail.
If both variants land on the same `lo <= hi` cross-interval obligation, write `escalation.md` confirming the spec gap.

## Attempt 6 — 2026-05-16
**Sub-tasks worked on:** Escalation (no code changes).

**Approach:** Wrote `logs/marzullo/escalation.md` instead of trying further variants on the Helly obligation. Rationale:
- Attempt 5's argmax-over-correct-indices is the natural proof shape, and it failed at exactly the Helly step the design predicted.
- I have a **constructive counterexample** from the design's "Critical caveat" (intervals=[[0,0],[10,10],[20,20]], f=1, n=3, with correct_at(0)=correct_at(1)=true): all preconditions hold but no point lies in ≥ n−f = 2 intervals. So the postcondition is structurally false under the frozen preconditions, and no variant — argmin on `hi`, different candidate point, Byzantine-aware pigeonhole — can avoid the same gap (the variants section of the escalation note enumerates them).
- The implementer.md "3 consecutive attempts on same obligation before escalation" rule is an upper bound; AGENTS.md rule #4 says "Never weaken a spec to make verification pass." With a proof of unprovability in hand, escalating now respects both rules.

**Verifier output:** EXIT=1. **8 verified, 2 errors** (unchanged from attempt 5 — no code changed). Same two failures:
```
error: assertion failed
   --> exercises/marzullo.rs:292:20
292 |             assert(intervals[jm].lo <= intervals[k].hi);
error: postcondition not satisfied
   --> exercises/marzullo.rs:334:9 (marzullo)
```

**Next idea:** Escalated. Orchestrator should re-invoke the architect to either (a) add the missing `correct_intervals_overlap` Helly-1D precondition to the frozen `marzullo` spec, or (b) mark the exercise blocked-by-spec-gap. The verified-up-to-the-Helly-step machinery (`count_containing`, `lemma_max_lo_in_set`, structural subset/finiteness lemmas, the main loop's trigger-aligned invariant) remains intact and would close immediately once the precondition is in place. Full analysis in `logs/marzullo/escalation.md`.
