# Shared rules for verus-calibration

This file is the shared rule book for every agent working in this repo. Each subagent (`.claude/agents/architect.md`, `implementer.md`, `reviewer.md`) reads this plus its own role-specific prompt.

The experiment's success criterion is binary: `verus exercises/<name>.rs --crate-type=lib` exits 0, **and** the reviewer approves the diff against the frozen spec.

## Hard rules — apply to every role

1. **Never modify the specification.** The `requires`, `ensures`, and any pre-existing `spec fn` definitions in each exercise are frozen. The frozen state is captured by the git tag `spec-frozen-<exercise>` (e.g. `spec-frozen-binary_search`). Diff against that tag before claiming success.
2. **Never use `#[verifier::external_body]`.** This bypasses the verifier; it is not a verification result.
3. **Never add `assume(...)` to discharge an obligation.** `assert` is fine; `assume` defeats the purpose.
4. **Never weaken a spec to make verification pass.** If you find yourself wanting to, stop and write a `// SPEC WEAKENED:` comment explaining why, then report blocked.
5. **Never replace `unimplemented!()` with `unreachable!()` or panicking stubs to dodge cases.** Every postcondition must be discharged for real.
6. **Stop after the iteration cap.** Binary search: 10 attempts. Bounded log: 20. Quorum count: 20. On cap, write `logs/<exercise>/blocked.md` describing what was tried and what failed.

## What "verified" means here

- `verus <file> --crate-type=lib` exits 0
- No `external_body`, `assume`, or commented-out spec
- The reviewer's audit (see `.claude/agents/reviewer.md`) returns APPROVE
- Code compiles under normal `cargo check`

## Per-attempt logging (implementer only)

Append to `logs/<exercise>/attempts.md` after every verification attempt:

```
## Attempt N — <ISO timestamp>
**Approach:** one sentence on what changed since the last attempt.
**Verifier output:** which obligation failed, paste the relevant 5-10 lines.
**Next idea:** what you'll try next, or "blocked" + why.
```

Keep raw verifier output in `logs/<exercise>/raw/attempt-N.txt`. Commit per attempt with message `<exercise> attempt-N: <one-line description>`.

## Iteration caps and escalation

- 3 consecutive attempts failing on the same proof obligation → implementer writes `logs/<exercise>/escalation.md` and stops. Orchestrator re-invokes the architect.
- Hitting the per-exercise iteration cap → implementer writes `logs/<exercise>/blocked.md` with the full context. Stop. Move to next exercise.

## Uninterpreted spec functions and trust boundaries

Some exercises declare `pub uninterp spec fn` predicates with no body
(e.g. `signature_valid` in `quorum_cert.rs`). These are deliberately
opaque trust-boundary abstractions. The implementer must not provide a
body for them and must not add an `assume_specification` against them.
Reasoning about them happens through the helper `spec fn`s and lemmas
that the verified code calls. A real deployment connects them to a
vetted external library via an `assume_specification` outside this
repo; here that connection is out of scope.

## Pre-spec verification (operator)

Before tagging `spec-frozen-<name>` for a new exercise, write a *witness* file at `exercises/<name>_witness.rs`. The witness has the same `requires`/`ensures` clauses (and the same `open spec fn` / `uninterp spec fn` definitions) as the exercise file, plus a real reference implementation. Run `ralph/check-spec.sh <name>`. If it passes (verus verifies the witness with no cheat tokens), the spec provably admits a model and the operator may tag the freeze. If it fails, the spec is unprovable or under-constrained — fix the spec, not the witness.

This catches two classes of bug the agent loop cannot:

1. **Logically unprovable specs.** The original marzullo freeze omitted the Helly-1D precondition `correct_intervals_overlap`. No algorithm can verify against the postcondition without it. The agent burned attempts 5–7 surfacing this via constructive counterexample before the operator re-froze. A witness would have failed at step 0.
2. **Spec syntax that no longer compiles.** The original bounded_log freeze used pre-`final(self)` syntax that newer Verus rejects. A witness would have failed `verus` immediately.

The witness is operator territory. The agent's tool whitelist explicitly denies `Read`, `Glob`, `Grep`, and corresponding `Bash` reads on any path matching `*_witness*` (single-file) or `*_witness/*` (multi-file). The agent cannot see or modify these files. The pre-commit hook still applies cheat-token detection to them.

*History note (2026-05-18):* the `vec_swap` exercise was originally run with a permissive whitelist that allowed witness reads; the agent's own attempt-1 commit message and playbook entry recorded "Ported the witness's proof structure," invalidating it as an invention test. The whitelist was tightened in response, and `vec_swap_v2` (same spec, fresh exercise name) is the clean rerun.

Empirical demonstration: `scripts/test-witness-catches-bad-spec.sh` strips the Helly-1D precondition from a copy of the marzullo witness and confirms verus rejects it.

## On SMT timeouts

Do not just raise the rlimit. First:
1. Break the proof into smaller asserts to localize where the solver gets stuck.
2. Replace `Vec` operations with `Seq` reasoning where possible.
3. Add a helper lemma the main proof can call.

If after three attempts the timeout persists, log it as a blocker — that's a data point, not a failure of the experiment.

## Exercise order

Work in this order. Do not start the next exercise until the previous one is either verified-and-approved or blocked-and-logged:

