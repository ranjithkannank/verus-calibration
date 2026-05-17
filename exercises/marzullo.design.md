# Design: marzullo.rs

## Critical caveat — read this first

The frozen postcondition is a **purely structural property of
`intervals@`**:

```
exists|p: Reading|
    result.lo <= p && p <= result.hi
        && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat
```

It never mentions `correct_at`. The only precondition that touches
`correct_at` is its *count* (`correct_indices(n).len() >= n - f`). The
only constraints on `intervals@` are `well_formed` (each interval has
`lo <= hi`) and the size bound `n >= 2f + 1`.

Because `correct_at` is `uninterp`, the verifier reasons over all
interpretations. The postcondition's `p` is independent of
`correct_at`, so the spec writer's prose justification ("the n - f
correct intervals all overlap, since they all contain any 'true'
value") is **not derivable from the spec** — that overlap assumption
is stated only in the prose comment, not in `requires`.

> **Counterexample to provability.** Take `intervals = [[0,0],
> [10,10], [20,20]]`, `f = 1`, `n = 3`. Pick `correct_at(0) =
> correct_at(1) = true`, `correct_at(2) = false`. Then
> `correct_indices.len() = 2 = n - f`, `n >= 2f + 1 = 3`, and
> well-formedness holds (each interval has `lo == hi`). All
> preconditions are satisfied, but no point lies in two of these
> intervals: `intervals_containing(p).len() <= 1` for every `p`, so
> the postcondition cannot hold.

The likely fix (out of scope — spec is frozen) is to add a Helly-1D
precondition linking `correct_at` to intervals, e.g.

```
forall|i: int, j: int|
    0 <= i < n && 0 <= j < n && correct_at(i) && correct_at(j)
        ==> intervals[i].lo <= intervals[j].hi
```

This is missing.

**Implementer guidance.** Build the brute-force machinery anyway: the
exec scan, the `intervals_containing` prefix-set abstraction, the
counting loop, and the structural pieces (subset/finiteness lemmas,
inclusion-exclusion). The only thing that *should* fail to discharge
is the existence lemma (§Helper Lemmas item 5 below). When you hit
it, do not `assume` it and do not weaken the spec. Try the brute-force
proof structure honestly; if it cannot close, write
`logs/marzullo/escalation.md` and let the architect re-confirm the
structural gap. If re-confirmed, exit via `blocked.md`.

## Representation choice

No new struct. `intervals: &Vec<Interval>` is read-only, so the exec
work is purely a scan with counting. The result is built directly as
`Interval { lo: p, hi: p }` — the spec's `result.lo <= result.hi`
trivially holds, and the existential witness is `p` itself.

Why not a non-trivial interval (`lo < hi`)? Because the spec only
requires *existence* of a supported `p` in `[lo, hi]`, and a
single-point interval makes both the equality `lo <= p <= hi`
trivial and the witness obvious. There is zero proof advantage to a
wider interval.

## Algorithmic sketch

```
fn marzullo(intervals, f):
    let n = intervals.len();
    let threshold = (n - f) as u32;
    // Scan candidate points = intervals[i].lo for i in [0, n).
    for i in 0..n:
        let p = intervals[i].lo;
        let c = count_containing(intervals, p);
        if c >= threshold:
            return Interval { lo: p, hi: p };
    // Provably unreachable (modulo the existence lemma).
    assert(false);
    return intervals[0];
```

`intervals[i].lo` is a sound candidate set because, if any point `q`
is contained in `k` intervals, then the largest `intervals[i].lo`
among those `k` is also contained in all `k` of them (each of those
intervals contains `q`, and their `lo <= q`, and the largest `lo` is
still `<= q` while being `>= lo_i` for each). Translation: scanning
input-`lo` endpoints suffices to find a maximal-overlap candidate.

## Spec helpers (proof-only, new in this file)

Mirror `ft_midpoint`'s prefix-set abstraction:

```rust
spec fn contained_set(intervals: Seq<Interval>, p: Reading) -> Set<int> {
    Set::new(|i: int| 0 <= i < intervals.len() && point_in_interval(p, intervals[i]))
}

spec fn contained_set_upto(intervals: Seq<Interval>, p: Reading, m: int) -> Set<int> {
    Set::new(|i: int|
        0 <= i < m && i < intervals.len() && point_in_interval(p, intervals[i]))
}
```

Note `contained_set` is extensionally equal to `intervals_containing`
(the spec helper). The implementer should keep `intervals_containing`
as the externally visible name and use `contained_set` only if
internal-naming hygiene helps — they can also just inline the
`Set::new` in helper lemmas.

## Key invariants

