# Vericoding in the Loop: A Calibration Against Three Verus Exercises

<!--
Status: complete draft of the 2026-05-15 run.
This document is the source for a two-post split on the blog: methodology
post (sections 1-3, 5, 8) and results post (sections 4, 6, 7, 9, 10).
-->

The previous four posts in this series tightened the feedback signal an
autonomous coding loop runs against. First plain tests. Then mutation
testing to check the tests caught real bugs. Then splitting the auditor
from the writer so the loop couldn't grade itself. Then catching the gap
between green tests and integration contracts.

A formal verifier is the limit of that progression. It is a feedback
signal the agent cannot satisfy except by either weakening the
specification or actually being correct. The whole methodology of this
post is the rule that closes path (a) — *no spec weakening, audited at
two boundaries, on every commit* — and the empirical test is whether
the loop can still produce real verified code under that constraint.

I ran the calibration on a Saturday's worth of work compressed into one
afternoon. Three Verus exercises of increasing difficulty, three roles
on different Claude models, two sandbox layers, one autonomous outer
loop. All three exercises ended in `verus: 0 errors` and `reviewer:
APPROVE`. Getting there surfaced more orchestrator failure modes than
verifier failures — the bottleneck on the hardest exercise was the
*wrapper*, not the proof. This post is what came out of it.

## 1. The problem

Vericoding — using a language model to fill in code body + proof
annotations against a frozen formal spec, with the verifier as the
arbiter — is a recent enough idea that the empirical literature on it
is small. The vericoding benchmark (Schubert et al., 2025) reports
44% first-try success on small isolated Verus functions. That is
encouraging in the same way 44% on LeetCode would be encouraging:
calibrating on toy problems is a necessary first step, not a
conclusion.

What I wanted to measure is what happens when you push past the toy.
Specifically:

- Multi-function code with cross-function invariants, not isolated
  exercises.
- A real autonomous loop, not a single-shot prompt.
- A loop that *can* try to cheat (the agent has access to the file,
  including the spec) and the methodology rules out the cheats it
  would naturally reach for.
- Two-model role splitting (Opus for design and audit, Sonnet for
  implementation) instead of one model wearing all hats.

The aerospace pivot motivating this calibration sits one layer out: if
this works for small primitives, then a verified Byzantine-fault-tolerant
sensor fusion system becomes a plausible quarter-long project rather
than a multi-year research endeavor. The calibration is the cheapest
test of whether the building block is reachable in practice.

## 2. Architecture

Three Claude Code subagents, three different responsibilities, three
different (mostly two different) models:

| Role        | Model              | Job                                                                                                |
|-------------|--------------------|----------------------------------------------------------------------------------------------------|
| Architect   | `claude-opus-4-7`  | Reads frozen spec, writes a design note. Does not see verifier output on first pass.               |
| Implementer | `claude-sonnet-4-6` | One verus attempt per call: edit, run verus, log, commit. Stops at cap or on escalation.           |
| Reviewer    | `claude-opus-4-7`  | After verus passes, audits the diff against the frozen tag. Returns APPROVE or REJECT.             |

The reason the reviewer is a separate Opus call rather than the
architect's second hat: it's the previous post's argument, ported. The
writer should not grade itself. The architect committed to a design
that produced the implementation; asking it to then audit that same
implementation for spec drift bakes in confirmation bias. A separate
audit role on a separate context is a cheap structural safeguard.

The roles are wired together by a Ralph-style outer loop — `bash` —
that reads state from filesystem artifacts and fires one `claude -p`
call per iteration with **fresh context every time** (`--no-session-persistence`).
This is the Huntley pattern. Memory lives in `AGENTS.md`, the design
note, `logs/<ex>/attempts.md`, and git history. Nothing about the
loop's reasoning survives a context reset, which forces the agents to
write down what matters in versioned files. The state machine:

```
                  ┌─────────────────────────────────┐
                  ▼                                 │
START ──► THINK ──► WORK ──► (verus passes) ──► REVIEW
                     │                            │   │
                     │                            │   ├─► APPROVE ──► DONE
                     │                            │   │
                     │                            │   └─► REJECT ──► WORK_AFTER_REJECT ──┐
                     │                            │                                       │
                     │                            └───────────────────────────────────────┘
                     │
                     ├─► (escalation written by worker) ──► THINK_REVISE ──► WORK
                     │
                     └─► (cap hit) ──► CAP_HIT / BLOCKED
```

