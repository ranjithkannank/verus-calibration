# Wiring a Formal Verifier into an Autonomous Coding Loop

The previous four posts in this series tightened the feedback signal an
autonomous coding loop runs against. First plain tests. Then mutation
testing, to check the tests caught real bugs. Then splitting the
auditor from the writer, so the loop couldn't grade itself. Then
catching the gap between green tests and integration contracts. Each
step closed a hole through which a wrong loop could still pass.

A formal verifier is the limit of that progression. It is a feedback
signal the agent cannot satisfy except by either weakening the
specification or actually being correct. The methodology of this post
is the rule that closes the first path — *no spec weakening, audited
at two boundaries, on every commit* — and the empirical question is
whether the loop can still produce real verified code under that
constraint.

I picked Verus, because it operates on real Rust rather than on a
custom language. I picked three exercises of increasing difficulty: a
sorted binary search, a fixed-capacity append-only log with a frame
property, and a Byzantine quorum check whose spec talks about
mathematical set cardinality but whose implementation has to walk a
`Vec`. The first two are textbook; the third is where the
abstraction-to-implementation gap actually shows up.

This post is what came out of it.

## The setup

Three roles, three Claude Code subagents, three (mostly two) models:

- **Architect** (Opus 4.7) — reads the frozen spec, writes a design note. Does not see verifier output on the first pass.
- **Implementer** (Sonnet 4.6) — one attempt per call: edit the file, run verus, log the result.
- **Reviewer** (Opus 4.7) — after verus passes, audits the diff against the frozen baseline. Returns `APPROVE` or `REJECT`.

The reviewer is a separate role for the same reason the previous post
argued for splitting audit from decision. The architect committed to a
design that produced the implementation; asking it to audit that same
implementation for spec drift bakes in confirmation bias. A separate
audit on a fresh context is a cheap structural safeguard.

The three roles are wired together by a Ralph-style outer loop — bash
— that reads state from filesystem artifacts and fires one `claude
-p` call per iteration with fresh context every time. Memory lives in
`AGENTS.md`, the design note, `attempts.md`, and git history. The
state machine:

```
                  ┌─────────────────────────────────┐
                  ▼                                 │
START ──► THINK ──► WORK ──► (verus passes) ──► REVIEW
                     │                            │   │
                     │                            │   ├─► APPROVE ──► DONE
                     │                            │   │
                     │                            │   └─► REJECT ──► WORK_AFTER_REJECT
                     │
                     └─► (escalation) ──► THINK_REVISE ──► WORK
```

State is *inferred from the filesystem* each iteration — presence of
the design note means past THINK, number of entries in `attempts.md`
is the current attempt count, the `Conclusion` line in `review.md`
drives APPROVE vs REJECT. There's no explicit state file. That means
the loop is fully resumable: kill it, re-run, it picks up where it
left off.

Each implementer attempt is its own git commit. That's load-bearing
for two reasons: the reviewer's diff target is a stable snapshot, and
when something goes wrong later you can quote the actual code.

## The two sandbox layers

A loop that runs unattended needs boundaries. There are two.

**Content boundary — a pre-commit hook.** Every commit goes through
`scripts/git-hooks/pre-commit`. It rejects three things:

1. Any staged file outside a path whitelist (`exercises/`, `logs/`,
   `writeup/`, `ralph/`, `scripts/`, `.claude/`, the named top-level
   docs).
2. Any cheat token added in `exercises/*.rs`: `assume(`,
   `#[verifier::external_body]`, `unreachable!()`, `panic!(`,
   `assume_specification`.
3. Spec drift — every line in the frozen baseline whose first token
   is `requires` or `ensures` must appear *verbatim* in the staged
   file. New helper `spec fn`s are allowed; modifying frozen ones is
   not.