Loop invariant for `count_containing` (analogous to ft_midpoint's
`count_le`):

```
0 <= i as int <= intervals@.len() as int,
intervals.len() <= u32::MAX as nat,
c as nat == contained_set_upto(intervals@, p, i as int).len(),
contained_set_upto(intervals@, p, i as int).finite(),
c as nat <= i as nat,
```

Loop invariant for the main `marzullo` scan (mirrors ft_midpoint's
final loop):

```
0 <= i as int <= n as int,
n == intervals@.len(),
threshold as nat == n as nat - f as nat,
intervals.len() <= u32::MAX as nat,
intervals.len() as nat >= 2 * (f as nat) + 1,
well_formed(intervals@),
correct_indices(intervals.len() as nat).len() >= intervals.len() as nat - f as nat,
forall|j2: int| 0 <= j2 < i as int ==>
    intervals_containing(intervals@, #[trigger] intervals@[j2].lo).len()
        < intervals.len() as nat - f as nat,
```

Note the explicit `#[trigger] intervals@[j2].lo` — this is the
ft_midpoint trigger-matching trick. Without it, the post-loop
instantiation of the loop invariant at the witness index from the
(would-be) existence lemma will not fire.

## Helper lemmas predicted

1. `lemma_contained_set_in_range(intervals, p)` — `contained_set` is
   a subset of `set_int_range(0, n)`, hence finite, hence
   `.len() <= n`. Modelled on `lemma_le_set_in_range` from
   ft_midpoint.

2. `lemma_contained_set_upto_in_range(intervals, p, m)` — same for
   the prefix variant. Modelled on `lemma_le_set_upto_in_range`.

3. `lemma_contained_set_upto_extend(intervals, p, i)` — prefix
   extension:
   ```
   point_in_interval(p, intervals[i]) ==>
       contained_set_upto(intervals, p, i+1)
           =~= contained_set_upto(intervals, p, i).insert(i),
   !point_in_interval(p, intervals[i]) ==>
       contained_set_upto(intervals, p, i+1)
           =~= contained_set_upto(intervals, p, i),
   ```
   Body should be empty (`=~=` should close on its own); if not, an
   `assert forall|x: int| ... by {}` block bridges. Modelled on
   `lemma_le_set_upto_extend`.

4. `count_containing(intervals: &Vec<Interval>, p: Reading) -> (c:
   u32)` — exec counting function, postcondition `c as nat ==
   contained_set(intervals@, p).len()`. Body is the prefix-set loop
   from `count_le` adapted to use `point_in_interval` instead of
   `<=`.

5. **`lemma_exists_supported_endpoint(intervals: Seq<Interval>, f:
   nat)`** — the load-bearing existence lemma. Signature:
   ```
   proof fn lemma_exists_supported_endpoint(intervals: Seq<Interval>, f: nat)
       requires
           intervals.len() >= 2 * f + 1,
           well_formed(intervals),
           correct_indices(intervals.len()).len() >= intervals.len() - f,
       ensures
           exists|j: int|
               0 <= j < intervals.len()
               && intervals_containing(intervals, #[trigger] intervals[j].lo).len()
                  >= intervals.len() - f,
   ```
   **This is the proof that should not close from the frozen
   preconditions** (see Critical caveat). The natural shape — pick
   `j = argmax over correct indices of intervals[j].lo`, show every
   correct interval contains `intervals[j].lo` — requires the
   missing Helly-1D assumption to bound `intervals[j].lo <=
   intervals[k].hi` for every correct `k != j`. Without it, that
   step is unsound.

## SMT trouble spots

- **Existence lemma #5** is the main wall. Expect 3 consecutive
  failed attempts on the same `forall|k: int| correct_at(k) ==>
  intervals[k].hi >= intervals[j].lo` step; that's the escalation
  trigger.

- **`u32` arithmetic for the threshold.** `n - f` must be computed
  as `u32`. The precondition gives `intervals.len() <= u32::MAX as
  nat` and `n >= 2f + 1 >= f`, so `n - f >= f + 1 >= 1` and fits in
  `u32`. Mirror the overflow guard from ft_midpoint:
  ```
  assert(f as nat + 1 <= u32::MAX as nat) by {
      assert(2 * (f as nat) + 1 <= u32::MAX as nat);
  };
  ```
  Compute `let threshold: u32 = (n as u32) - f;`.

- **Frame on `c = c + 1`.** Standard — defensive
  `assert(!contained_set_upto(intervals@, p, i as int).contains(i as int))`
  before the increment, mirroring `count_le`.