State is *inferred from the filesystem* each iteration. Presence of
`exercises/<ex>.design.md` means past THINK. Number of entries in
`logs/<ex>/attempts.md` is the current attempt count. A `status` file
holds the last attempt's terminal signal (`verus_passed`, `verus_failed`,
`escalated`). The `logs/<ex>/review.md` Conclusion line drives the
APPROVE/REJECT split. That means the loop is fully resumable — Ctrl-C
any time, re-run, it picks up at the next state by reading what's on
disk.

This is also the autoresearch part: each implementer attempt is a
fixed-budget experiment with a binary outcome (verus exits 0 or it
doesn't), comparable across attempts and across exercises. The metric
isn't loss; it's whether the verifier accepts.

## 3. The two sandbox layers

A loop that runs unattended needs to be sandboxed. I did not put this
behind Docker — partly because the canonical Verus binary release for
Apple Silicon doesn't have a matching arm64-linux build, partly
because for a weekend experiment with the operator watching the first
iterations a heavier wrapper buys less than I'd spend on it. Instead I
built two narrower boundaries.

**Layer 1 — content boundary: git pre-commit hook.** Symlinked into
`.git/hooks/pre-commit` from `scripts/git-hooks/`. On every commit
attempt the hook rejects:

- Files outside a path whitelist (`exercises/`, `logs/`, `writeup/`,
  `ralph/`, `scripts/`, `.claude/`, and a few named top-level docs).
- Cheat tokens added in any `exercises/*.rs` file: `assume(`,
  `#[verifier::external_body]`, `unreachable!()`, `panic!(`,
  `assume_specification`.
- Spec drift — every line in the frozen baseline (the git tag
  `spec-frozen-<exercise>`) whose first non-whitespace token is
  `requires` or `ensures` must appear *verbatim, byte-identical* in
  the staged file. New helper `spec fn`s are allowed; modifying
  frozen ones is not.

The hook is short — ~80 lines of bash — and deliberately strict. It
catches the obvious cheats without trying to be a parser.

**Layer 2 — capability boundary: Claude Code tool whitelist.** Each
`claude -p` call passes a role-scoped `--allowedTools` list and a
universal `--disallowedTools` deny set. The architect gets `Read`,
`Write`, `Glob`, `Grep`, plus narrow `Bash(git *)` patterns. The
implementer adds `Edit` and `Bash(verus *)`. The reviewer drops `Edit`.
The deny set includes `WebFetch`, `WebSearch`, `Task` (no sub-agent
fan-out), bash patterns for `rm`, `git push/reset/rebase/checkout/restore/config`,
`*--no-verify*` (so the agent can't bypass the hook),
`curl/wget/ssh/scp`, `brew/npm/pip/cargo install`, `sudo`.

The `--no-verify` deny is what keeps layer 1 load-bearing. If the
agent could ask the orchestrator to bypass the hook, all of this would
be theater.

Honest limitations on this sandbox model — worth stating plainly so
the writeup isn't claiming more than it earned:

- This is not process isolation. The `Write` tool isn't path-restricted
  the way `Bash` patterns are. A file written outside the repo wouldn't
  be caught until the commit attempt, and a side-effect that doesn't
  get committed wouldn't be caught at all.
- The hook's spec-preservation check covers *lines whose first token is
  `requires` or `ensures`*, not the entire body of those clauses. A
  pre-existing line like `        self.well_formed(),` (a continuation
  of an `ensures` block) is not on the frozen-line list. I'll come back
  to this gap — it's exactly where the loop's most interesting failure
  mode lived.
- Compound bash commands like `verus foo; rm -rf bar` — pattern
  matching against `Bash(verus *)` may or may not match the whole
  pipeline depending on Claude Code's matcher semantics. The disallow
  list explicitly blocks `Bash(rm *)` as belt-and-suspenders.

A motivated adversarial agent could find paths around all of this.
The agents in this experiment are not adversarial — they want to
verify code, not to escape. But the load-bearing claim of the
methodology is that they *can't successfully cheat even if they wanted
to,* and that's the boundary I wanted to make load-bearing in writing.

## 4. What happened, exercise by exercise

Results at a glance:

| Exercise         | Status | Attempts | Implementer wall time | Notable                                          |
|------------------|--------|----------|-----------------------|--------------------------------------------------|
| binary_search    | DONE   | 1        | ~2 min                | Clean. Architect's design predicted every invariant. |
| bounded_log      | DONE   | 1 (post re-freeze) | ~2 min      | Surfaced a Verus version mismatch the operator (me) had introduced; the implementer's blocker report was the cleanest evidence of methodology working. |
| quorum_count     | DONE   | 2 (attempt 2 via direct streaming, not the wrapper) | ~30 min for attempt 2 | Genuine concrete-to-abstract proof work. The orchestrator wrapper, not the proof itself, was the bottleneck. |

The next three subsections tell each story.

### 4.1 binary_search — clean first-try

The easiest of the three. Spec: a verified binary search on a sorted
`Vec<i64>` returning `Option<usize>` with a precondition that the
input is sorted and the standard "found / not-found" postconditions.

End-to-end timing:

```
THINK (Opus, 60s)        →  exercises/binary_search.design.md (7.6 KB)
WORK  (Sonnet, 120s)     →  binary_search.rs verified (2 verified, 0 errors)
REVIEW (Opus, 47s)       →  Conclusion: APPROVE
```

Three things worth flagging from this run.

**The architect's design note was substantive, not boilerplate.** It
chose a half-open window `[lo, hi)`, justified that choice ("avoids the
`hi = mid - 1` underflow problem; `usize` cannot go negative"),
proposed an overflow-safe `mid = lo + (hi - lo) / 2`, and predicted
the five conjuncts of the loop invariant including the two `forall`
exclusion ranges that tile the index space on loop exit. The
implementer then wrote a body that matched the design almost exactly.

**The implementer appended discovered patterns to `AGENTS.md`
unprompted.** Four bullets: `decreases hi - lo` is required (Verus
doesn't allow loops without a decreases clause), the
`assert forall ... by { assert(is_sorted(...)); }` block is a reliable
way to trigger sortedness instantiation, half-open avoids underflow,
and the invariant should be five conjuncts in this shape. The
`AGENTS.md` role file's escape hatch — *"Update this AGENTS.md with
discovered patterns"* — worked as intended. Ralph's file-based memory
accumulated across the per-exercise context resets.

**The reviewer's audit was substantive, not pro-forma.** It cited
specific HEAD line numbers, named the `is_sorted` spec function and
confirmed its body was untouched, identified the `assert ... by` blocks
as legitimate proof hints rather than verification bypasses, and added
two carry-forward notes for the next exercise. APPROVE, but on the
basis of an actual five-point check.

### 4.2 bounded_log — the operator-intervention case

This is the exercise that surfaced the most interesting methodological
failure mode of the run. Spec: a fixed-capacity append-only log with
`new` / `len` / `get` / `append`. The `append` postcondition includes
a *frame property* — *existing entries must not change* — which is the
kind of obligation SMT solvers handle unevenly.

**Attempt 1.** The implementer wrote `final(self)` everywhere in the
`ensures` clause of `append`. Verus accepted: `4 verified, 0 errors`.
The reviewer then ran the audit against the frozen tag and rejected:

> Item 1 fails outright. The implementer rewrote six lines inside the
> `ensures` block of `append`, changing every post-state reference to
> `self` into `final(self)`. The reviewer rules explicitly state:
> "Are `requires` and `ensures` clauses byte-identical to
> `spec-frozen-<exercise>`? ... Any change = REJECT." Even granting
> the implementer's claim ... that `final(self)` is semantically
> equivalent to the post-state `self` ... this is not byte-identical
> and therefore falls under rule 1.

Reviewer rule firing exactly as designed. The implementer had also
added a "discovered pattern" note to `AGENTS.md` claiming that
`final(self)` was required by the current Verus version — and that
turned out to be true.

**Attempt 2 (WORK_AFTER_REJECT).** The implementer dutifully restored
bare `self` to match the frozen spec, and Verus 0.2026.05.13 rejected
it:

```
error: to dereference a mutable reference parameter in a postcondition,
       disambiguate by wrapping it in either `old` or `final`
```

with a pointer to <https://github.com/verus-lang/verus/blob/main/source/docs/migration-mut-ref.md>.
This was a real Verus migration. The frozen spec I'd written was
incompatible with the current Verus version. The implementer
correctly identified this as an irreconcilable conflict — *both* paths
violate a rule, one rule from the reviewer and one from the compiler —
and wrote a thoughtful `blocked.md`:

> | Constraint                    | What it requires                                                                |
> |-------------------------------|---------------------------------------------------------------------------------|
> | Frozen spec (reviewer rule 1) | `self.well_formed()`, `self.capacity()`, `self.view()` (bare `self`)            |
> | Verus 0.2026.05.13 syntax     | `final(self).well_formed()`, `final(self).capacity()`, `final(self).view()`     |
>
> These two constraints are mutually exclusive. No implementer-level
> change can satisfy both simultaneously.
>
> Only the **architect** (per AGENTS.md) is empowered to re-freeze
> the spec. The correct fix is:
>
> 1. The architect updates the frozen spec to use `final(self)`
>    (semantically identical post-state reference, required by current
>    Verus).
> 2. The `spec-frozen-bounded_log` tag is force-moved to the new commit.
> 3. The reviewer re-audits against the new frozen baseline.

This was the most informative outcome of the whole run. The agent did
not cheat. It did not silently bypass the reviewer's rule with
`final(self)`; it did not silently bypass Verus by adding `assume`. It
articulated the conflict precisely, named the role empowered to resolve
it, and stopped.

**Operator intervention.** I (the human) made the call: the frozen
spec I'd written was the wrong baseline. The intervention was a single
targeted commit — replacing every post-state bare `self` in the
`ensures` of `append` with `final(self)`, leaving function bodies as
`unimplemented!()`, force-moving `spec-frozen-bounded_log` to point at
the new commit. I cleared the bounded_log artifacts so the experiment
restarted against the corrected baseline.

**Re-run.** First-try clean: architect → 1 implementer attempt → REVIEW
APPROVE → DONE. The proof body was nearly identical to attempt 1; only
the spec text in the frozen tag had changed.

The methodology survived its own author's mistake. That's not nothing.

### 4.3 quorum_count — concrete-to-abstract

The hardest of the three on paper. Spec: given a `Vec<NodeId>` of
voters (with possible duplicates) and a total node count `n`, return
`true` iff the *distinct* voters meet the standard Byzantine threshold
`2n/3 + 1`. The spec defines `distinct_count(voters) =
voters.to_set().len()` — mathematical set cardinality. The
implementation has to walk the `Vec` and count distinct elements with
some concrete data structure. The proof has to bridge those two worlds.

The architect's design proposed a bitmap-backed approach: `Vec<bool>`
of length `n` plus a `u64` counter, one linear pass, O(1) per step.
The proof obligation reduces to three helper lemmas:

1. `s.subrange(0, i+1) == s.subrange(0, i).push(s[i])` — prefix step.
2. `s.push(x).to_set() == s.to_set().insert(x)` — concrete ↔ abstract bridge.
3. `s.insert(x).len() == s.len()` when `x ∈ s` — already-present
   insertion is identity.

The orchestrator hit *three different failure modes* on this exercise
before the implementer got a real attempt at the proof:

- **Pro plan rate limit on Sonnet.** Hit immediately on the first
  implementer call after binary_search and bounded_log had consumed
  the hourly window. Every subsequent claude call returned
  `You've hit your limit · resets 10:30pm`.

- **Bogus state-machine churn.** The orchestrator treats `exit: 1`
  from a claude call as a normal verus failure and tries the next
  iteration. Twenty-seven rate-limited iterations in a row before
  hitting the outer ceiling. `attempts.md` was never written because
  no real attempt completed; the count stayed at 0 the whole time.
  Real loop-robustness bug — see §7.

- **Per-call budget cap too tight.** I had set `--max-budget-usd 2.00`
  as a safety net per call. binary_search and bounded_log fit
  comfortably under it. quorum_count's distinct-count proof did not —
  Sonnet was producing serious work (helper lemmas, ghost variables,
  careful invariant maintenance, 328 lines of actual proof) and got
  chopped mid-call by the budget. The exercise file was left in a
  syntactically invalid partial state.

The fix was small (`PER_CALL_BUDGET=20.00`), the lesson was not. A
"safety net" is only a safety net if it doesn't fire on the work you
want to do.

**Attempt 1 — real proof work, 5/7 obligations verified.** 14 minutes
of Sonnet wall time. The implementer wrote 333 lines of code and proof
including the three helper lemmas the architect predicted plus two
extras (`lemma_to_set_finite`, `lemma_set_insert_existing`). It also
discovered the correct vstd API name (`axiom_set_insert_len`, not
`lemma_set_insert_len`) on its own. Verus output:

```
error: invariant not satisfied before loop
   --> exercises/quorum_count.rs:153:13
    |
153 |             count as nat == voters@.subrange(0, i as int).to_set().len(),

error: assertion failed
   --> exercises/quorum_count.rs:254:20
    |
254 |             assert(count as nat <= n as nat);

verification results:: 5 verified, 2 errors
```

The implementer's `attempts.md` entry diagnosed both failures
precisely (empty-subrange `to_set().len()` not seen as 0 at loop entry;
the `count <= n` bound after increment needs a pigeonhole argument)
and proposed specific fixes (`lemma_range_set_finite_len(n)` plus
subset monotonicity). That's the autoresearch loop working — each
attempt produces a measurable, named outcome that feeds the next one.

**Attempt 2 — the wrapper hung. Twice.** Two consecutive Ralph
iterations exhibited the same pathology: 24+ minutes of claude-call
wall time, *zero bytes* of streaming output, exercise file untouched
since attempt 1's commit, claude process consuming CPU (sometimes 80%,
sometimes 5%) but producing nothing. No error message, no commit, no
log entry. The `--no-session-persistence` + `--agent implementer`
flag combination plus this specific file state seemed to deadlock
inside Claude Code's response handling, but I couldn't reproduce it
under simpler conditions. It is the single most operationally annoying
failure mode of the run, because it doesn't surface as an error — it
surfaces as silence.

The diagnosis came by deletion. Removing `--max-budget-usd` (which I'd
suspected first) didn't fix it. Removing `--agent implementer` and
`--no-session-persistence`, switching to direct streaming via
`--output-format stream-json --include-partial-messages`, and
re-invoking the same model with the same tool whitelist and a
hand-written prompt made the call work immediately and stream
hundreds of events as it went. Same model, same files, same task,
same restrictions on tools — what changed was the framing flags and
the output mode.

That counts as a failure of the methodology's autonomy claim. The
wrapper's promise is "you set up the experiment and walk away." On the
hardest exercise, the wrapper hung opaquely and recovery required
operator-driven streaming-mode introspection. See §7 for what I'd
fix.

**Attempt 2 — actual results, via direct streaming call.**
Single Sonnet call. $1.32. ~30 minutes wall time. Internally
multi-iteration: read all context files, grep across four vstd
modules for the right lemma names, find `lemma_len_subset` and
`lemma_int_range` in `set_lib.rs`, recognize the type mismatch
(`voters.to_set()` is `Set<NodeId>` but `set_int_range` is `Set<int>`),
write a new helper `lemma_range_nodeid_len(n: u32)` as the NodeId
analogue of `lemma_int_range`, edit the file three times in surgical
passes (one for the loop-entry hints, one for the Case A bound, one
for the new lemma), run verus, hit a Rust-level type error in its own
just-added code (`u32 - 1` evaluates to `int` inside a `proof fn`),
catch the exit code, fix the cast, re-run verus:

```
verification results:: 8 verified, 0 errors
```

The agent's self-correction on the type error matters. It would have
been easy to silently introduce a regression after fixing the two
named obligations and not notice the new failure. Instead it read its
own verus output, recognized the new error pattern, and patched it in
the same call.

**Reviewer — APPROVE, 78 seconds.** Five-point checklist passed with
specific line citations:

> The diff hunk at `exercises/quorum_count.rs` lines 149-154 shows
> `is_byzantine_quorum`'s `requires` and `ensures` appear only as
> unchanged context lines. ... The three frozen `pub open spec fn`
> declarations and `is_byzantine_quorum`'s `requires`/`ensures` appear
> only as unchanged context. The new helpers (`lemma_prefix_extend`,
> `lemma_push_to_set`, `lemma_to_set_finite`, `lemma_set_insert_new_len`,
> `lemma_set_insert_existing`, `lemma_range_nodeid_len`) have reasonable
> preconditions and ensure-clauses that look like genuine library-level
> facts, not trivializations.

The reviewer also flagged a cross-exercise pattern worth canonizing:

> The implementer leans heavily on `=~=` extensional equality and
> `choose` witnesses to push set/seq reasoning through the SMT solver;
> this pattern recurred in `bounded_log` too and is worth canonizing
> in the architect's playbook.

That's the audit role doing more than gatekeeping — it's accumulating
institutional knowledge across exercises and proposing it be promoted
to architect-level guidance for the next exercise. The methodology
working as designed.

## 5. Methodology contributions

Three small ideas, none of them mine in isolation, but the combination
isn't (to my knowledge) sitting in a public repo together yet.

**The no-spec-weakening rule, audited at two layers.** The hook is
mechanical and strict — verbatim line preservation. The reviewer is
semantic and structured — a five-point checklist that runs against the
diff against the frozen tag. The two together cover different failure
shapes. The hook catches `assume(`; the reviewer catches the agent
adding a new `requires false` clause that trivially satisfies an
otherwise honest function.

**Per-attempt commits in the autonomous loop.** Each implementer
attempt is its own git commit (`<exercise> attempt-N: <one-line>`), so
the writeup's failure taxonomy can quote actual diffs and the
reviewer's audit has a stable reference. This came out of the *audit
versus decision* post — if the audit is going to be load-bearing, the
artifact under audit has to be a frozen snapshot, not a moving target.

**Model-per-role wiring with the bash script as source of truth.** The
subagent files in `.claude/agents/` declare default models, but the
outer loop overrides them on every call with `--model claude-opus-4-7`
or `--model claude-sonnet-4-6`. That means swapping models — *what
happens if Sonnet does the review too?* — is a one-line change in a
single file, not five files. Useful when you want to vary one axis of
a calibration without rewriting subagents.

## 6. Failure taxonomy

| Failure                                | Where caught                                       | Recoverable?                                  |
|----------------------------------------|----------------------------------------------------|-----------------------------------------------|
| Spec drift (bare `self` → `final(self)`) | Reviewer (hook missed it: not on its line whitelist) | Yes — required operator re-freeze of the tag  |
| Implementer hallucination               | Implementer self-caught (wrote escalation, not pretended success) | Yes — operator intervention                    |
| Implementer regression (Rust type error mid-proof) | Implementer self-caught via verus exit code      | Yes — agent self-corrected in same call        |
| Pro plan rate limit                     | Operator (script kept retrying)                    | Required plan upgrade                          |
| Loop churning on rate-limit `exit:1`    | Operator only (loop has no signal-aware behavior)  | Code change to ralph script                    |
| Per-call budget cap too tight           | Operator (saw partial file, traced to log)         | Configuration change                           |
| Wrapper hang with `--no-session-persistence` + `--agent` | **Operator only — silently**, via OS process inspection | Bypass the wrapper for that exercise; direct streaming call |
| Bash permission filter trips on space-in-path | Implementer (verus invocation got rejected as "multiple operations") | Agent retried with relative path; fix is to not have spaces in cwd |
| Output redirection outside cwd blocked  | Implementer (write to `/tmp/` rejected by Claude Code session boundary) | Agent retried with cwd-relative path           |
| Pre-commit hook spec-line gap           | Reviewer caught what hook missed                   | Hook strengthen-to-multi-line is straightforward |

The interesting row is the first. The hook's frozen-line check looks
for lines whose first non-whitespace token is `requires` or `ensures`.
The bounded_log diff changed *body* lines of the `ensures` block —
lines like `        self.well_formed(),` and
`        result.is_ok() ==> { ... }` — and none of those start with
the keyword. The hook approved the commit. The reviewer then caught
the change because its diff-against-frozen-tag check sees full hunks
regardless of which token is first.

That's the whole argument for keeping the reviewer as a separate role
in one paragraph. Mechanical checks have shape blind spots. Semantic
audits don't share those blind spots. Two layers that fail differently
is the boundary; one layer alone isn't enough.

## 7. Limitations

To be plain about what this writeup does not get to claim.

**Small sample.** Three exercises is not a benchmark. The Schubert
vericoding paper has 12,504 specifications. This has three. The
generalizations I'd draw from this are at the level of *failure mode
existed* rather than *success rate equals X%*.

**Single operator, single model versions, single afternoon.** I am
mixing my prompt-engineering reflexes with the loop's capability.
Sonnet 4.6 and Opus 4.7 were the versions on the day. The next
six-week model cohort could change every number in this post.

**No process isolation.** The sandbox is path-and-token-shaped, not
namespace-shaped. The post-mortem of this experiment would not survive
a deliberately adversarial agent. The agents I ran were not
adversarial; they were honest workers running into rules.

**The hook has a known gap.** Section 6's first row. The fix —
extending the spec-preservation check from line-prefix matching to
multi-line clause matching — is straightforward but I did not do it
during the run. The reviewer was the load-bearing detector for this
case. That's exactly what *splitting audit from decision* predicts
would happen.

**Loop is not signal-aware.** A `exit: 1` from a claude call could
mean (a) verus rejected the implementation, (b) the call hit a rate
limit, (c) the call hit a budget cap, (d) the network blipped. The
orchestrator currently treats all of these as "verus failed, try
again," which is wrong for everything except (a). Twenty-seven
rate-limited iterations of the third exercise were spent on this
before the outer ceiling fired. The fix is small (grep the `iter_log`
for known non-verus signatures and exit with a different status); I'd
do it before any longer-running version of this experiment.

**Per-exercise budget caps need recalibration.** The $2 per-call cap
that worked fine for binary_search and bounded_log chopped quorum_count
mid-proof. A static cap isn't the right shape; either it should scale
with prior context size or it should be paired with retry-with-bigger
behavior. Removing the flag entirely was the eventual fix on a
subscription plan; on metered API it would need to come back in a
smarter form.

**The wrapper hangs silently on the hardest exercise.** The single
worst experience of the run. With `--no-session-persistence` and
`--agent implementer` set, two consecutive Sonnet calls on
quorum_count's attempt 2 produced zero bytes for 24+ minutes each. No
error, no timeout, no diagnostic — the bash script just sat waiting for
a `claude -p` invocation that never exited. The only way to make
progress was to bypass the wrapper, run `claude -p` directly with
`--output-format stream-json --include-partial-messages` (and without
those two flags), and watch events stream in real time. That worked
immediately. I don't have a confirmed root cause. The plausible
hypotheses are (a) some buffering interaction between
`--no-session-persistence` and long extended-thinking blocks, (b) a
subagent-prompt loading issue that the trivial successful runs (which
were on smaller exercises) didn't surface, or (c) something specific
about this exercise's file state. Any of those is fixable, but I'd
want a reproducer before claiming I'd fixed it.

**Spaces in the working-directory path break Bash permission
filtering.** Claude Code's filter treats `verus "/path with space/file"`
as multiple operations and partially denies it. Sonnet adapted by
retrying with a relative path. Not a hard blocker but worth knowing
before naming a project directory.

**Output redirection outside the working directory is silently
restricted.** My implementer prompt told the agent to redirect verus
output to `/tmp/quorum_attempt2.txt`. Claude Code's session-boundary
check rejected the write. The error was clear enough for the agent to
adapt, but I shouldn't have written the prompt that way in the first
place — and the Ralph script's own prompts have the same shape, so
they'd hit the same wall on a different exercise.

## 8. What's different from existing work

The Verus vericoding benchmark paper measures success rates on
isolated single-function tasks. Huntley's Ralph posts describe the
outer-loop pattern in unsafe-code domains (CRUD apps, internal tools)
where the feedback signal is tests. Karpathy's autoresearch demos use
a scalar metric (loss) and a single agent. The intersection — *real
multi-function code, an autonomous loop, a formal verifier as the
feedback signal, an explicit no-cheating sandbox, multi-model role
splitting* — is where this experiment sits, and there's not a lot of
public work in that intersection.

What's also genuinely new here, more from accident than design: two
structured-failure-mode outputs that most published vericoding work
doesn't surface.

First, the *operator-intervention case* on bounded_log. The loop's
behavior on the bounded_log conflict — agent refused to cheat in
either direction, articulated the constraint, named the empowered
role, stopped — is the kind of output you'd want from a methodology
that's intended to be trustworthy. A single-shot prompt would either
silently apply `final(self)` or silently fail; the calibration
surfaced the conflict, blamed the right party (the operator, not the
agent), and waited.

Second, the *visible wrapper hang* on quorum_count. Most published
agent-loop results report success rates, not failure shapes. The
specific shape here — claude process running at low CPU, zero bytes
streamed, no error, recoverable only by bypassing the wrapper —
isn't in the literature I've read. If anyone else is building an
autonomous Verus loop on Claude Code, this is the rake they will
step on. Writing the rake down is worth more than another 1% on a
benchmark.

## 9. What I'd do next

In order of how much they'd improve the next run of this experiment:

1. **Reproduce and fix the silent wrapper hang.** The single worst
   experience of this run. Build a smallest-failing example: which of
   `--no-session-persistence`, `--agent`, the long prompt context, or
   the specific file state triggers the deadlock? Once reproducible,
   the fix is either a watchdog (kill after N seconds of zero stream
   output) or a different invocation shape. Either is small; the
   diagnosis is the hard part.
2. **Signal-aware orchestrator.** Distinguish rate-limit, budget,
   verus-fail, network-fail, *silent-hang* in the iteration loop. Each
   gets its own recovery behavior. Specifically: detect Pro-plan
   rate-limit signatures in `iter_log` and exit with a sentinel, not
   another iteration. Add a per-iteration wall-clock budget; if it
   elapses with zero bytes streamed, kill the claude call and treat as
   the silent-hang failure.
3. **Fix the hook's spec-preservation gap.** Multi-line clause
   tracking instead of line-prefix matching. Two evenings of work.
4. **Stream-mode logging by default.** This run started in
   `--output-format text` mode (the wrapper default), so the per-call
   logs were 0 bytes during a 24-minute hang. Switching to
   `--output-format stream-json --include-partial-messages` would have
   given me the in-flight diagnostic I had to bolt on later. Cost: more
   bytes per log file, slightly heavier parsing in the monitor. Worth
   it.
5. **Per-attempt time and token measurement landed in `attempts.md`.**
   Currently we have to infer from wall-clock + ralph logs. A small
   addition to the implementer prompt + a one-line bash addition would
   give us per-attempt cost data without a separate logging story.
   Quorum_count's $1.32 single-call cost is a useful data point; I
   want that for every attempt.
6. **A fourth exercise that genuinely needs escalation.** Something
   where the architect's first design plausibly *wouldn't* work and a
   THINK_REVISE path would fire. None of the three I picked hit that
   path in this run.
7. **No spaces in the project working directory.** Trivial config fix;
   the bash permission filter doesn't handle them cleanly.

Beyond the methodology fixes, the actual aerospace-software direction:
this calibration was the lowest-cost test of whether *vericoding under
an honest sandbox* is feasible. The next layer up is a verified
Byzantine-fault-tolerant sensor fusion system across dissimilar
hardware (Pi + BeagleBone + STM32), implemented in Rust with Verus,
with empirical worst-case-execution-time measurements layered on top.
That's a quarter, not a Saturday. The calibration's role is to tell me
whether it's plausible. If the numbers from this experiment held, the
answer is yes; if they didn't, the scope-down would be a verified
primitive (a Merkle log, a quorum certificate) rather than the full
sensor fusion system.

## 10. Reproducibility

- Repo: <https://github.com/ranjithkannank/verus-calibration>
- Verus version: `0.2026.05.13.fae8859` (arm64-macos binary release)
- Claude Code version: (the version installed on 2026-05-15)
- Models: `claude-opus-4-7` for architect and reviewer,
  `claude-sonnet-4-6` for implementer
- All prompts are inlined in `ralph/run-exercise.sh` — no external
  prompt files
- All raw verifier output committed under `logs/<ex>/raw/`
- `AGENTS.md` is the rule book all roles read; the pre-commit hook in
  `scripts/git-hooks/pre-commit` is the enforcement
- If your results don't match mine, those three files are where to
  look first; the per-exercise `logs/<ex>/ralph/` directory contains
  the per-iteration agent output

Code: MIT. Writing (this file): CC BY 4.0, matching the rest of
ranjithkannan.com.