1. `exercises/binary_search.rs`
2. `exercises/bounded_log.rs`
3. `exercises/quorum_count.rs`
4. `exercises/quorum_cert.rs` — first step on the BFT-for-aerospace path. Two obligations: an exec structural check, and a proof-only safety lemma about honest voters.
5. `exercises/ft_midpoint.rs` — first sensor-fusion exercise. Verified Schmid-Schossmaier fault-tolerant midpoint over `n >= 2f+1` readings with at most `f` Byzantine. One obligation: an exec function whose result is bracketed by correct sensor readings on each side. Single-round, single-value variant.
6. `exercises/marzullo.rs` — interval variant of the sensor-fusion track. Verified Marzullo's algorithm: given `n >= 2f+1` sensor intervals with at most `f` Byzantine, return an interval whose interior contains a point supported by at least `n - f` input intervals. Same trust boundary as ft_midpoint; reuses the inclusion-exclusion pigeonhole pattern from there.
7. `exercises/cross_module_counter.rs` — first multi-module exercise. A `counter` module exports a bounded counter with `closed spec fn` value/bound/invariant; a `client` module uses it via the public API only. Stresses module-level visibility, `closed spec fn` opacity, and cross-module loop invariants. Single file, nested `mod` blocks inside one `verus!{}`.
8. `exercises/counter_multifile/` — first multi-file exercise. Same algorithm as `cross_module_counter` but split into `main.rs` (entry) and `counter.rs` (module). The variable being tested is the tooling regime: directory layout, multi-file `verus` invocation, hook spec preservation across files, witness directory structure. If the agent succeeds on first attempt, the playbook generalises to multi-file with no code change; if it gets stuck, the breakage is most likely on the harness side, not algorithmic.
9. `exercises/counter_producer/` — second multi-file exercise; first one that stresses genuine cross-module composition. Three modules: `counter` (same as `counter_multifile`), `producer` (bulk-increments a counter by `n` via a loop that composes `incr`'s single-step postcondition), `main` (entry, pipeline). The producer's loop invariant carries facts the counter doesn't expose. If the playbook generalises cleanly, this verifies in one attempt; if not, the sticking points are interesting data for multi-module pattern work.
10. `exercises/sensor_poll/` — first composition-of-existing-primitives exercise. Three modules: `fusion` (port of `marzullo`), `auth` (distinct-sensor structural check, smaller version of `quorum_cert`'s `verify_qc_structure`), `main` (poll function with a composition theorem). The new work is `poll`'s body: project from `SensorReport` to `Interval`, call `marzullo`, instantiate a projection lemma to bridge `intervals_containing` (frame of `marzullo`'s postcondition) to `reports_containing` (frame of `poll`'s ensures). Tests whether the methodology handles system-level integration where the correctness statement spans the seam between two primitives.
11. `exercises/sensor_poll_signed/` — second composition exercise. Same three-module layout as `sensor_poll`, with `auth` extended by the cryptographic trust boundary lifted from `quorum_cert`: `Hash` / `PubKey` / `Signature` aliases, `pk_of` / `signature_valid` / `report_msg` uninterp predicates, `all_signatures_valid` and `valid_report_bundle` open spec fns, and a `sig: Signature` field on `SensorReport`. `poll`'s precondition gains `all_signatures_valid(reports@)` and its `Some`-branch ensures gains `valid_report_bundle(reports@)`. Exec layer is unchanged — `check_distinct` still only checks the structural half, and the signature predicates stay opaque trust-boundary abstractions. The new work is the one-line conjunction `assert(valid_report_bundle(reports@))` that combines `check_distinct`'s ensures with the new precondition; the rest of `poll`'s body is byte-equivalent to `sensor_poll`'s.
12. `exercises/sensor_poll_honest/` — third composition exercise, set up as a discovery test for the methodology. Same three-module layout as `sensor_poll_signed` (with `fusion.rs` and `auth.rs` byte-identical); `main.rs` adds an honest-voter ensures clause to `poll`'s Some branch: `exists p, k. interval.lo <= p <= interval.hi && 0 <= k < reports.len() && correct_at(k) && point_in_interval(p, reports[k].interval)`. The design note states only the proof obligation and the BFT intuition; it deliberately does NOT name the load-bearing lemmas or set constructions, so the agent must discover the proof structure rather than execute a designed proof. The 1-attempt caveat that applies to all four post-marzullo 1-try wins (architect pre-named the load-bearing invariant) is what this exercise is built to address; iteration count and any escalation cycles are the measurement.
13. `exercises/counter_filler/` — third multi-file composition exercise, set up as the second deliberate discovery test for the methodology. Same proof family as `counter_producer` (cross-module composition over a closed-spec counter API) but a structurally different loop shape: `fill_to(c, target)` advances `c.value()` until it equals `target`, with no separate counter variable. The design note states the obligation only and explicitly warns the implementer not to copy `counter_producer`'s loop invariant verbatim. The test is whether the agent can adapt the snapshot+bound-preservation family from the playbook to a different loop shape without the conjuncts being pre-named. Second data point on cross-exercise pattern transfer, on a different proof family from `sensor_poll_honest`'s inclusion-exclusion.
14. `exercises/vec_swap.rs` — first attempted invention test. INVALIDATED on 2026-05-18: the agent's attempt-1 commit and self-authored playbook entry recorded "Ported the witness's proof structure," because the tool whitelist did not yet deny reads on `*_witness*` paths. Kept on disk as evidence; do not treat the 1-attempt result as data on invention.
15. `exercises/vec_swap_v2.rs` — second attempted invention test. ALSO INVALIDATED on 2026-05-18: the operator created the v2 scaffold by `cp exercises/vec_swap.rs exercises/vec_swap_v2.rs` *after* the agent had filled in vec_swap.rs's body in the first run, so the v2 "scaffold" already contained the full proof. The agent's iter-1 log flagged this honestly ("I made no edits to exercises/vec_swap_v2.rs"). Kept on disk as evidence.
16. `exercises/swap_multiset.rs` — third attempt at the invention test. Spec identical to `vec_swap`; scaffold typed by hand (no copy from the modified vec_swap.rs); witness denied to the agent under the hardened whitelist. Iteration cap 25.

## External-validity tests (VeruSAGE-Bench)

Separate track from the numbered internal-exercise sequence above. Tasks here are drawn from Microsoft's [VeruSAGE-Bench](https://github.com/microsoft/verus-proof-synthesis) (849-task benchmark, sampled-100 subset). The point is *external validity* of the methodology: do tasks we did not design behave the same way through this harness? The result that matters is attempts-to-verify on each task, set alongside whatever published AutoVerus / VeruSAGE numbers exist (per upstream README, per-task leaderboard is "TO COME"; aggregate numbers are in the papers).

Scaffold rules for any external task:

- `exercises/<TASK_NAME>.rs` — upstream task file byte-for-byte. Becomes the frozen spec on `git tag spec-frozen-<TASK_NAME>`.
- `exercises/<TASK_NAME>_witness.rs` — operator-authored minimal verified version (task file + smallest body fix). Subject to the same witness-deny ACL and pre-commit cheat-token check as every other witness.
- `exercises/<TASK_NAME>.design.md` — operator-authored short note: signature, what the body needs to do, suggested order-of-operations. Same shape as internal exercises.
- Iteration cap chosen per task; set in `ralph/run-exercise.sh`'s `case` block.

Upstream tasks contain `fn main() {}` (binary form) and run under `verus <task>.rs` per upstream docs. Our harness keeps invoking `verus <file> --crate-type=lib` — confirmed locally that both flag forms verify upstream ground-truth witnesses, so the harness is unchanged.

Honest scope: AGENTS.md's "Discovered patterns" section accumulates per-exercise findings; the same applies here. External-task entries should be tagged with `[VeruSAGE-Bench]` so they stay distinct from the internal track when reading the playbook.

17. `exercises/MA__bin_sizes__mul_assoc.rs` — first external-validity task. Pure `proof fn` synthesis: `(x*y)*z == y*(x*z)` over `nat`. Upstream prefix `MA` = memory-allocator (Verus-verified mimalloc port). Smallest task in the benchmark by byte count (136 B). Iteration cap 15. Ground-truth witness closes via `by (nonlinear_arith)`.
18. `exercises/VE__utils__init_vec_u8.rs` — second external-validity task. Exec function with a `while` loop that needs an `invariant` + `decreases` clause to verify. Upstream prefix `VE` = vest (verified serializer). Closest match to our existing methodology shape (exec + loop invariant, same family as `binary_search`). Iteration cap 15.

### Batch 2 — methodology probe (neutral design notes)

Items 17–18 above shipped with design notes that named the relevant Verus tooling family (`by (nonlinear_arith)` and "loop needs invariant + decreases"). That made them harness probes, not methodology probes. Batch 2 corrects the test design: each task's `.design.md` states the obligation and a standard sub-task ordering only — no Verus tooling-family names, no lemma names, no proof-structure hints, no cross-references to internal exercises by name. AGENTS.md's "Discovered patterns" section is left in place because it is part of the methodology under test.

19. `exercises/IR__seq_is_unique__singleton_seq_to_set_is_singleton_set.rs` — Pure proof fn: `seq![x].to_set() == set![x]`. Upstream prefix `IR` = ironkv. 171 B. Iteration cap 15.
20. `exercises/NR__extra__lemma_set_of_first_n_nat_is_finite.rs` — Pure proof fn: `Set::new(|i: nat| i < n).finite()`. Upstream prefix `NR` = nrkernel. 180 B. Iteration cap 15.
21. `exercises/AL__a_submap_of_a_finite_map_is_finite.rs` — Pure proof fn: submap-of-finite-map is finite. Upstream prefix `AL` = anvil-library. 242 B. Iteration cap 15.
22. `exercises/MA__bin_sizes__shift_is_div.rs` — Pure proof fn: `x >> shift == x as nat / pow2(shift as int)`. 308 B. Different family from `mul_assoc` (bit-shift / division recursion, not nonlinear chain). Iteration cap 20.
23. `exercises/IR__verus_extra__lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity.rs` — Pure proof fn: if every element satisfies a predicate, `Seq::filter(pred) == self`. 309 B. Iteration cap 15.
24. `exercises/NR__definitions_u__lemma_maxphyaddr_facts.rs` — Pure proof fn: `0xFFFFFFFF <= MAX_PHYADDR <= 0xFFFFFFFFFFFFF`, where MAX_PHYADDR is computed from an axiomatized bit-width range. 879 B. Mix of compute, bit_vector, and axiom calls. Iteration cap 20.
25. `exercises/OS__array__impl4__init2none.rs` — Exec function: `init2none` zeroes out an `Array<Option<T>, N>` via a `for i in 0..N` loop that needs an invariant. Upstream prefix `OS` = atmosphere (OS kernel). Second exec data point on the external-validity track. 1132 B. Iteration cap 15.
26. `exercises/NO__spec__unbounded_log__get_fresh_nat_not_in.rs` — Pure proof fn: existence of a fresh request id outside a finite set of in-flight requests. Upstream prefix `NO` = node-replication. 3768 B. Stress test on size; uses multiple operator-axiom helper lemmas already declared in the task file. Iteration cap 25.

## Multi-agent workflow (brief)

The full state machine and how the human operator drives it is in `ORCHESTRATION.md`. In short:

- **Architect** (Opus, `.claude/agents/architect.md`) — designs strategy, writes `exercises/<name>.design.md`. Does not see verifier output on first pass. Re-invoked on escalation.
- **Implementer** (Sonnet, `.claude/agents/implementer.md`) — fills in bodies and proofs, runs verus, iterates.
- **Reviewer** (Opus, `.claude/agents/reviewer.md`) — audits the diff against `spec-frozen-<exercise>` after verus passes. Returns APPROVE/REJECT.

## Discovered patterns

(implementer: append findings here as you go — Verus quirks, SMT-friendly patterns, things to avoid)

### binary_search (attempt 1, success)
- **`decreases` clause required**: Verus requires every `while` loop to have a `decreases` clause or an explicit `#[verifier::exec_allows_no_decreases_clause]` attribute. Use `decreases hi - lo` for a half-open binary search window.
- **Sortedness instantiation via `assert forall ... by { assert(is_sorted(...)); }`**: Wrapping the body in an `assert forall ... implies ... by { ... }` block inside an outer `assert(forall ...) by { ... }` reliably triggers sortedness instantiation. The SMT solver can then chain `v@[k] <= v@[mid]` with `v@[mid] < target` (or `> target`) to discharge the exclusion foralls.
- **Half-open window `[lo, hi)` avoids `usize` underflow**: Use `mid = lo + (hi - lo) / 2` to avoid overflow, and `hi = mid` (not `mid - 1`) to avoid underflow on the upper-cursor update.
- **Invariant structure**: 5 conjuncts: `is_sorted(v@)`, `0 <= lo <= hi <= v@.len()`, `hi <= v.len()`, left-exclusion forall, right-exclusion forall. The two foralls tile the full index range on loop exit, directly yielding the `None` postcondition.

### bounded_log (attempt 1, success)
- **Verus 0.2026.05.13 `&mut self` postcondition migration**: New Verus versions require explicit disambiguation of `self` in postconditions of `&mut self` functions. Use `final(self).X()` instead of `self.X()` in `ensures` clauses; `old(self).X()` in `requires` clauses. This is purely syntactic (same semantics) but required for compilation.
- **Frame property closes automatically with a defensive assert**: After `Vec::push`, adding `assert(self.msgs@ == old(self).msgs@.push(msg));` followed by `assert(forall|i: int| 0 <= i < old(self).msgs@.len() ==> self.msgs@[i] == old(self).msgs@[i]);` reliably closes the frame forall. The built-in `Seq::push` axioms fire with this nudge.
- **Loop-free Vec-backed log**: All four functions (`new`, `len`, `get`, `append`) verify with no loops. `Vec::new()`, `Vec::len()`, indexing, and `Vec::push()` all have sufficient built-in specs in vstd.

### ft_midpoint (attempts 1–7, success)
- **`Set::new(|i| ...).len()` requires a finite-universe bridge before any cardinality reasoning fires.** Pattern: build `set_int_range(0, n)` as the universe, call `vstd::set_lib::lemma_int_range(0, n)` to give it finiteness + length, prove the constructed set is `subset_of(set_int_range(0, n))` with a one-line membership-equivalence assert, then `vstd::set_lib::lemma_len_subset(subset, universe)`. The three pieces — finiteness, subset, length bound — all flow from this chain.
- **Inclusion-exclusion is `vstd::set_lib::lemma_set_intersect_union_lens(a, b)`.** Given both `a` and `b` finite, this gives `(a+b).len() + a.intersect(b).len() == a.len() + b.len()` for free. Combined with a universe-size upper bound on `(a+b).len()`, this is the load-bearing arithmetic for pigeonhole proofs.
- **Argmax/argmin over a finite `Set<int>` of indices** is a recursion on `decreases s.len()`. Extract a witness `j0 = choose|x| s.contains(x)` after `axiom_is_empty_len0(s)` + `axiom_is_empty(s)` flip `s.len() >= 1` into `!s.is_empty()` into the choose-witness. Recurse on `s.remove(j0)`; case-split on the chosen element vs. the IH result. Each branch needs an `assert forall|j: int| s.contains(j) implies readings[j] <= readings[jm] by { if j != j0 { assert(s2.contains(j)); } }` to bridge the recursive forall back to `s`.
- **Existential-by-contradiction.** When the spec ensures `exists|j: int| P(j)`, the proof pattern `if !(exists|j| P(j)) { ... assert(false); }` lets you assume the negation as a `forall|j| !P(j)` inside the if-branch. Build the "this fails" set, derive `lo.len() + hi.len() >= n` via inclusion-exclusion, case-split on which side gets `>= f+1`, and contradict via argmax/argmin.
- **Bridging exec count to set cardinality.** Define a prefix-set spec helper `le_set_upto(readings, v, m)` that's just `le_set` restricted to `[0, m)`. Loop invariant: `c as nat == le_set_upto(_, v, i as int).len()`. A one-line extension lemma proves `le_set_upto(_, v, i+1) =~= le_set_upto(_, v, i).insert(i)` when `readings[i] <= v` and `=~= le_set_upto(_, v, i)` otherwise. After the loop, the extensional collapse `le_set_upto(_, v, n) =~= le_set(_, v)` closes the count-equals-cardinality post.
- **Trigger matching across `lemma_exists_midpoint` and the loop invariant.** When the witness from `choose|jx: int| ... le_set(readings@, readings@[jx]).len() >= f + 1 ...` needs to instantiate a `forall|j2: int| ... le_set(readings@, readings@[j2]).len() < f + 1 ...` invariant, mark the invariant's body with `#[trigger] readings@[j2]` to align with Verus's chosen `readings@[jx]` trigger on the existential. Without the explicit trigger, instantiation across the lemma/invariant boundary doesn't fire.

### marzullo (attempt 1, success)
- **Constructive existence beats contradiction.** Where `ft_midpoint` had to do `Lo ∪ Hi` set-decomposition + inclusion-exclusion in `lemma_exists_midpoint`, Marzullo's existence is `argmax-lo + Helly-1D`: pick `jm` argmax over correct indices, let `p = intervals[jm].lo`, then for every correct `k` directly conclude `intervals[k].lo <= p` (argmax) and `p <= intervals[k].hi` (Helly-1D at `(jm, k)`). The constructive path is *strictly shorter* and avoids the second pigeonhole lemma family. When the precondition gives you "all of these pairwise interact in a useful way," prefer constructive existence to contradiction.
- **Field-by-field reads avoid silent `Copy` traps.** `Interval` happens to be `Copy`-eligible (two `i64`s), but writing `let iv = intervals[i]; iv.lo ... iv.hi ...` invites surprise if a struct grows a non-`Copy` field later. Reading `intervals[i].lo` and `intervals[i].hi` as separate exec lets is explicit and equally efficient. Cheap defensive pattern.
- **Full port reuses 80%+ of the parent exercise.** All three L1 subset/finiteness lemmas, the L2 prefix-extend lemma (empty body), the L3 argmax lemma, and the `count_*` exec function were near-verbatim ports from `ft_midpoint.rs` with `intervals[_].lo` substituted for `readings[_]` and `intervals_containing` substituted for `le_set`. The only genuinely new code was the L4 existence proof (which is *simpler* than its parent). When the design note explicitly identifies the parent exercise as the proof skeleton, do the full port in one attempt — splitting it into smaller iterations is wasted ceremony.

### cross_module_counter (attempt 1, success)
- **`closed spec fn` + postcondition bridge is sufficient for cross-module reasoning.** The `client` module never sees `Counter`'s internal `value: u32` / `bound: u32` fields. All facts it needs come from the postconditions on `new`, `incr`, `get`. The four-conjunct loop invariant (`c.invariant()`, `c.value() == i`, `c.bound() == target`, `i <= target`) is stated entirely in the closed spec-fn vocabulary, and Verus re-establishes each conjunct after `c.incr()` from `incr`'s ensures alone — no defensive frame asserts needed.
- **Nested `mod` blocks inside a single `verus! { }` work cleanly.** Both modules can `use vstd::prelude::*;` independently. `client` imports the counter via `use super::counter::Counter;`. No special tooling needed; single-file `verus` invocation handles the module structure.
- **`final(self)` syntax** (from the bounded_log discovery) carries over verbatim to `&mut self` postconditions in this exercise.

### counter_multifile (attempt 1, success)
- **Multi-file tooling generalises trivially.** Splitting `cross_module_counter.rs` into `main.rs` + sibling `counter.rs` with a `mod counter;` declaration at the top of `main.rs` (and `use vstd::prelude::*;` inside each file's own `verus! { }` block) "just works" with `verus exercises/counter_multifile/main.rs --crate-type=lib`. No build system, no manifest, no extra flags — verus walks the `mod` declarations the same way rustc does. The byte-identical exec bodies from `cross_module_counter` verify with 5 verified, 0 errors.
- **`closed spec fn` visibility is preserved across files.** The client in `main.rs` only sees `counter::Counter`'s public closed spec fns (`value`, `bound`, `invariant`) and the function postconditions — the private `value: u32` / `bound: u32` fields stay opaque exactly as they did in the single-file nested-`mod` version. The four-conjunct loop invariant (`c.invariant() && c.value() == i && c.bound() == target && i <= target`) carries over verbatim.

### counter_producer (attempt 1, success)
- **Cross-module loop invariants compose via "snapshot + bound preservation".** Producer's `produce(c, n)` uses six-conjunct invariant: `c.invariant()`, `c.value() == start + i`, `c.bound() == old(c).bound()`, `i <= n`, `start == old(c).value()`, `start + n <= c.bound()`. The last conjunct is the load-bearing one — it threads the function's precondition (`old(c).value() + n <= old(c).bound()`) through each iteration so that `incr`'s precondition `value() < bound()` is derivable as `start + i < start + n <= c.bound()`.
- **`let start = c.get()` snapshot pattern.** Capturing the initial value via the exec `get()` (rather than ghost) gives a real u32 the invariant can name. The fact `start == old(c).value()` is provable inside the invariant from `get`'s postcondition `v == self.value()`, and persists across loop iterations because `start` is immutable. This anchors the composed claim `c.value() == start + i` so that at loop exit (`i == n`) the final ensures `final(c).value() == old(c).value() + n` collapses to `start + n == old(c).value() + n`.
- **Genuine cross-module composition with closed spec fns "just works".** `producer.rs` reasons about `Counter` entirely through `c.invariant()`, `c.value()`, `c.bound()` — closed spec fns whose bodies live in `counter.rs`. No defensive asserts needed across the module boundary; each conjunct of the loop invariant is re-established after `c.incr()` directly from `incr`'s ensures. The playbook from `cross_module_counter` and `counter_multifile` generalises with zero modification.
- **`use crate::counter::Counter;` for sibling modules declared in `main.rs`.** When `main.rs` declares both `mod counter;` and `mod producer;`, the producer module references the counter via `crate::counter::Counter`, not `super::counter::Counter` — both modules are at crate root, not nested under each other.

### sensor_poll (attempt 1, success)
- **System-level composition by direct port + projection lemma.** Three modules: `fusion` (verbatim marzullo port — body + 5 spec helpers + 4 proof lemmas), `auth` (bitmap-backed `check_distinct` lifted from `verify_qc_structure` in quorum_cert; four-conjunct invariant — cursor bound, seen-len, distinct-prefix, bitmap-vs-prefix abstraction), `main` (`poll`: `check_distinct` → fail returns `None`; else project intervals into fresh `Vec<Interval>` with view `=~= project_intervals(reports@)`; call `marzullo`; `choose` witness `p`; one-line empty-body extensional-equality lemma `reports_containing(reports@, p) =~= intervals_containing(project_intervals(reports@), p)`). 16 verified, 0 errors on first attempt.
- **The projection lemma is one `proof fn` with empty body and `=~=` in the ensures.** Verus closes it because both sets are built from `Set::new(|i| 0 <= i < n && point_in_interval(p, X[i]))` where `X[i]` is `reports[i].interval` on the LHS and `project_intervals(reports)[i]` on the RHS — and `Seq::new(reports.len(), |i| reports[i].interval)[i]` reduces to `reports[i].interval`. No `assert forall` needed inside the lemma.
- **Build-the-projection-loop pattern.** When you need a `Vec<B>` whose `@` view equals `Seq::new(a.len(), |i| f(a[i]))`, the loop is: `let mut out = Vec::with_capacity(a.len()); for i in 0..a.len() { out.push(f(a[i])); }` with invariant `out@.len() == i as nat` and `forall|k| 0 <= k < i ==> out@[k] == f(a@[k])`. After the loop, `assert(out@ =~= Seq::new(a@.len(), |k| f(a@[k])));` closes the extensional equality.
- **`#[derive(Copy, Clone)]` on `Interval` was crucial** to let `reports[i].interval` be read into a fresh `Vec<Interval>` without moves; without it the projection loop would need `clone()` or per-field reads. The architect put the derive in the frozen spec; the implementer reaped the benefit.

### sensor_poll_signed (attempt 1, success)
- **Cryptographic trust boundary threads through pre → post without exec touch.** When the precondition states `all_signatures_valid(reports@)` and no exec function mutates `reports`, the postcondition's `valid_report_bundle(reports@)` collapses to a one-line `assert(valid_report_bundle(reports@))` placed at the point where `distinct_sensors(reports@)` is also in scope (i.e. after `check_distinct` returns `true`). `valid_report_bundle` is just the conjunction of the two, and Verus discharges the conjunction immediately from its definition. Zero exec changes vs. the unsigned `sensor_poll` variant.
- **The new `sig: Signature` field on `SensorReport` is invisible to `check_distinct`.** The bitmap-backed body only reads `sensor_id`; the structural invariant is identical to `sensor_poll`'s, byte-for-byte. The uninterp `pk_of` / `signature_valid` / `report_msg` predicates and the open `all_signatures_valid` spec fn never appear in `check_distinct`'s body or proof.
- **The projection lemma `reports_containing =~= intervals_containing(project_intervals(...))` is byte-identical to `sensor_poll`'s** — adding the `sig` field doesn't touch the membership predicate (which reads `reports[i].interval`), so the empty-body extensional-equality lemma still closes from `=~=` alone.

### sensor_poll_honest (attempt 1, success — audit-confirmed 2026-05-18)
- **The honest-voter clause is one pigeonhole lemma on top of the signed exercise.** The body of `poll` is byte-equivalent to `sensor_poll_signed`'s `poll` (check_distinct → bundle-assert → projection loop → `marzullo` → projection-lemma bridge giving `reports_containing(reports@, p_witness).len() >= n - f`). The new work is exactly one helper lemma `lemma_honest_supporter_exists(reports, p, f)` plus a one-line `choose` for the second witness inside `poll`'s proof block. No new exec code, no changes to `fusion` or `auth`.
- **The pigeonhole recipe is identical to ft_midpoint's `Lo ∪ Hi` decomposition, but inverted.** Where ft_midpoint *assumed* the existence failed and derived `|Lo ∪ Hi| >= n` for a contradiction, here we *constructively* derive `|s ∩ c| >= 1` from `|s|, |c| >= n - f` and `n >= 2f + 1`. The same four-step chain (range subset, finiteness via `lemma_len_subset`, inclusion–exclusion via `lemma_set_intersect_union_lens`, witness via `axiom_is_empty_len0`/`axiom_is_empty` + `choose`) closes the lemma in a single proof block.
- **`s + c =~= s.union(c)` is the bridge between `lemma_set_intersect_union_lens`'s output and a `.union(c)` upper bound from `subset_of(set_int_range(0, n))`.** The lemma states its conclusion with `(a + b).len()`; subset-based finiteness reasoning uses `s.union(c)`. A one-line extensional `=~=` collapses the two, then arithmetic closes `|s ∩ c| >= |s| + |c| - n`.
- **Discovery without a named load-bearing lemma.** The design note deliberately omitted the proof structure. The agent identified (a) that the new clause is a second existential, (b) that both required cardinalities collapse onto the same `[0, n)` universe, (c) that pigeonhole/inclusion–exclusion is the off-the-shelf tool, and (d) that the ft_midpoint discovery notes already encode the lemma chain. Time-to-first-success: one attempt, originally and again on the 2026-05-18 audit re-run (with the hardened whitelist denying witness reads and this same entry temporarily stripped from AGENTS.md).

### counter_filler (attempt 1, success — audit-confirmed 2026-05-18)
- **Target-bounded loop shape: invariant uses `c.value() <= target` + `target <= c.bound()` instead of producer's `c.value() == start + i` + `start + n <= c.bound()`.** The two conjuncts that carry the proof are (a) the upper-bound progress fact `c.value() <= target` (so loop exit + negated guard collapses to `c.value() == target`), and (b) the bound-preservation fact `target <= c.bound()` (so `c.value() < target` from the loop guard plus this transitively gives `c.value() < c.bound()`, discharging `incr`'s precondition). No `start` snapshot, no `i: u32` counter, no `start + n <= c.bound()` — those were producer-specific.
- **`decreases target - c.value()` works when `c.value()` is a closed spec fn returning `u32`.** Each `incr()` raises `c.value()` by 1, so the measure decreases by 1. The invariant's `c.value() <= target` keeps the measure non-negative. No need for a ghost expression — the closed spec-fn read is acceptable in `decreases`.
- **`c.get() < target` as the loop guard is fine.** `get` takes `&self`, has no side effects, and its precondition `self.invariant()` is in the loop invariant. Verus re-checks the precondition on each iteration from the invariant. No need for a `let mut v: u32 = c.get();` shadow variable — the direct exec call in the guard verifies cleanly and is closer to the design's "no separate loop counter" intent.
- **Cross-family transfer worked on first attempt.** The design note deliberately omitted the invariant shape and warned against copying `counter_producer`'s. The agent identified that the snapshot family had to be re-derived for the target-bounded shape: drop `start`, drop `i`, replace `c.value() == start + i` with `c.value() <= target`, replace `start + n <= c.bound()` with `target <= c.bound()`. Two data points (original run + 2026-05-18 audit re-run under hardened whitelist with this entry stripped); both 1-attempt.

### MA__bin_sizes__mul_assoc [VeruSAGE-Bench] (attempt 1, success)
- **Mixed-associativity-and-commutativity nat goals close with one `by (nonlinear_arith)` assert.** Empty body fails (`postcondition not satisfied`), as expected: Verus's linear fragment handles each individual `*` axiom but does not chain associativity + commutativity across `(x*y)*z == y*(x*z)`. A single line — `assert((x * y) * z == y * (x * z)) by (nonlinear_arith);` — invokes the nonlinear solver fragment and closes the goal. No vstd lemma chain needed for this size of goal.
- **Smallest external task (136 B upstream) is also a sanity check on the nonlinear_arith pathway.** Confirms `by (nonlinear_arith)` is wired through this harness's verus build for proof-mode use; useful baseline for future arithmetic-heavy proofs.

### IR__seq_is_unique__singleton_seq_to_set_is_singleton_set [VeruSAGE-Bench] (attempt 2, success)
- **`=~=` alone is not enough for `seq![x].to_set() == set![x]`.** Attempt 1's one-line `assert(seq![x].to_set() =~= set![x])` failed: the SMT solver doesn't bridge the existential `seq![x].contains(y) <==> exists i. 0 <= i < 1 && seq![x][i] == y` to `set![x].contains(y) <==> y == x` on its own. The seq-contains-after-push family (`lemma_seq_contains_after_push`, `lemma_push_to_set_commute`) lives in `group_seq_properties` / `group_seq_extra`, NOT in vstd's default broadcast group `group_vstd_default` (only `group_seq_axioms` and `group_seq_lib_default` are default), so these lemmas don't auto-fire.
- **Bridge pattern: explicit `lemma_push_to_set_commute` call + two `=~=` collapses.** `seq![x]` desugars to `Seq::empty().push(x)`; `set![x]` desugars to `Set::empty().insert(x)`. The proof is three lines: (1) `Seq::<T>::empty().lemma_push_to_set_commute(x);` to get `seq![x].to_set() =~= Seq::<T>::empty().to_set().insert(x)`; (2) `assert(Seq::<T>::empty().to_set() =~= Set::<T>::empty());` — both sides have no members, closed by `axiom_set_new` (default-broadcast) plus the fact that `Seq::empty().contains(a)` reduces to `exists i. 0 <= i < 0 && ...` = false; (3) `assert(seq![x].to_set() =~= set![x]);` chains the above. This avoids `broadcast use` (which would pull in a lot of unrelated lemmas) by surgically invoking the one lemma needed.
- **`uninterp spec fn` empty-set facts still derive from default axioms.** Even though `Seq::empty` is an `uninterp spec fn`, the broadcast axiom `axiom_seq_empty` (in default group) gives `Seq::<A>::empty().len() == 0`, and the body of `Seq::contains` (`open spec fn`, `exists|i| 0 <= i < self.len() && self[i] == needle`) inlines, so the SMT trivially refutes the existential. No additional lemma calls needed for the empty side.

### NR__extra__lemma_set_of_first_n_nat_is_finite [VeruSAGE-Bench] (attempt 1, success)
- **Finiteness of `Set::new(|i: nat| i < n)` closes by structural induction on `n`.** Three-line body: `decreases n`; base case `n == 0` asserts `=~= Set::<nat>::empty()`; inductive case lets `m = (n - 1) as nat`, recurses, then asserts `Set::new(|i: nat| i < n) =~= Set::new(|i: nat| i < m).insert(m)`. Verus's `axiom_set_insert_finite` (default broadcast) then derives finiteness on the inductive step, and `Set::empty().finite()` is a default axiom for the base. No vstd lemma calls needed beyond the extensional `=~=` collapse.
- **`(n - 1) as nat` cast pattern.** In the `else` branch of `if n == 0`, Verus knows `n >= 1`, so subtracting 1 from a `nat` and re-casting via `as nat` is well-defined and Verus admits it without an explicit precondition assert. Cleaner than threading the witness through an extra `let`.

### AL__a_submap_of_a_finite_map_is_finite [VeruSAGE-Bench] (attempt 1, success)
- **Submap-of-finite-map finiteness is the same `lemma_len_subset` chain as quorum_cert's pigeonhole bound.** Two-line proof: (1) `assert(m1.dom().subset_of(m2.dom()));` — Verus derives the subset from `submap_of`'s definition (`forall k. m1.dom().contains(k) ==> m2.dom().contains(k) && m1[k] == m2[k]`); (2) `vstd::set_lib::lemma_len_subset(m1.dom(), m2.dom());` — given `s1.subset_of(s2)` and `s2.finite()`, ensures `s1.finite()` (and a length bound we don't need). No need to mention `Map::dom` axioms explicitly; the open-spec-fn definitions of `submap_of` and `subset_of` chain through.
- **The intermediate `subset_of` assert is load-bearing.** Without it, `lemma_len_subset`'s precondition `m1.dom().subset_of(m2.dom())` doesn't auto-fire from `submap_of` — the value-equality conjunct in `submap_of` confuses the SMT trigger search. The one-line assert makes the subset relation directly available with no other consequences.

### MA__bin_sizes__shift_is_div [VeruSAGE-Bench] (attempt 1, success)
- **Bit-shift-equals-div proofs reuse `vstd::bits::lemma_u64_shr_is_div` + a one-direction `pow2` bridge.** vstd already proves `(x >> shift) == x as nat / vstd::arithmetic::power2::pow2(shift as nat)` for u8/u16/u32/u64/u128/usize via a step-by-4 induction with `by (bit_vector)` leaves. The only new work needed when the exercise defines its own `pow2` (over `int` here, not `nat`) is a tiny `proof fn` bridge `local_pow2(n as int) == vstd_pow2(n)` by induction on `n`. Total proof: 1 bridge lemma + 3 lines in the main `proof fn` (vstd lemma call, bridge call, one `as nat as int == as int` defensive assert).
- **Bridge-lemma base case via `lemma2_to64()`.** `vstd::pow2(0)` desugars through `pow(2, 0)` which is `#[verifier::opaque]`, so `assert(vstd::pow2(0) == 1)` won't fire from definitions alone. `vstd::arithmetic::power2::lemma2_to64()` gives `pow2(0)..pow2(32)` and `pow2(64)` as concrete values via one `by (compute_only)` inside vstd — cheaper than trying to `reveal(pow)` locally. Inductive step uses `lemma_pow2_unfold(n)` which gives `vstd_pow2(n) == 2 * vstd_pow2((n - 1) as nat)`; combined with the local definition `pow2(n as int) == pow2(n as int - 1) * 2` (commutative `* 2`), Verus closes the equality with no further nudging.

### IR__verus_extra__lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity [VeruSAGE-Bench] (attempt 1, success)
- **`Seq::filter` proofs close via `reveal(Seq::filter)` + induction on `s.len()` with `decreases s.len()`.** The vstd `filter` is `pub open spec fn` but its recursive body needs a `reveal(Seq::filter)` at the call site to unfold (it's annotated `decreases self.len()` and the SMT solver does not auto-unfold recursive opens). Pattern: base case `s.len() == 0` closes by `assert(s.filter(pred) =~= s)`; inductive case recurses on `s.drop_last()` after a one-line `assert forall|i| ... implies pred(s2[i]) by { assert(s2[i] == s[i]); }` to re-establish the per-element precondition, then collapses the post via two `=~=` asserts (`s.filter(pred) =~= s2.push(s.last())` from the filter body unfold, and `s2.push(s.last()) =~= s` extensional).
- **`pred(s.last())` requires an explicit `assert(s.last() == s[s.len() - 1])` bridge.** The precondition is stated with explicit indexing `s[i]`; the `last()` open spec fn reads `self[self.len() - 1]`. SMT does not chain these automatically inside an `assert(pred(s.last())) by { ... }` block — a single intermediate equality assert closes it.

### NR__definitions_u__lemma_maxphyaddr_facts [VeruSAGE-Bench] (attempt 1, success)
- **Bit-shift bounds over an axiomatized width close in 3 lines: axiom call + 2 compute asserts + 1 bit_vector monotonicity forall.** Goal is `0xFFFFFFFF <= ((1usize << w) - 1) as usize <= 0xFFFFFFFFFFFFF` for `w: usize` constrained by an axiom to `32 <= w <= 52`. Body: (1) call the axiom to get the width range into scope; (2) `assert(1usize << 32 == 0x100000000) by (compute);` and `assert(1usize << 52 == 0x10000000000000) by (compute);` to pin the endpoint values literally; (3) `assert(forall|m: usize, n: usize| n <= m < 64 ==> 1usize << n <= 1usize << m) by (bit_vector);` to give Verus monotonicity. SMT then chains `1usize << 32 <= 1usize << w <= 1usize << 52` and subtracts 1 from each side. Total proof body: 4 lines.
- **`compute` mode handles `1usize << k` for concrete `k`; `bit_vector` mode handles the universally-quantified monotonicity.** Mixing the two is the load-bearing trick here — neither alone closes the goal. `compute` won't quantify over the abstract `w`, and `bit_vector` on a forall-shift fact does not by itself give you the concrete numeric values of the endpoints.
- **`#[verifier::when_used_as_spec(MAX_PHYADDR_SPEC)]` on an `exec const` is the bridge from exec to spec.** When the lemma's ensures clause uses the exec-side identifier `MAX_PHYADDR`, the annotation tells Verus to resolve it to `MAX_PHYADDR_SPEC` in spec position. No manual unfold, no `reveal`, no extra cast needed.

### VE__utils__init_vec_u8 [VeruSAGE-Bench] (attempt 1, success)
- **Counted-`while`-fills-`Vec` is the same two-conjunct pattern as `binary_search`'s loop.** Invariant `i <= n && ret@.len() == i` plus `decreases n - i` is sufficient; `Vec::push`'s built-in spec auto-discharges the `ret@.len() == i + 1` step on each iteration without a defensive `assert(ret@ =~= ...)` nudge. The post-loop obligation `ret@.len() == n` collapses from `i == n` (negated guard + `i <= n`) and `ret@.len() == i`.
- **No `push`-frame assert needed for this size of goal.** The bounded_log discovery note (push + frame assert) was a caveat for `&mut self` postconditions across a `Vec::push` mutation; in a pure exec body where the postcondition only constrains `ret@.len()` (not element-wise content), the SMT solver closes the length step from `push`'s ensures alone. Reserved the pattern in case future VE tasks ask for element-content postconditions.

### quorum_cert (attempts 1–6, success)
- **Pigeonhole-via-contradiction pattern**: The `lemma_qc_has_honest_voter` style proof is best written as `if !(exists honest. P(honest)) { ... assert(false); }`. Inside that branch the negated existential gives `forall h. !P(h)`, which an `assert forall ... implies ... by { }` block can convert into a subset relation. Combined with `vstd::set_lib::lemma_len_subset`, the cardinality contradiction closes.
- **`vstd::arithmetic::div_mod::lemma_fundamental_div_mod(x, d)` is the right primitive for div/mod arithmetic**: `nonlinear_arith` does NOT know the basic euclidean identity `x == d * (x/d) + (x%d)`. Call this lemma explicitly with `int`-typed args, then bridge to `nat`. The remainder bound `0 <= r < 3` is known by default; the identity is not.
- **`lemma_len_subset` requires the *superset* finite**: `vstd::set_lib::lemma_len_subset(s1, s2)` ensures `s1.finite() && s1.len() <= s2.len()` given `s1.subset_of(s2)` and `s2.finite()`. Use it both to lift finiteness from a universe set (`{k : k < n}`) to the abstract `voters(qc)` set, and to bound `|voters(qc)| <= |byzantine|` after deriving subset under contradiction.
- **Bitmap-backed single-pass structural checks**: For "distinct voters in range + threshold" (`verify_qc_structure`), use a `Vec<bool>` seen-bitmap of length `n`. Loop invariant has four conjuncts: cursor bounds, in-range prefix, pairwise-distinct prefix, bitmap-vs-prefix abstraction (`seen@[k] == exists j < i. voter(j) == k`). Re-establishing (c)/(d) in the fall-through branch needs `seen.set` frame asserts and a captured "v is not yet in the prefix" fact derived by reading (d) at `k = v_id` as a contrapositive.
- **Bridge `voters(qc).len()` ↔ `qc.votes.len()` via Seq projection**: An *internal* `spec fn voter_seq(qc) -> Seq<NodeId>` projects votes onto NodeIds. Then `voters(qc) =~= voter_seq(qc).to_set()` (extensional set equality), and under `voters_distinct`, `voter_seq(qc).to_set().len() == voter_seq(qc).len() == qc.votes@.len()` via an induction on Seq length using `lemma_push_to_set` + `axiom_set_insert_len`.