- **`=~=` for the post-loop bridge** in `count_containing`:
  ```
  assert(contained_set_upto(intervals@, p, intervals@.len() as int)
      =~= contained_set(intervals@, p));
  ```
  matches `ft_midpoint`'s pattern verbatim.

- **Trigger on `intervals@[j2].lo` in the main loop invariant.** Make
  sure the trigger is on `intervals@[j2].lo` (not `intervals@[j2]`),
  because the post-loop witness chosen from the existence lemma will
  also be `intervals@[jw].lo`. Mismatch here means SMT never
  instantiates.

- **Post-loop `assert(false)` requires the existence lemma to fire.**
  If lemma #5 cannot be proved, the post-loop unreachable claim also
  fails. They block as a pair.

## Suggested order of operations

1. Stub `marzullo` returning `intervals[0]` with empty body and a
   stub `assert(false); ` (placeholder for unreachable) — confirm the
   file parses but verification fails as expected.
2. Add the prefix-set spec helpers `contained_set` /
   `contained_set_upto`.
3. Prove the three structural lemmas (`lemma_contained_set_in_range`,
   `lemma_contained_set_upto_in_range`,
   `lemma_contained_set_upto_extend`). These should all be near-empty
   bodies thanks to `=~=`.
4. Write `count_containing` with its loop invariant (copy-and-adapt
   from `count_le`). Should verify cleanly.
5. Wire up the main loop with the structural invariant
   (`forall|j2 < i| contained_set_upto.len() < threshold`). The
   "return on hit" branch should close.
6. **Now attempt `lemma_exists_supported_endpoint`.** This is where
   the spec gap bites. Try the natural argmax-over-correct-indices
   shape; if the Helly step can't close, write `escalation.md`.