**Capability boundary — a Claude Code tool whitelist.** Each `claude
-p` call passes a role-scoped `--allowedTools` list and a universal
`--disallowedTools` deny set. The deny set includes `WebFetch`,
`WebSearch`, `Task` (no sub-agent fan-out), bash patterns for `rm`,
`git push/reset/rebase/checkout/restore/config`, `*--no-verify*` so
the agent can't bypass the hook, `curl/wget/ssh`, `brew/npm/pip`,
`sudo`. The architect gets only the tools it needs to write a
design note; the reviewer doesn't get `Edit`.

To be clear about what this is not: it's not process isolation, it's
not a network namespace, and a deliberately adversarial agent could
find paths around it. The agents in this experiment are not
adversarial. The boundary's job is to make it impossible for an
honest worker to *accidentally* cheat — to silently weaken a spec
because the verifier complained, or to slip an `assume` past the
reviewer because it was tired. That boundary held across all three
exercises.

## What happened

| Exercise         | Status | Attempts to verify | Notable                                         |
|------------------|--------|--------------------|-------------------------------------------------|
| binary_search    | DONE   | 1                  | Architect's design predicted every invariant.   |
| bounded_log      | DONE   | 1 (post re-freeze) | The methodology earned its keep here.           |
| quorum_count     | DONE   | 2                  | Real concrete-to-abstract proof engineering.    |

### binary_search

The easiest. A verified binary search with a sortedness precondition
and the standard "found / not-found" postconditions.

