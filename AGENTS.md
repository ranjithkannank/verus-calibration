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

### quorum_cert (attempts 1–6, success)
- **Pigeonhole-via-contradiction pattern**: The `lemma_qc_has_honest_voter` style proof is best written as `if !(exists honest. P(honest)) { ... assert(false); }`. Inside that branch the negated existential gives `forall h. !P(h)`, which an `assert forall ... implies ... by { }` block can convert into a subset relation. Combined with `vstd::set_lib::lemma_len_subset`, the cardinality contradiction closes.
- **`vstd::arithmetic::div_mod::lemma_fundamental_div_mod(x, d)` is the right primitive for div/mod arithmetic**: `nonlinear_arith` does NOT know the basic euclidean identity `x == d * (x/d) + (x%d)`. Call this lemma explicitly with `int`-typed args, then bridge to `nat`. The remainder bound `0 <= r < 3` is known by default; the identity is not.
- **`lemma_len_subset` requires the *superset* finite**: `vstd::set_lib::lemma_len_subset(s1, s2)` ensures `s1.finite() && s1.len() <= s2.len()` given `s1.subset_of(s2)` and `s2.finite()`. Use it both to lift finiteness from a universe set (`{k : k < n}`) to the abstract `voters(qc)` set, and to bound `|voters(qc)| <= |byzantine|` after deriving subset under contradiction.
- **Bitmap-backed single-pass structural checks**: For "distinct voters in range + threshold" (`verify_qc_structure`), use a `Vec<bool>` seen-bitmap of length `n`. Loop invariant has four conjuncts: cursor bounds, in-range prefix, pairwise-distinct prefix, bitmap-vs-prefix abstraction (`seen@[k] == exists j < i. voter(j) == k`). Re-establishing (c)/(d) in the fall-through branch needs `seen.set` frame asserts and a captured "v is not yet in the prefix" fact derived by reading (d) at `k = v_id` as a contrapositive.
- **Bridge `voters(qc).len()` ↔ `qc.votes.len()` via Seq projection**: An *internal* `spec fn voter_seq(qc) -> Seq<NodeId>` projects votes onto NodeIds. Then `voters(qc) =~= voter_seq(qc).to_set()` (extensional set equality), and under `voters_distinct`, `voter_seq(qc).to_set().len() == voter_seq(qc).len() == qc.votes@.len()` via an induction on Seq length using `lemma_push_to_set` + `axiom_set_insert_len`.