7. (If #6 closes, miraculously) Connect the existence lemma at the
   post-loop point and discharge the final `assert(false)`.

## Sub-tasks

1. Stub `marzullo` returning a placeholder and confirm the file
   parses.
2. Add `contained_set` and `contained_set_upto` spec helpers.
3. Prove `lemma_contained_set_in_range` (subset + finiteness +
   length bound).
4. Prove `lemma_contained_set_upto_in_range` (same shape, prefix
   variant).
5. Prove `lemma_contained_set_upto_extend` (insert / no-op via
   `=~=`).
6. Implement `count_containing` with its loop invariant, mirroring
   `count_le`. Confirm it verifies.
7. Write the main loop skeleton in `marzullo` with the structural
   loop invariant; verify the in-loop return branch closes (assumes
   #5/#6 are landed).
8. Add the threshold computation and overflow-safety assert before
   the loop.
9. Re-establish the loop invariant in the fall-through branch of
   each iteration (the `count < threshold` recording step).
10. Stub `lemma_exists_supported_endpoint` with an empty proof body
    and confirm Verus reports the existence as the only remaining
    failure.
11. Attempt the proof of `lemma_exists_supported_endpoint` via
    argmax-over-correct-indices + Helly step. **Expected to block
    here.** If three consecutive attempts on the Helly step fail,
    write `logs/marzullo/escalation.md` describing the spec gap.
12. (Conditional on #11 closing) Use the lemma at the post-loop
    point, extract the witness, contradict the invariant, discharge
    `assert(false)`.

## Summary: brute-force endpoint scan with prefix-set counting, mirroring ft_midpoint; expected to block on the existence lemma because the frozen spec lacks the Helly-1D "correct intervals overlap" precondition the algorithm needs.

## Revision (escalation 20260517T044650Z)

### What the implementer reported

Attempt 5 confirmed the design's "Critical caveat" exactly. The
implementer landed all structural machinery (prefix-set abstraction,
`count_containing` with its prefix-set invariant, the three
subset/finiteness lemmas, `lemma_max_lo_in_set` argmax recursion, and
the main scan loop's trigger-aligned invariant) and reached
`lemma_exists_supported_endpoint`. The single remaining failure is

```
error: assertion failed
   --> exercises/marzullo.rs:292:20
292 |             assert(intervals[jm].lo <= intervals[k].hi);
```

which is the Helly-1D bound the design flagged as unprovable. The
implementer additionally supplied a constructive counterexample to
the postcondition itself — three singleton intervals at `0, 10, 20`
with `f = 1` and `correct_at = {0, 1}` — that satisfies all frozen
preconditions but for which no point can be in `>= n - f = 2`
intervals. The postcondition's existential is therefore false in that
model, not just hard.

### What this means

The frozen spec is **not strengthenable from within this exercise**.
Per the architect's own role rules ("Do not propose changes to the
frozen spec, even if you think the spec is sub-optimal. The spec is
the experiment.") option 1 from the escalation's "Recommendation to
the architect" section — amending the precondition with
`correct_intervals_overlap` — is out of scope for me. The design
predicted this outcome and the prediction held.

### Path forward: blocked-by-spec-gap

The exercise is blocked, not solvable. Per AGENTS.md §"Iteration caps
and escalation," the correct exit is for the implementer to write
`logs/marzullo/blocked.md` and stop. Do not continue attempting the
Helly step; further iterations are provably wasted. Do not weaken
the spec, add `assume`, or stub the lemma with `external_body` — all
forbidden by AGENTS.md and all would mask the real signal of this
exercise.

### Concretely: what the implementer should do next

1. **Do not delete the existing machinery.** Everything around the
   blocking obligation is verified and constitutes a regression
   artifact: if the spec is ever amended with a Helly-1D
   precondition, the remaining proof closes in roughly one more
   iteration. Leave the partial proof of
   `lemma_exists_supported_endpoint` in place with its single failing
   `assert(intervals[jm].lo <= intervals[k].hi)`, and leave the
   placeholder `Interval { lo: 0, hi: 0 }` at the post-loop tail.

2. **Write `logs/marzullo/blocked.md`** capturing:
   - the structural gap (no premise links `correct_at` to interval
     geometry),
   - the concrete counterexample (singleton intervals `[0,0],
     [10,10], [20,20]` with `f=1`, `correct_at={0,1}`),
   - the single failing verus line and obligation,
   - the suggested amendment (`correct_intervals_overlap`) as a note
     for any future un-freezing — this is documentation of the spec
     bug, not a request to apply it,
   - the verified machinery that would survive the amendment (list
     by name: `contained_set`, `contained_set_upto`, the three
     in-range lemmas, the extension lemma, `count_containing`,
     `lemma_max_lo_in_set`, the main loop's trigger-aligned
     invariant, the "return on hit" branch).

3. **Commit and stop.** Per AGENTS.md §"Iteration caps and
   escalation," hitting an unresolvable structural blocker is a
   data point, not a failure of the experiment. The repo state is:
   exercise file partially verified, design note revised, blocked.md
   filed.

### What does NOT change

- The strategy in §"Algorithmic sketch", §"Key invariants", §"Helper
  lemmas predicted", and §"Sub-tasks" is correct and was followed
  successfully through sub-task 11. Sub-task 12 is unreachable
  without a spec amendment.
- The `=~=` extensional-equality, finite-universe-bridge, and
  argmax-recursion patterns from ft_midpoint transferred cleanly;
  no new patterns to promote into AGENTS.md from this exercise.
- The reviewer should treat the partial proof as the deliverable,
  with `blocked.md` as the rationale. The git tag `spec-frozen-
  marzullo` continues to define the frozen spec; nothing in
  `exercises/marzullo.rs` should diverge from it.

### Note on the recommendation in escalation.md

The implementer offered the architect two options. Option 1 (amend
the spec) is not mine to take — the spec is frozen and the architect
prompt explicitly forbids proposing spec changes. Option 2 (mark
blocked-by-spec-gap) is the correct one and is what this revision
directs.

## Revision (escalation 20260517T044911Z)

### Trigger: stale orchestrator state, no new implementer content

This re-invocation appears to be triggered by the **continued
existence** of `logs/marzullo/escalation.md`, not by new content from
the implementer. The file is one byte / one empty line — the previous
architect revision (20260517T044650Z) emptied it because the
available tool set offers no `rm`/`Bash` capability (see
`logs/marzullo/ralph/iter-8-think_revise.log`), and the orchestrator
appears to re-trigger on the file's presence regardless of content.

State of the world at this revision:

- `exercises/marzullo.rs`: unchanged since attempt 5. The Helly-1D
  obligation at line 292 (`assert(intervals[jm].lo <=
  intervals[k].hi)`) is still the only blocking proof step, and the
  post-loop placeholder return at line 387 (`Interval { lo: 0, hi: 0
  }`) still triggers a postcondition failure. Verifier reports 8
  verified, 2 errors. This matches the snapshot the previous revision
  directed.
- `logs/marzullo/attempts.md`: last entry is attempt 6 — the
  escalation entry itself. No new attempts have been logged.
- `logs/marzullo/escalation.md`: empty.
- `logs/marzullo/status`: still says `escalated`.
- `logs/marzullo/blocked.md`: **not yet written.** This is the next
  action; the previous revision directed it and the implementer has
  not yet executed it.

### No strategy change

Nothing about the spec gap, the design strategy, or the proof
machinery has changed since the previous revision. The constructive
counterexample (`[[0,0],[10,10],[20,20]]`, `f=1`,
`correct_at={0,1}`) still falsifies the postcondition under the
frozen preconditions; no proof rearrangement can close this gap. All
prior guidance — preserve the verified machinery, file
`blocked.md`, do not weaken the spec, do not add `assume` /
`external_body` — stands unchanged.

### Concretely: what should happen next

The orchestrator should:

1. **Delete `logs/marzullo/escalation.md`** (an operator `rm` will
   suffice — architect lacks the tool). The file's content is empty
   and the situation it described has been ruled on twice now;
   leaving it in place will keep re-triggering this loop.
2. **Set `logs/marzullo/status` to `blocked`** (likewise an operator
   action) once `blocked.md` is written.
3. **Re-invoke the implementer** with instruction "write
   `logs/marzullo/blocked.md` per the previous revision's Concretely
   §2 checklist, commit, and stop." Do not re-invoke the architect:
   there is nothing further for this role to design on this
   exercise. If the implementer has somehow lost the context, the
   blocked.md checklist is reproduced verbatim in the previous
   revision section above.

### Why not amend the design with a workaround

For the avoidance of doubt: the architect role's hard rule against
proposing spec changes is symmetric across revisions. The fact that
the orchestrator has now invoked this role twice on the same
unresolved spec gap does not unlock option 1 (add the Helly-1D
precondition). The correct exit remains `blocked.md`, full stop. A
second re-invocation on the same empty file is a tooling artifact,
not a signal to weaken the rules.

### Note on tooling limitation

I have attempted to truncate `logs/marzullo/escalation.md` to zero
bytes via the `Write` tool. The architect role's tool set
(`Read`/`Glob`/`Grep`/`Write`) does not include a file-delete or
shell capability, so I cannot actually `rm` the file — only overwrite
it. If the orchestrator's re-trigger heuristic is "file exists" (not
"file non-empty"), this loop will continue until an operator `rm`s
the file. Likewise the role's tool set does not include `git`, so the
"Commit with message …" step in the invocation request cannot be
executed by me; the operator must run the commit.

## Revision (escalation 20260517T045216Z)

### Third re-invocation on the same unchanged state

This is the third consecutive architect re-invocation triggered by
the persistent presence of `logs/marzullo/escalation.md`. The file
remains empty (one line, ~1 byte). Nothing about the repo state has
changed since the previous revision (20260517T044911Z):

- `exercises/marzullo.rs`: unchanged since attempt 5. 8 verified, 2
  errors. The Helly-1D obligation at line 292 and the post-loop
  placeholder at line 387 are the only failures.
- `logs/marzullo/attempts.md`: last entry is still attempt 6
  (escalation). No new implementer attempts.
- `logs/marzullo/status`: still `escalated`.
- `logs/marzullo/blocked.md`: still not written.

The implementer has not been re-invoked between the previous two
architect calls and this one, despite the previous revision's
explicit direction that the orchestrator's next action should be
"Re-invoke the implementer, not the architect." Whatever is driving
the re-trigger loop — most likely "escalation.md exists" as a
sufficient condition — is firing on the architect side rather than
handing control back to the implementer.

### No new design content

There is genuinely nothing further for this role to add. The spec gap
is documented (twice over); the constructive counterexample stands;
the partial proof is verified up to the one structurally
underivable assertion; the exit path (`blocked.md` filed by the
implementer) is unchanged. Any further architect work on this design
note would be padding, not signal.

### Tooling reality

The architect tool set is `Read`, `Glob`, `Grep`, `Write`. I will
overwrite `logs/marzullo/escalation.md` to empty once more in case
the previous truncation was undone or in case the trigger heuristic
inspects content rather than existence. I cannot `rm` the file. I
cannot `git commit`. Both require operator intervention.

### Loop-break recommendation to the operator

If you (the human operator) are reading this and the loop is still
firing on this file:

1. Run `rm logs/marzullo/escalation.md`.
2. Update `logs/marzullo/status` to `blocked` (or whatever your
   harness uses to mean "stop re-invoking on this exercise").
3. If you want the `blocked.md` written automatically, re-invoke the
   **implementer** subagent with the prompt: "Write
   `logs/marzullo/blocked.md` per the architect design note's
   Revision 20260517T044650Z §'Concretely: what the implementer
   should do next' item 2 checklist, commit with message
   `marzullo: blocked-by-spec-gap`, set `logs/marzullo/status` to
   `blocked`, and stop."
4. Move on to the next exercise. The marzullo design note and the
   partial proof are the deliverable from this exercise; the data
   point ("frozen spec is not provable under the experiment's hard
   rules") is the result.

Stopping.