The architect's design note ran to about 200 lines and predicted
exactly what the proof would need: a half-open window `[lo, hi)`
(justified — *"avoids the `hi = mid - 1` underflow problem; `usize`
cannot go negative"*), an overflow-safe `mid = lo + (hi - lo) / 2`,
and the five conjuncts of the loop invariant including the two
`forall` exclusion ranges that tile the index space on loop exit. The
implementer then wrote a body that matched the design almost exactly.
First try, verus passed, reviewer approved.

The implementer also appended four discovered patterns to `AGENTS.md`
on its own — `decreases` clauses are required on `while` loops, the
`assert forall ... by { assert(is_sorted(...)); }` block reliably
triggers sortedness instantiation, half-open avoids underflow, the
invariant should be five conjuncts in this shape. The role file's
escape hatch — *"append findings to the Discovered patterns
section"* — turned into a memory artifact that future exercises
could read.

### bounded_log

This is the exercise where the methodology earned its keep.

The spec is a fixed-capacity append-only log with `new`, `len`, `get`,
and an `append` whose postcondition includes a frame property — *the
existing entries don't change* — which is the kind of obligation SMT
solvers handle unevenly.

On the first attempt, the implementer wrote `final(self)` everywhere
in the `ensures` clause of `append`. Verus accepted: `4 verified, 0
errors`. The reviewer then ran the audit against the frozen tag and
rejected:

> The implementer rewrote six lines inside the `ensures` block of
> `append`, changing every post-state reference to `self` into
> `final(self)`. ... Even granting the implementer's claim ... that
> `final(self)` is semantically equivalent to the post-state `self` ...
> this is not byte-identical and therefore falls under rule 1.

Reviewer rule firing exactly as designed. The implementer had even
added a discovered-pattern note claiming `final(self)` was required by
the current Verus version — and it turned out to be right.

On the second attempt, the implementer dutifully restored bare `self`
to match the frozen spec, and Verus rejected it with a clear migration
error pointing at the new `&mut self` postcondition disambiguation
rule. Both paths now violated a rule: one from the reviewer, one from
the compiler. The implementer wrote a structured blocker report:

> | Constraint                    | What it requires                                                                |
> |-------------------------------|---------------------------------------------------------------------------------|
> | Frozen spec (reviewer rule 1) | `self.well_formed()`, `self.capacity()`, `self.view()` (bare `self`)            |
> | Verus 0.2026.05.13 syntax     | `final(self).well_formed()`, `final(self).capacity()`, `final(self).view()`     |
>
> These two constraints are mutually exclusive. No implementer-level
> change can satisfy both simultaneously. Only the architect (per
> AGENTS.md) is empowered to re-freeze the spec.

This is the outcome you actually want from a methodology that is
intended to be trustworthy. The agent did not silently bypass the
reviewer's rule with `final(self)`. It did not silently bypass Verus
by adding an `assume`. It articulated the conflict, named the role
empowered to resolve it, and stopped.

The frozen spec I had written was wrong — it predated the Verus
version on the machine. I re-froze the baseline (a single targeted
commit moving every post-state `self` to `final(self)`, leaving
function bodies as `unimplemented!()`) and the loop restarted clean
against the corrected tag. First-try clean: architect → one
implementer attempt → reviewer APPROVE → DONE. The proof body was
nearly identical to the rejected version; only the spec text in the
frozen tag had changed.

The methodology survived its own author's mistake. That's not nothing.

### quorum_count

The hardest, by design. The spec defines `distinct_count(voters) =
voters.to_set().len()` — mathematical set cardinality. The
implementation has to walk a `Vec<NodeId>` and count distinct elements
with some concrete data structure. The proof has to bridge those two
worlds.

The architect proposed a bitmap-backed approach: a `Vec<bool>` of
length `n` plus a `u64` counter, one linear pass, O(1) per step. The
proof reduces to three helper lemmas:

1. `s.subrange(0, i+1) == s.subrange(0, i).push(s[i])` — prefix step.
2. `s.push(x).to_set() == s.to_set().insert(x)` — concrete-to-abstract bridge.
3. `s.insert(x).len() == s.len()` when `x ∈ s` — already-present insertion is identity.

On attempt 1 the implementer wrote about 330 lines covering the
algorithm and all three architect-predicted lemmas plus two more it
decided it needed. Verus produced `5 verified, 2 errors`:

```
error: invariant not satisfied before loop
   --> exercises/quorum_count.rs:153:13
    |
153 |             count as nat == voters@.subrange(0, i as int).to_set().len(),

error: assertion failed
   --> exercises/quorum_count.rs:254:20
    |
254 |             assert(count as nat <= n as nat);
```

The `attempts.md` entry diagnosed both failures precisely — the
empty-subrange `to_set().len()` not seen as 0 at loop entry, and the
`count <= n` bound after increment needing a pigeonhole argument. It
proposed specific fixes. The autoresearch loop working as designed:
each attempt is a measurable, named outcome that feeds the next.

On attempt 2 the implementer did something more interesting than I'd
predicted. Rather than guessing at lemma names, it grepped the local
`vstd` source for relevant helpers. It found
`vstd::set_lib::lemma_len_subset(s1, s2)` which proves
`s1.len() <= s2.len()` when `s1 ⊆ s2` and `s2.finite()`. It found
`lemma_int_range(lo, hi)` which proves a range set is finite with
cardinality `hi - lo`. It noticed the type mismatch — `voters.to_set()`
is `Set<NodeId>` but `lemma_int_range` is on `Set<int>` — and wrote
a new helper `lemma_range_nodeid_len(n: u32)` as the NodeId analogue
by structural recursion on `u32`.

It also caught its own regression. After making the surgical proof
additions, it ran verus and got a Rust-level type error in its own
just-added code: `u32 - 1` evaluates to `int` inside a Verus `proof
fn` context, and the subtraction needed a cast. It read the verus
output, recognized the new failure pattern, and patched the cast in
the same iteration. Then re-ran:

```
verification results:: 8 verified, 0 errors
```

The reviewer's audit took under a minute. Five-point checklist
passed with specific line citations, plus a substantive cross-exercise
observation:

> The implementer leans heavily on `=~=` extensional equality and
> `choose` witnesses to push set/seq reasoning through the SMT solver;
> this pattern recurred in `bounded_log` too and is worth canonizing
> in the architect's playbook.

That is the audit role doing more than gatekeeping. It is
accumulating institutional knowledge across exercises and proposing
it be promoted to architect-level guidance. The methodology working
as designed.

## What the loop got right

Three claims I'd be willing to defend from this run.

**The no-spec-weakening rule held under pressure.** Both the
mechanical hook and the semantic reviewer caught real violations.
The hook caught a body-content modification on bounded_log that
matched no other rule. The reviewer caught a spec-shape modification
that the hook missed because its check is keyword-prefix only. Two
layers that fail differently is the boundary; one layer alone wasn't
enough.

**Per-attempt commits made the methodology auditable.** Every
implementer iteration is its own commit, with the verifier output
captured under `logs/<ex>/raw/`. The reviewer's audits cite specific
HEAD lines. The git log is a usable timeline for the failure
taxonomy. None of this needed extra tooling — git was sufficient,
once the orchestrator was disciplined about committing on each attempt.

**The role split bought what it advertised.** The architect produced
substantive design notes that the implementers actually followed. The
implementers self-diagnosed their own failures and named follow-up
plans rather than thrashing. The reviewer audited the diff with
checklist rigor and contributed cross-exercise pattern observations.
Could a single Opus call have done all three jobs? Probably. Would
its review of its own work be as honest as a separate role's? I doubt
it.

## What the loop got wrong

Three honest shortcomings worth flagging.

**The hook's spec-preservation check has a known gap.** It looks for
lines whose first token is `requires` or `ensures`, but the body
content of those clauses — the continuation lines — is not on the
frozen-line list. The bounded_log diff changed those body lines, and
the hook let it through. The reviewer caught it, which is exactly
what splitting audit from decision predicts would happen, but the
hook should be tighter. The fix is line-range matching instead of
line-prefix matching.

**The orchestrator treats every non-zero claude exit code as
verus-failed.** It isn't. An exit code can also mean a rate-limit
response, a budget cap firing, a network blip. The orchestrator
should distinguish these and recover appropriately, not just count
another failed attempt and try again. On a longer run, this would
produce wasted iterations against transient infrastructure issues.

**Three exercises is not a benchmark.** The vericoding paper has
12,504 specifications. This has three. The generalizations I'd be
comfortable drawing are at the level of *this failure mode exists*
or *this rule survived this pressure*, not *success rate equals
X percent*. Sample size three is sample size three.

## Where this fits

The vericoding benchmark paper measures success rates on isolated
single-function tasks. Huntley's Ralph posts describe the outer-loop
pattern in domains where the feedback signal is tests. Karpathy's
autoresearch demos use a scalar metric and a single agent. The
intersection — multi-function Verus code, an autonomous loop, a
formal verifier as the feedback signal, an explicit no-cheating
sandbox, multi-model role splitting — is where this experiment sat,
and there isn't much public work in that intersection yet.

What's also new here, more from accident than design: the
operator-intervention case on bounded_log. Most published
vericoding results either succeed silently or fail silently. The
loop's behavior on the bounded_log conflict — agent refused to cheat
in either direction, articulated the constraint, named the empowered
role, stopped — is the kind of structured-failure output you want
from a methodology that's intended to be trustworthy. A single-shot
prompt would either silently apply `final(self)` or silently fail.
This calibration surfaced the conflict, blamed the right party (the
operator, not the agent), and waited for an operator decision.

The trust ladder has another rung now. The next rung is harder.

## Reproducing

- Repo: <https://github.com/ranjithkannank/verus-calibration>
- Verus: `0.2026.05.13.fae8859`, arm64-macos binary release
- Models: `claude-opus-4-7` for architect and reviewer,
  `claude-sonnet-4-6` for implementer (the bash script holds the
  source of truth; subagent frontmatter declares defaults, but the
  outer loop overrides per call)
- All prompts are inlined in `ralph/run-exercise.sh`. All raw
  verifier output is committed under `logs/<ex>/raw/`. `AGENTS.md`
  is the rule book; the pre-commit hook in
  `scripts/git-hooks/pre-commit` is the enforcement.

Code: MIT. Writing: CC BY 4.0.
