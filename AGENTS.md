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

The witness is operator territory. The agent's tool whitelist does not name `*_witness.rs`; the agent never sees or modifies these files. The pre-commit hook still applies cheat-token detection to them.

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

### sensor_poll_honest (attempt 1, success)
- **The honest-voter clause is one pigeonhole lemma on top of the signed exercise.** The body of `poll` is byte-equivalent to `sensor_poll_signed`'s `poll` (check_distinct → bundle-assert → projection loop → `marzullo` → projection-lemma bridge giving `reports_containing(reports@, p_witness).len() >= n - f`). The new work is exactly one helper lemma `lemma_honest_supporter_exists(reports, p, f)` plus a one-line `choose` for the second witness inside `poll`'s proof block. No new exec code, no changes to `fusion` or `auth`.
- **The pigeonhole recipe is identical to ft_midpoint's `Lo ∪ Hi` decomposition, but inverted.** Where ft_midpoint *assumed* the existence failed and derived `|Lo ∪ Hi| >= n` for a contradiction, here we *constructively* derive `|s ∩ c| >= 1` from `|s|, |c| >= n - f` and `n >= 2f + 1`. The same four-step chain (range subset, finiteness via `lemma_len_subset`, inclusion–exclusion via `lemma_set_intersect_union_lens`, witness via `axiom_is_empty_len0`/`axiom_is_empty` + `choose`) closes the lemma in a single proof block.
- **`s + c =~= s.union(c)` is the bridge between `lemma_set_intersect_union_lens`'s output and a `.union(c)` upper bound from `subset_of(set_int_range(0, n))`.** The lemma states its conclusion with `(a + b).len()`; subset-based finiteness reasoning uses `s.union(c)`. A one-line extensional `=~=` collapses the two, then arithmetic closes `|s ∩ c| >= |s| + |c| - n`.
- **Discovery without a named load-bearing lemma.** The design note deliberately omitted the proof structure. The agent identified (a) that the new clause is a second existential, (b) that both required cardinalities collapse onto the same `[0, n)` universe, (c) that pigeonhole/inclusion–exclusion is the off-the-shelf tool, and (d) that the ft_midpoint discovery notes already encode the lemma chain. Time-to-first-success: one attempt.

### quorum_cert (attempts 1–6, success)
- **Pigeonhole-via-contradiction pattern**: The `lemma_qc_has_honest_voter` style proof is best written as `if !(exists honest. P(honest)) { ... assert(false); }`. Inside that branch the negated existential gives `forall h. !P(h)`, which an `assert forall ... implies ... by { }` block can convert into a subset relation. Combined with `vstd::set_lib::lemma_len_subset`, the cardinality contradiction closes.
- **`vstd::arithmetic::div_mod::lemma_fundamental_div_mod(x, d)` is the right primitive for div/mod arithmetic**: `nonlinear_arith` does NOT know the basic euclidean identity `x == d * (x/d) + (x%d)`. Call this lemma explicitly with `int`-typed args, then bridge to `nat`. The remainder bound `0 <= r < 3` is known by default; the identity is not.
- **`lemma_len_subset` requires the *superset* finite**: `vstd::set_lib::lemma_len_subset(s1, s2)` ensures `s1.finite() && s1.len() <= s2.len()` given `s1.subset_of(s2)` and `s2.finite()`. Use it both to lift finiteness from a universe set (`{k : k < n}`) to the abstract `voters(qc)` set, and to bound `|voters(qc)| <= |byzantine|` after deriving subset under contradiction.
- **Bitmap-backed single-pass structural checks**: For "distinct voters in range + threshold" (`verify_qc_structure`), use a `Vec<bool>` seen-bitmap of length `n`. Loop invariant has four conjuncts: cursor bounds, in-range prefix, pairwise-distinct prefix, bitmap-vs-prefix abstraction (`seen@[k] == exists j < i. voter(j) == k`). Re-establishing (c)/(d) in the fall-through branch needs `seen.set` frame asserts and a captured "v is not yet in the prefix" fact derived by reading (d) at `k = v_id` as a contrapositive.
- **Bridge `voters(qc).len()` ↔ `qc.votes.len()` via Seq projection**: An *internal* `spec fn voter_seq(qc) -> Seq<NodeId>` projects votes onto NodeIds. Then `voters(qc) =~= voter_seq(qc).to_set()` (extensional set equality), and under `voters_distinct`, `voter_seq(qc).to_set().len() == voter_seq(qc).len() == qc.votes@.len()` via an induction on Seq length using `lemma_push_to_set` + `axiom_set_insert_len`.
